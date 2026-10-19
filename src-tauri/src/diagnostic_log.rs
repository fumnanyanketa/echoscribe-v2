//! One job: point the Rust core's standard error stream at a small local log
//! file beside the database, so the diagnostic sentences the app already
//! writes with `eprintln!` land somewhere a person can read and attach, and
//! nothing about them ever leaves the machine
//! (docs/decisions/0008-instruments.md).
//!
//! It lives beside its only caller, `run()` in `lib.rs`. It is not a feature
//! and it is not a shared module: no feature calls it and no feature knows it
//! exists. The 92 `eprintln!` lines across the feature folders are not
//! converted; redirecting the stream makes them correct as they are.
//!
//! How it works. A pipe is created, the process's standard error handle is
//! pointed at the pipe's writing end, and one thread of this module's own
//! reads the other end line by line, prefixes each line with the UTC time and
//! appends it to `echoscribe.log`. On a dev build, where a console exists,
//! each line is also written to the console it would have gone to, so
//! `tauri dev` loses nothing. On a release build there is no console
//! (`main.rs` hides it) and the file is the only place the lines go.
//!
//! What a line may hold is the record's standard, enforced by a source guard
//! in this file's tests across every `.rs` file in the crate: a feature
//! prefix, a plain sentence, the library's own error. Never a transcript, a
//! key, a token, a code.
//!
//! Standing rule 14 applies to the redirect: `SetStdHandle` returning Ok says
//! the request was accepted, not that a line reached the file. The proof is a
//! line in the file, which `/check verify` reads on the installed build
//! (AC-2). For the same reason the panic hook below writes to the file
//! directly rather than trusting the stream to still be read at the moment
//! the process is dying (AC-5).

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::mem::ManuallyDrop;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::Manager;

/// Record 0008: the file's name, beside `echoscribe.sqlite3`.
pub const LOG_FILE: &str = "echoscribe.log";
/// Record 0008 AC-4: the one previous file kept when the current one is
/// rotated. Nothing else accumulates.
pub const PREVIOUS_LOG_FILE: &str = "echoscribe.log.1";
/// Record 0008 AC-4: the cap, checked once at startup. About 12,000 lines of
/// 80 bytes, which is days of ordinary use.
pub const CAP_BYTES: u64 = 1_048_576;

/// The one shared handle on the open log file. The pump thread and the panic
/// hook both write through it, one line at a time.
type Sink = Arc<Mutex<File>>;

/// Open the log file, rotating first if it is over the cap, then point the
/// process's standard error at it.
///
/// Called first in `run()`'s setup, before any feature's `init`, so every
/// line a feature says from its first moment is kept. The folder is the one
/// the database already resolves from, read through the same Tauri path
/// call the six features use, so the file and the database cannot end up in
/// different places (record 0008, value sourcing).
///
/// Returning an error here must never stop the app: the app is more useful
/// than its log. The caller says so on whatever standard error it still has
/// and carries on.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join(LOG_FILE);
    rotate_if_over_cap(&path)?;
    let file = OpenOptions::new().create(true).append(true).open(&path)?;
    let sink: Sink = Arc::new(Mutex::new(file));
    install_panic_hook(Arc::clone(&sink));
    redirect_stderr(sink)?;
    // The first line of every run, so a file that holds nothing else still
    // says the app started and which version it was. No path: a path carries
    // the Windows user name, and the file is meant to be safe to email.
    eprintln!(
        "log: EchoScribe {} started, diagnostics go to {LOG_FILE}",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

/// AC-4. If the current file is over the cap it becomes the previous file,
/// replacing whatever previous file there was, and the caller begins a fresh
/// one. A missing file is simply a first run.
fn rotate_if_over_cap(path: &Path) -> std::io::Result<()> {
    match fs::metadata(path) {
        Ok(meta) if meta.len() > CAP_BYTES => {
            let previous = path.with_file_name(PREVIOUS_LOG_FILE);
            fs::rename(path, previous)
        }
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// One line into the file, prefixed with the UTC time, flushed at once so the
/// line is on disk before whatever is said next. A poisoned lock still
/// writes: a panic on another thread is exactly the moment a line matters.
fn append_line(sink: &Mutex<File>, line: &str) {
    let mut file = sink.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _ = writeln!(file, "{} {}", now_iso8601(), line);
    let _ = file.flush();
}

/// Create the pipe, hand its writing end to the operating system as the
/// process's standard error, and start the thread that drains the other end
/// into the file.
fn redirect_stderr(sink: Sink) -> std::io::Result<()> {
    let (reader, writer) = std::io::pipe()?;
    let console = platform::point_stderr_at(&writer)?;
    // The writer is the process's standard error for the rest of its life.
    // Dropping it would close the handle under every `eprintln!` in the app,
    // so it is deliberately never dropped.
    std::mem::forget(writer);
    std::thread::Builder::new()
        .name("diagnostic-log".into())
        .spawn(move || pump(BufReader::new(reader), &sink, console))?;
    Ok(())
}

/// Drain `lines` into the file, one timestamped line each, until the reading
/// end closes. Each raw line is also echoed to `console` when there is one.
fn pump(mut lines: impl BufRead, sink: &Mutex<File>, console: Option<ManuallyDrop<File>>) {
    let mut line = String::new();
    loop {
        line.clear();
        match lines.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                append_line(sink, line.trim_end_matches(['\r', '\n']));
                if let Some(console) = console.as_ref() {
                    let mut console: &File = console;
                    let _ = console.write_all(line.as_bytes());
                }
            }
        }
    }
}

/// AC-5. A panic writes its own line straight into the file before the
/// process ends, then hands over to the hook that was there before, so a dev
/// build's console still gets Rust's usual message and backtrace.
fn install_panic_hook(sink: Sink) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned());
        let location = info.location().map(|l| (l.file(), l.line()));
        append_line(&sink, &crash_line(message.as_deref(), location));
        previous(info);
    }));
}

/// The sentence the file ends with when the app crashes. The message is
/// Rust's own panic text, which names what was expected and was not, and
/// the location is a source file and line: neither can carry what AC-3
/// forbids.
fn crash_line(message: Option<&str>, location: Option<(&str, u32)>) -> String {
    let message = message.unwrap_or("no message was given");
    match location {
        Some((file, line)) => format!("crash: EchoScribe stopped: {message} ({file}:{line})"),
        None => format!("crash: EchoScribe stopped: {message}"),
    }
}

/// The current time as `YYYY-MM-DDThh:mm:ssZ` in UTC, the same shape the
/// stored timestamps use.
///
/// This is the second copy of `sign_in::clock`'s formatter. Feature folders do
/// not import from each other and this module is not a feature, so a shared
/// module is not yet owed (AGENTS.md: three features make something shared,
/// two is a coincidence). Two copies held by a guard is the sanctioned answer
/// at two: `both_clocks_turn_days_into_dates_the_same_way` below reads both
/// files and fails the build when the date arithmetic drifts.
fn now_iso8601() -> String {
    let unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    to_iso8601(unix_secs)
}

fn to_iso8601(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs_of_day = unix_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let (hh, mm, ss) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );
    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Days since the Unix epoch to a civil (year, month, day). Howard Hinnant's
/// `civil_from_days`, which is exact for the whole range we care about.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (y + i64::from(m <= 2), m, d)
}

/// The one operating system call in this file, behind the platform boundary
/// AGENTS.md requires. Windows today; macOS gets a `dup2` here when it comes.
#[cfg(windows)]
mod platform {
    use std::fs::File;
    use std::io::PipeWriter;
    use std::mem::ManuallyDrop;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};

    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{GetStdHandle, SetStdHandle, STD_ERROR_HANDLE};

    /// Point the process's standard error at `writer`.
    ///
    /// Returns the console the stream pointed at before, when there was one,
    /// so a dev build keeps seeing its own lines. A release build hides its
    /// console and gets `None`. The returned file is wrapped so it is never
    /// closed: the handle belongs to the console, not to this module.
    ///
    /// Rust's `eprintln!` asks Windows for the standard error handle on every
    /// write rather than caching it, which is what makes switching it here
    /// reach all 92 lines without touching one of them.
    pub fn point_stderr_at(writer: &PipeWriter) -> std::io::Result<Option<ManuallyDrop<File>>> {
        // SAFETY: GetStdHandle takes no pointers and only reads process state.
        let before = unsafe { GetStdHandle(STD_ERROR_HANDLE) }
            .ok()
            .filter(|h| !h.0.is_null() && h.0 as isize != -1);
        // SAFETY: the handle is the pipe writer's, which `redirect_stderr`
        // keeps alive for the life of the process and never closes.
        unsafe { SetStdHandle(STD_ERROR_HANDLE, HANDLE(writer.as_raw_handle())) }
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        // SAFETY: the handle came from Windows for this process's console and
        // ManuallyDrop guarantees this File never calls CloseHandle on it.
        Ok(before.map(|h| ManuallyDrop::new(unsafe { File::from_raw_handle(h.0) })))
    }
}

#[cfg(not(windows))]
mod platform {
    use std::fs::File;
    use std::io::PipeWriter;
    use std::mem::ManuallyDrop;

    pub fn point_stderr_at(_writer: &PipeWriter) -> std::io::Result<Option<ManuallyDrop<File>>> {
        Err(std::io::Error::other(
            "redirecting standard error is only built for Windows (record 0008)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "echoscribe-log-{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    #[test]
    fn a_missing_file_is_a_first_run() {
        // covers: AC-4
        let dir = temp_dir("missing");
        let path = dir.join(LOG_FILE);
        rotate_if_over_cap(&path).expect("nothing to rotate is fine");
        assert!(!path.exists());
        assert!(!dir.join(PREVIOUS_LOG_FILE).exists());
    }

    #[test]
    fn a_file_under_the_cap_is_left_alone() {
        // covers: AC-4
        let dir = temp_dir("under");
        let path = dir.join(LOG_FILE);
        fs::write(&path, "one line\n").unwrap();
        rotate_if_over_cap(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "one line\n");
        assert!(!dir.join(PREVIOUS_LOG_FILE).exists());
    }

    #[test]
    fn a_file_over_the_cap_becomes_the_previous_file_and_replaces_the_old_one() {
        // covers: AC-4. One current and one previous, nothing else.
        let dir = temp_dir("over");
        let path = dir.join(LOG_FILE);
        let previous = dir.join(PREVIOUS_LOG_FILE);
        fs::write(&previous, "the older previous\n").unwrap();
        fs::write(&path, vec![b'x'; (CAP_BYTES + 1) as usize]).unwrap();
        rotate_if_over_cap(&path).unwrap();
        assert!(!path.exists(), "the over-cap file should have moved");
        assert_eq!(
            fs::metadata(&previous).unwrap().len(),
            CAP_BYTES + 1,
            "the previous file should now be the one that was over the cap"
        );
        let names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(names, vec![PREVIOUS_LOG_FILE.to_string()]);
    }

    #[test]
    fn every_line_written_through_the_pipe_lands_in_the_file_with_a_utc_time() {
        // covers: AC-2. The whole path except the handle switch itself:
        // a pipe, the pump thread, the timestamp, the flush. The switch is
        // an operating system call whose effect /check verify reads off the
        // installed build (standing rule 14).
        let dir = temp_dir("pipe");
        let path = dir.join(LOG_FILE);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        let sink: Sink = Arc::new(Mutex::new(file));
        let (reader, mut writer) = std::io::pipe().unwrap();
        let pump_sink = Arc::clone(&sink);
        let pump = std::thread::spawn(move || pump(BufReader::new(reader), &pump_sink, None));
        writeln!(writer, "dictate: the microphone could not be opened: busy").unwrap();
        writeln!(writer, "sign-in: no callback arrived within the timeout").unwrap();
        drop(writer);
        pump.join().unwrap();

        let text = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "two lines in, two lines out: {text:?}");
        for line in &lines {
            let (stamp, rest) = line.split_at(20);
            assert!(
                stamp.ends_with('Z') && stamp.as_bytes()[10] == b'T' && stamp.as_bytes()[4] == b'-',
                "each line starts with a 20 character UTC time, got {stamp:?}"
            );
            assert_eq!(&rest[..1], " ");
        }
        assert!(lines[0].ends_with("dictate: the microphone could not be opened: busy"));
        assert!(lines[1].ends_with("sign-in: no callback arrived within the timeout"));
    }

    #[test]
    fn the_crash_line_names_the_message_and_where_it_happened() {
        // covers: AC-5, the sentence. Installing the hook is process-wide and
        // would change how every other test reports a failure, so the hook's
        // installation is held by `the_panic_hook_is_installed_before_the_redirect`
        // and the live panic by /check verify.
        assert_eq!(
            crash_line(Some("called `Option::unwrap()` on a `None` value"), Some(("src/dictate/mod.rs", 42))),
            "crash: EchoScribe stopped: called `Option::unwrap()` on a `None` value (src/dictate/mod.rs:42)"
        );
        assert_eq!(
            crash_line(None, None),
            "crash: EchoScribe stopped: no message was given"
        );
    }

    #[test]
    fn the_panic_hook_is_installed_before_the_redirect() {
        // covers: AC-5, as a source guard. If the hook went in after the
        // redirect, a panic in the redirect itself would be the one crash the
        // file never saw.
        let flat = flattened(include_str!("diagnostic_log.rs"));
        let hook = flat
            .find("install_panic_hook(Arc::clone(&sink));")
            .expect("init installs the panic hook");
        let redirect = flat
            .find("redirect_stderr(sink)?;")
            .expect("init redirects standard error");
        assert!(
            hook < redirect,
            "the panic hook must be installed before standard error is redirected"
        );
    }

    #[test]
    fn a_known_moment_formats_the_same_as_the_sign_in_clock() {
        // covers: AC-2, the time on each line. Same fixtures as
        // sign_in::clock's own tests, so the two copies are proved against
        // the same facts.
        assert_eq!(to_iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(to_iso8601(1_787_923_245), "2026-08-28T13:20:45Z");
        assert_eq!(to_iso8601(1_709_208_000), "2024-02-29T12:00:00Z");
    }

    #[test]
    fn both_clocks_turn_days_into_dates_the_same_way() {
        // covers: AGENTS.md's two-copies-held-by-a-guard rule. This file and
        // sign_in/clock.rs each hold a copy of the date arithmetic. The guard
        // reads both and fails the build when they drift.
        fn body_of(source: &str, name: &str) -> String {
            let flat = flattened(source);
            let start = flat.find(name).unwrap_or_else(|| panic!("{name} missing"));
            let rest = &flat[start..];
            // From the function's opening brace to the one that closes it.
            let open = rest.find('{').expect("a function has a body");
            let mut depth = 0usize;
            for (i, c) in rest[open..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return rest[..open + i + 1].to_string();
                        }
                    }
                    _ => {}
                }
            }
            panic!("{name} has no closing brace")
        }
        for name in [
            "fncivil_from_days(z:i64)->(i64,u32,u32){",
            "fnto_iso8601(unix_secs:i64)->String{",
        ] {
            assert_eq!(
                body_of(include_str!("diagnostic_log.rs"), name),
                body_of(include_str!("sign_in/clock.rs"), name),
                "{name} differs between diagnostic_log.rs and sign_in/clock.rs. Change both or extract a shared module"
            );
        }
    }

    /// A file's own source, minus its tests, flattened so a guard survives
    /// the formatter moving a call across lines.
    fn flattened(source: &str) -> String {
        source
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    // ----- AC-3: what a diagnostic line may format, across the whole crate -----

    /// Record 0008's standard: a diagnostic line never formats a value that
    /// could be a transcript, a key, a token or a code. Loose on purpose,
    /// matched on the identifier's whole name; a false positive is reworded,
    /// never exempted.
    const FORBIDDEN: [&str; 8] = [
        "transcript",
        "text",
        "key",
        "secret",
        "token",
        "code",
        "verifier",
        "body",
    ];

    /// Every `.rs` file under `src/`, read from disk at test time because the
    /// set changes and `include_str!` cannot take a glob.
    fn every_source_file() -> Vec<(std::path::PathBuf, String)> {
        fn walk(dir: &Path, out: &mut Vec<(std::path::PathBuf, String)>) {
            for entry in fs::read_dir(dir).expect("src/ is readable") {
                let path = entry.expect("a dir entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = fs::read_to_string(&path).expect("a source file is utf-8");
                    out.push((path, text));
                }
            }
        }
        let mut out = Vec::new();
        walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
        out.sort();
        out
    }

    /// The text between the parentheses of each `eprintln!(...)` in `source`,
    /// string literals included.
    fn eprintln_invocations(source: &str) -> Vec<String> {
        const OPEN: &str = "eprintln!(";
        let mut out = Vec::new();
        let mut rest = source;
        while let Some(i) = rest.find(OPEN) {
            let start = i + OPEN.len();
            let bytes = rest.as_bytes();
            let (mut depth, mut in_str, mut prev, mut j) = (1, false, b' ', start);
            while j < bytes.len() && depth > 0 {
                let c = bytes[j];
                if in_str {
                    if c == b'"' && prev != b'\\' {
                        in_str = false;
                    }
                } else if c == b'"' {
                    in_str = true;
                } else if c == b'(' {
                    depth += 1;
                } else if c == b')' {
                    depth -= 1;
                }
                prev = c;
                j += 1;
            }
            out.push(rest[start..j - 1].to_string());
            rest = &rest[j..];
        }
        out
    }

    /// The identifiers an invocation formats: every `{name}` inside its
    /// string literals, and every identifier outside them.
    fn formatted_identifiers(invocation: &str) -> Vec<String> {
        let mut ids = Vec::new();
        let mut current = String::new();
        let mut literal = String::new();
        let (mut in_str, mut prev) = (false, ' ');
        for c in invocation.chars() {
            if in_str {
                if c == '"' && prev != '\\' {
                    in_str = false;
                    ids.extend(inline_arguments(&literal));
                    literal.clear();
                } else {
                    literal.push(c);
                }
            } else if c == '"' {
                in_str = true;
                if !current.is_empty() {
                    ids.push(std::mem::take(&mut current));
                }
            } else if c.is_alphanumeric() || c == '_' {
                current.push(c);
            } else if !current.is_empty() {
                ids.push(std::mem::take(&mut current));
            }
            prev = c;
        }
        if !current.is_empty() {
            ids.push(current);
        }
        ids
    }

    /// The `{name}` and `{name:?}` arguments inside one format string.
    /// `{{` is an escaped brace and `{}` and `{:?}` name nothing.
    fn inline_arguments(literal: &str) -> Vec<String> {
        let mut out = Vec::new();
        let chars: Vec<char> = literal.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' {
                if chars.get(i + 1) == Some(&'{') {
                    i += 2;
                    continue;
                }
                let mut name = String::new();
                let mut k = i + 1;
                while k < chars.len() && (chars[k].is_alphanumeric() || chars[k] == '_') {
                    name.push(chars[k]);
                    k += 1;
                }
                if !name.is_empty() {
                    out.push(name);
                }
                i = k;
            } else {
                i += 1;
            }
        }
        out
    }

    #[test]
    fn the_guard_itself_tells_a_formatted_key_from_the_word_key_in_a_sentence() {
        // covers: AC-3, the guard's own reading. A sentence may say "key";
        // an argument may not be one.
        let flagged = |src: &str| -> Vec<String> {
            eprintln_invocations(src)
                .iter()
                .flat_map(|inv| formatted_identifiers(inv))
                .filter(|id| FORBIDDEN.contains(&id.as_str()))
                .collect()
        };
        assert!(flagged(r#"eprintln!("dictate: could not save the key: {e}");"#).is_empty());
        assert!(flagged(r#"eprintln!("dictate: could not save the key: {}", e);"#).is_empty());
        assert_eq!(
            flagged(r#"eprintln!("dictate: saved {key}");"#),
            vec!["key"]
        );
        assert_eq!(
            flagged(r#"eprintln!("dictate: saved {key:?}");"#),
            vec!["key"]
        );
        assert_eq!(
            flagged(r#"eprintln!("dictate: saved {}", token);"#),
            vec!["token"]
        );
        assert_eq!(
            flagged(r#"eprintln!("sign-in: got {}", response.text());"#),
            vec!["text"]
        );
        assert!(flagged(r#"eprintln!("{{literal braces}} and {e}");"#).is_empty());
    }

    #[test]
    fn no_diagnostic_line_anywhere_in_the_crate_formats_a_transcript_a_key_a_token_or_a_code() {
        // covers: record 0008 AC-3 and the AGENTS.md data rules. Reads every
        // .rs file under src/ and fails on the first eprintln! whose
        // arguments name something a person's dictation, key, session or
        // sign-in could be hiding behind.
        let mut offences = Vec::new();
        let mut counted = 0;
        for (path, source) in every_source_file() {
            // Application code only. A test module's own fixtures may quote
            // a forbidden line on purpose, and nothing a test prints reaches
            // a person's log file.
            let application = source.split("#[cfg(test)]").next().unwrap_or("");
            for invocation in eprintln_invocations(application) {
                counted += 1;
                let bad: Vec<String> = formatted_identifiers(&invocation)
                    .into_iter()
                    .filter(|id| FORBIDDEN.contains(&id.as_str()))
                    .collect();
                if !bad.is_empty() {
                    offences.push(format!(
                        "{}: eprintln!({}) formats {}",
                        path.display(),
                        invocation.trim(),
                        bad.join(", ")
                    ));
                }
            }
        }
        assert!(
            counted >= 50,
            "expected the crate's eprintln! lines to be found, saw {counted}"
        );
        assert!(
            offences.is_empty(),
            "a diagnostic line formats a value record 0008 forbids. Reword it so the \
             line says what happened without the value (a false positive is reworded, \
             never exempted):\n{}",
            offences.join("\n")
        );
    }

    #[test]
    fn the_pipe_round_trip_keeps_bytes_intact() {
        // covers: AC-2. Non-ASCII in a library error must not be mangled on
        // the way to the file.
        let dir = temp_dir("bytes");
        let path = dir.join(LOG_FILE);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        let sink: Sink = Arc::new(Mutex::new(file));
        let (reader, mut writer) = std::io::pipe().unwrap();
        let pump_sink = Arc::clone(&sink);
        let pump = std::thread::spawn(move || pump(BufReader::new(reader), &pump_sink, None));
        writeln!(writer, "language: Deutsch, ääkköset, 日本語").unwrap();
        drop(writer);
        pump.join().unwrap();
        let mut text = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert!(
            text.trim_end()
                .ends_with("language: Deutsch, ääkköset, 日本語"),
            "{text:?}"
        );
    }
}
