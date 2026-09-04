//! Reading `dictation` back: record 0007's one read, and nothing else.
//!
//! Everything EchoScribe persists lives in one SQLite file on the person's own
//! machine (AGENTS.md data rules). This module opens its own connection to that
//! file, the way the dictate feature's store, the shell's, the language
//! feature's and the vocabulary feature's all do beside it.
//!
//! **This feature stores nothing.** It creates no table, runs no migration and
//! writes no row. `dictation` is the dictate feature's, created by
//! `src-tauri/src/dictate/store.rs`, whose own opening comment is the named
//! source for this arrangement: "Reading history back is plan row 5's own
//! feature and is deliberately not here." So `history::init` runs after
//! `dictate::init` in `lib.rs`, because the owner creates the table before the
//! reader opens it. A `CREATE`, an `INSERT`, an `UPDATE` or a `DELETE` in this
//! file is a bug, not a feature, and there is a test at the bottom that says so
//! by reading this file's own source.
//!
//! **Every query is scoped to one account in its `WHERE` clause**, never by
//! filtering afterwards. That is record 0002's AC-18 and record 0007's AC-9.
//!
//! **No transcript is ever logged**, on any path. Record 0005 set that rule for
//! a person's custom words; these are the words they actually said, so it holds
//! here more strongly and not less. A failure logs its machine reason and never
//! a row.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

/// How many dictations one page holds (record 0007). More than a person
/// dictates in a day, so the first page is usually the whole answer, and few
/// enough that a page of long transcripts is still a page.
pub const PAGE_SIZE: usize = 50;

/// One dictation, as this screen shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dictation {
    pub id: i64,
    /// Exactly the phrases that were typed, in order, with the joining spaces
    /// that were typed between them (record 0002). So what this screen shows
    /// and what is in the person's document cannot differ.
    pub text: String,
    /// UTC, as record 0002 stored it. What a person reads is the screen's, in
    /// the machine's own locale and time zone.
    pub started_at: String,
    pub duration_ms: i64,
    /// The code the dictation was **asked** with, never what Deepgram detected
    /// (record 0006).
    pub language: String,
}

/// Where the next page starts: the start time and identifier of the last row
/// already shown. A cursor and not an offset, so a dictation finished while a
/// person is reading cannot make a row appear twice (record 0007).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub started_at: String,
    pub id: i64,
}

/// One page of history, and whether there is more of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub rows: Vec<Dictation>,
    /// Whether there are older dictations after this page. Answered by asking
    /// for one row more than the page holds and seeing whether it was there,
    /// so it can never disagree with the rows beside it.
    pub has_older: bool,
}

/// Owns this feature's connection to the shared database file.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open the shared database file at `path`. **Nothing is created here.**
    /// The dictate feature has already created `dictation` by the time this
    /// runs, which is what the init order in `lib.rs` is for.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// One page of this account's dictations, newest first, optionally narrowed
    /// to those whose text contains `query` (record 0007 AC-2, AC-5, AC-7,
    /// AC-9).
    ///
    /// `query` is hostile input: it is text a person typed, headed for a `LIKE`
    /// in a real query. It is bound as a parameter, never joined into the SQL,
    /// and its wildcards are escaped by [`like_pattern`] so a search for `100%`
    /// searches for `100%`.
    ///
    /// The order is `started_at DESC, id DESC`, which is total, so the cursor
    /// below cannot skip or repeat a row. `dictation_by_account_recent` is the
    /// index record 0002 created for exactly this read.
    pub fn page(
        &self,
        account_id: &str,
        query: Option<&str>,
        before: Option<&Cursor>,
    ) -> rusqlite::Result<Page> {
        // One more than the page holds, so `has_older` is an observation rather
        // than a second count that could disagree with the rows.
        let limit = PAGE_SIZE as i64 + 1;

        let mut sql = String::from(
            "SELECT id, text, started_at, duration_ms, language
             FROM dictation
             WHERE account_id = ?1",
        );
        // Every fragment appended below is a literal in this file. Nothing a
        // person typed is ever part of the SQL text; the two values they can
        // influence are bound as ?2 and ?3.
        if query.is_some() {
            sql.push_str(" AND text LIKE ?2 ESCAPE '\\'");
        }
        if before.is_some() {
            let (started, id) = if query.is_some() {
                ("?3", "?4")
            } else {
                ("?2", "?3")
            };
            sql.push_str(&format!(
                " AND (started_at < {started} OR (started_at = {started} AND id < {id}))"
            ));
        }
        sql.push_str(" ORDER BY started_at DESC, id DESC LIMIT ");
        sql.push_str(&limit.to_string());

        let conn = self.conn.lock().expect("history store mutex poisoned");
        let mut stmt = conn.prepare(&sql)?;

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(account_id.to_string())];
        if let Some(q) = query {
            params.push(Box::new(like_pattern(q)));
        }
        if let Some(cursor) = before {
            params.push(Box::new(cursor.started_at.clone()));
            params.push(Box::new(cursor.id));
        }
        let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(refs.as_slice(), |row| {
                Ok(Dictation {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    started_at: row.get(2)?,
                    duration_ms: row.get(3)?,
                    language: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let has_older = rows.len() > PAGE_SIZE;
        Ok(Page {
            rows: rows.into_iter().take(PAGE_SIZE).collect(),
            has_older,
        })
    }

    /// How many of this account's dictations match `query`, or how many there
    /// are when there is none (record 0007 AC-5).
    ///
    /// The same `WHERE` as [`Store::page`], deliberately, so the number beside
    /// the search and the rows below it can never come from two different
    /// questions.
    pub fn count(&self, account_id: &str, query: Option<&str>) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().expect("history store mutex poisoned");
        match query {
            None => conn.query_row(
                "SELECT COUNT(*) FROM dictation WHERE account_id = ?1",
                [account_id],
                |row| row.get(0),
            ),
            Some(q) => conn.query_row(
                "SELECT COUNT(*) FROM dictation
                 WHERE account_id = ?1 AND text LIKE ?2 ESCAPE '\\'",
                rusqlite::params![account_id, like_pattern(q)],
                |row| row.get(0),
            ),
        }
    }
}

/// Turn what a person typed into a `LIKE` pattern that matches it literally.
///
/// `%` and `_` are wildcards in `LIKE` and `\` is the escape character this
/// project's queries declare, so all three are escaped and the whole thing is
/// wrapped in `%` for a contains match. Without this, a search for `100%`
/// would match every dictation and a search for `_` would match all of them
/// too, which is a wrong answer rather than a security hole; the security half
/// is that the result is still **bound as a parameter** and never joined into
/// the SQL.
///
/// The one honest limitation, named in record 0007: SQLite's `LIKE` ignores
/// case for the ASCII letters only, so a search in Greek or Cyrillic matches
/// the case it was typed in. Most scripts have no case at all. Fixing it needs
/// SQLite built with ICU, which is a new dependency and the user's decision.
fn like_pattern(query: &str) -> String {
    let mut out = String::with_capacity(query.len() + 2);
    out.push('%');
    for ch in query.chars() {
        if ch == '%' || ch == '_' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('%');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An in-memory database carrying the dictate feature's `dictation` table.
    ///
    /// **It lives inside the test module and not beside `Store::open`**, where
    /// it would read more naturally, and that placement is deliberate. Two
    /// guards at the bottom of this file read the file's own source up to the
    /// first `#[cfg(test)]`, and a test-only constructor above that line would
    /// put a `CREATE TABLE` inside the half they check, which is exactly the
    /// thing one of them exists to forbid. Keeping every test-only line below
    /// one boundary is what makes both guards mean what they say.
    ///
    /// The table's shape is copied from `dictate/store.rs` rather than shared,
    /// because sharing it would make this feature a creator of that table,
    /// which is the one thing it must never be.
    fn open_in_memory() -> Store {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);
             INSERT INTO account (id) VALUES ('acct_one'), ('acct_two');
             CREATE TABLE dictation (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id  TEXT NOT NULL REFERENCES account(id),
                text        TEXT NOT NULL,
                started_at  TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                language    TEXT NOT NULL DEFAULT 'en'
             );",
        )
        .unwrap();
        Store {
            conn: Mutex::new(conn),
        }
    }

    /// A real file on disk carrying the dictate feature's `dictation` table, so
    /// that closing and reopening can be tested the way a person experiences
    /// it: an in-memory database cannot be reopened, so nothing else would
    /// catch this feature failing to find rows that are actually there.
    fn a_fresh_database_file() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "echoscribe-history-test-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);
             INSERT INTO account (id) VALUES ('acct_one'), ('acct_two');
             CREATE TABLE dictation (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id  TEXT NOT NULL REFERENCES account(id),
                text        TEXT NOT NULL,
                started_at  TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                language    TEXT NOT NULL DEFAULT 'en'
             );",
        )
        .unwrap();
        path
    }

    fn save(store: &Store, account: &str, text: &str, started_at: &str, language: &str) -> i64 {
        let conn = store.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO dictation (account_id, text, started_at, duration_ms, language)
             VALUES (?1, ?2, ?3, 1000, ?4)",
            rusqlite::params![account, text, started_at, language],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn texts(page: &Page) -> Vec<&str> {
        page.rows.iter().map(|d| d.text.as_str()).collect()
    }

    #[test]
    fn a_dictation_comes_back_whole() {
        // covers: AC-2, AC-3.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "Hello there.",
            "2026-09-01T10:00:00Z",
            "en",
        );
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].text, "Hello there.");
        assert_eq!(page.rows[0].started_at, "2026-09-01T10:00:00Z");
        assert_eq!(page.rows[0].duration_ms, 1000);
        assert_eq!(page.rows[0].language, "en");
        assert!(!page.has_older);
    }

    #[test]
    fn dictations_come_back_newest_first() {
        // covers: AC-2.
        let store = open_in_memory();
        save(&store, "acct_one", "first", "2026-09-01T10:00:00Z", "en");
        save(&store, "acct_one", "second", "2026-09-01T11:00:00Z", "en");
        save(&store, "acct_one", "third", "2026-09-01T09:00:00Z", "en");
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&page), vec!["second", "first", "third"]);
    }

    #[test]
    fn two_dictations_at_the_same_instant_still_have_one_order() {
        // covers: AC-7. The order has to be total or the cursor below can skip
        // a row. Identical start times are not hypothetical: `started_at` is a
        // second-resolution timestamp and two short dictations can share one.
        let store = open_in_memory();
        let older = save(&store, "acct_one", "a", "2026-09-01T10:00:00Z", "en");
        let newer = save(&store, "acct_one", "b", "2026-09-01T10:00:00Z", "en");
        assert!(newer > older);
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&page), vec!["b", "a"]);
    }

    #[test]
    fn one_accounts_history_is_invisible_to_another() {
        // covers: AC-9, and record 0002's AC-18.
        let store = open_in_memory();
        save(&store, "acct_one", "mine", "2026-09-01T10:00:00Z", "en");
        save(&store, "acct_two", "theirs", "2026-09-01T11:00:00Z", "en");

        let one = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&one), vec!["mine"]);
        assert_eq!(store.count("acct_one", None).unwrap(), 1);

        let two = store.page("acct_two", None, None).unwrap();
        assert_eq!(texts(&two), vec!["theirs"]);
        assert_eq!(store.count("acct_two", None).unwrap(), 1);
    }

    #[test]
    fn a_search_narrows_to_matching_text() {
        // covers: AC-5.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "the cat sat",
            "2026-09-01T10:00:00Z",
            "en",
        );
        save(
            &store,
            "acct_one",
            "a dog barked",
            "2026-09-01T11:00:00Z",
            "en",
        );
        let page = store.page("acct_one", Some("cat"), None).unwrap();
        assert_eq!(texts(&page), vec!["the cat sat"]);
        assert_eq!(store.count("acct_one", Some("cat")).unwrap(), 1);
    }

    #[test]
    fn a_search_ignores_case_for_ascii_letters() {
        // covers: AC-5. The documented behaviour of SQLite's LIKE, and the
        // limitation record 0007 names: this holds for ASCII and not beyond it.
        let store = open_in_memory();
        save(&store, "acct_one", "Fumnanya", "2026-09-01T10:00:00Z", "en");
        assert_eq!(store.count("acct_one", Some("fumnanya")).unwrap(), 1);
        assert_eq!(store.count("acct_one", Some("FUMNANYA")).unwrap(), 1);
    }

    #[test]
    fn a_search_that_matches_nothing_returns_nothing_and_changes_nothing() {
        // covers: AC-6.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "the cat sat",
            "2026-09-01T10:00:00Z",
            "en",
        );
        let page = store.page("acct_one", Some("giraffe"), None).unwrap();
        assert!(page.rows.is_empty());
        assert!(!page.has_older);
        assert_eq!(store.count("acct_one", Some("giraffe")).unwrap(), 0);
        // The history itself is untouched.
        assert_eq!(store.count("acct_one", None).unwrap(), 1);
    }

    #[test]
    fn a_search_never_matches_another_accounts_dictations() {
        // covers: AC-9. The account scope is in the same WHERE as the search,
        // so narrowing can never widen.
        let store = open_in_memory();
        save(
            &store,
            "acct_two",
            "the secret plan",
            "2026-09-01T10:00:00Z",
            "en",
        );
        assert_eq!(store.count("acct_one", Some("secret")).unwrap(), 0);
        assert!(store
            .page("acct_one", Some("secret"), None)
            .unwrap()
            .rows
            .is_empty());
    }

    #[test]
    fn a_percent_sign_in_a_search_is_searched_for_and_not_used_as_a_wildcard() {
        // covers: AC-5. Without the escaping in `like_pattern`, this query
        // would match every dictation this account has.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "up 100% this year",
            "2026-09-01T10:00:00Z",
            "en",
        );
        save(
            &store,
            "acct_one",
            "nothing like it",
            "2026-09-01T11:00:00Z",
            "en",
        );
        let page = store.page("acct_one", Some("100%"), None).unwrap();
        assert_eq!(texts(&page), vec!["up 100% this year"]);
        assert_eq!(store.count("acct_one", Some("%")).unwrap(), 1);
    }

    #[test]
    fn an_underscore_in_a_search_is_searched_for_and_not_used_as_a_wildcard() {
        // covers: AC-5. `_` matches any single character in LIKE, so an
        // unescaped one matches every row with a character in that position.
        //
        // The second row below is the whole test. It read "the filename"
        // first, which is one character shorter and so matched the pattern
        // neither way, and the test therefore still passed with the escaping
        // deliberately removed. Found on 2026-09-04 by breaking `like_pattern`
        // on purpose, which is the only thing that finds a test that cannot
        // fail.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "the file_name",
            "2026-09-01T10:00:00Z",
            "en",
        );
        save(
            &store,
            "acct_one",
            "the fileXname",
            "2026-09-01T11:00:00Z",
            "en",
        );
        let page = store.page("acct_one", Some("file_name"), None).unwrap();
        assert_eq!(texts(&page), vec!["the file_name"]);
        assert_eq!(store.count("acct_one", Some("file_name")).unwrap(), 1);
    }

    #[test]
    fn a_backslash_in_a_search_is_searched_for() {
        // covers: AC-5. The escape character itself has to be escaped, or a
        // trailing one makes the pattern invalid and the search errors.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "C:\\Users\\me",
            "2026-09-01T10:00:00Z",
            "en",
        );
        save(
            &store,
            "acct_one",
            "C:/Users/me",
            "2026-09-01T11:00:00Z",
            "en",
        );
        let page = store.page("acct_one", Some("\\Users"), None).unwrap();
        assert_eq!(texts(&page), vec!["C:\\Users\\me"]);
        // A lone backslash, which is the shape that would otherwise throw.
        assert_eq!(store.count("acct_one", Some("\\")).unwrap(), 1);
    }

    #[test]
    fn a_search_holding_sql_is_searched_for_and_never_run() {
        // covers: AC-5. The query is bound as a parameter, so this is text and
        // not an instruction. If it ever stopped being bound, the two rows
        // below would be gone and this would fail on the count.
        let store = open_in_memory();
        save(
            &store,
            "acct_one",
            "a quiet morning",
            "2026-09-01T10:00:00Z",
            "en",
        );
        save(
            &store,
            "acct_one",
            "I said '; DROP TABLE dictation; -- out loud",
            "2026-09-01T11:00:00Z",
            "en",
        );
        let page = store
            .page("acct_one", Some("'; DROP TABLE dictation; --"), None)
            .unwrap();
        assert_eq!(page.rows.len(), 1);
        assert_eq!(store.count("acct_one", None).unwrap(), 2);
    }

    #[test]
    fn a_page_holds_fifty_and_says_there_are_older_ones() {
        // covers: AC-7.
        let store = open_in_memory();
        for n in 0..60 {
            save(
                &store,
                "acct_one",
                &format!("number {n}"),
                &format!("2026-09-01T10:{n:02}:00Z"),
                "en",
            );
        }
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(page.rows.len(), PAGE_SIZE);
        assert!(page.has_older);
        assert_eq!(page.rows[0].text, "number 59");
        assert_eq!(store.count("acct_one", None).unwrap(), 60);
    }

    #[test]
    fn the_last_page_says_there_are_no_older_ones() {
        // covers: AC-7. Exactly one page's worth, which is the boundary the
        // "one more than the page" trick exists to get right.
        let store = open_in_memory();
        for n in 0..PAGE_SIZE {
            save(
                &store,
                "acct_one",
                &format!("number {n}"),
                &format!("2026-09-01T10:{n:02}:00Z"),
                "en",
            );
        }
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(page.rows.len(), PAGE_SIZE);
        assert!(!page.has_older);
    }

    #[test]
    fn the_next_page_carries_on_where_the_last_one_stopped() {
        // covers: AC-7. Every row exactly once, in order, across two pages.
        let store = open_in_memory();
        for n in 0..60 {
            save(
                &store,
                "acct_one",
                &format!("number {n}"),
                &format!("2026-09-01T10:{n:02}:00Z"),
                "en",
            );
        }
        let first = store.page("acct_one", None, None).unwrap();
        let last = first.rows.last().unwrap();
        let cursor = Cursor {
            started_at: last.started_at.clone(),
            id: last.id,
        };
        let second = store.page("acct_one", None, Some(&cursor)).unwrap();

        assert_eq!(second.rows.len(), 10);
        assert!(!second.has_older);
        assert_eq!(second.rows[0].text, "number 9");

        let mut all: Vec<&str> = texts(&first);
        all.extend(texts(&second));
        assert_eq!(all.len(), 60);
        let unique: std::collections::HashSet<&&str> = all.iter().collect();
        assert_eq!(unique.len(), 60, "a row appeared on both pages");
    }

    #[test]
    fn a_dictation_finished_while_reading_does_not_repeat_a_row() {
        // covers: AC-7. This is the whole reason the cursor is not an offset.
        // A new row lands at the top between the two reads; with an offset the
        // second page would start one row too early and repeat one.
        let store = open_in_memory();
        for n in 0..60 {
            save(
                &store,
                "acct_one",
                &format!("number {n}"),
                &format!("2026-09-01T10:{n:02}:00Z"),
                "en",
            );
        }
        let first = store.page("acct_one", None, None).unwrap();
        save(
            &store,
            "acct_one",
            "brand new",
            "2026-09-01T12:00:00Z",
            "en",
        );

        let last = first.rows.last().unwrap();
        let cursor = Cursor {
            started_at: last.started_at.clone(),
            id: last.id,
        };
        let second = store.page("acct_one", None, Some(&cursor)).unwrap();

        let mut all: Vec<&str> = texts(&first);
        all.extend(texts(&second));
        let unique: std::collections::HashSet<&&str> = all.iter().collect();
        assert_eq!(unique.len(), all.len(), "a row appeared on both pages");
        assert!(!all.contains(&"brand new"));
    }

    #[test]
    fn paging_keeps_the_search() {
        // covers: AC-7. The cursor and the query travel together, so pressing
        // for older rows cannot widen the search.
        let store = open_in_memory();
        for n in 0..60 {
            let text = if n % 2 == 0 {
                format!("even {n} meeting")
            } else {
                format!("odd {n}")
            };
            save(
                &store,
                "acct_one",
                &text,
                &format!("2026-09-01T10:{n:02}:00Z"),
                "en",
            );
        }
        let first = store.page("acct_one", Some("meeting"), None).unwrap();
        assert_eq!(first.rows.len(), 30);
        assert!(!first.has_older);
        assert!(first.rows.iter().all(|d| d.text.contains("meeting")));
    }

    #[test]
    fn a_cursor_from_another_account_still_shows_only_your_own() {
        // covers: AC-9. The cursor narrows; the account scope is what limits.
        let store = open_in_memory();
        save(&store, "acct_two", "theirs", "2026-09-01T23:00:00Z", "en");
        save(&store, "acct_one", "mine", "2026-09-01T10:00:00Z", "en");
        let cursor = Cursor {
            started_at: "2026-09-01T22:00:00Z".to_string(),
            id: 9999,
        };
        let page = store.page("acct_one", None, Some(&cursor)).unwrap();
        assert_eq!(texts(&page), vec!["mine"]);
    }

    #[test]
    fn a_dictation_in_any_script_comes_back_exactly_as_it_was_stored() {
        // covers: AC-11. Nothing here normalises, folds or reorders text.
        let store = open_in_memory();
        let arabic = "مرحبا بالعالم";
        let chinese = "这是一个测试";
        save(&store, "acct_one", arabic, "2026-09-01T10:00:00Z", "ar");
        save(&store, "acct_one", chinese, "2026-09-01T11:00:00Z", "zh");
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&page), vec![chinese, arabic]);
        assert_eq!(page.rows[0].language, "zh");
        assert_eq!(page.rows[1].language, "ar");
    }

    #[test]
    fn a_dictation_is_still_there_after_closing_and_reopening_the_app() {
        // covers: AC-3, which is plan row 5's own `Done when:` line. This is
        // the one test that goes through a real file rather than memory,
        // because an in-memory database cannot be reopened and nothing else
        // would catch this feature failing to find rows that are there.
        let path = a_fresh_database_file();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute(
                "INSERT INTO dictation (account_id, text, started_at, duration_ms, language)
                 VALUES ('acct_one', 'said before', '2026-09-02T10:00:00Z', 3500, 'en')",
                [],
            )
            .unwrap();
        }

        {
            let store = Store::open(&path).expect("first open");
            let page = store.page("acct_one", None, None).unwrap();
            assert_eq!(texts(&page), vec!["said before"]);
        }

        // The app closes and starts again. Nothing was written by this feature
        // in between, and nothing needs to have been.
        let store = Store::open(&path).expect("second open");
        let page = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&page), vec!["said before"]);
        assert_eq!(page.rows[0].duration_ms, 3500);
        assert_eq!(store.count("acct_one", None).unwrap(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn opening_the_file_creates_nothing() {
        // covers: AC-3, and record 0007's data model. `Store::open` on a file
        // whose `dictation` table is missing must not conjure one: the dictate
        // feature owns that schema, and a table created here would be a second
        // creator with no migration behind it. The failure lands on the read,
        // which is where the read failure state already goes.
        let path = std::env::temp_dir().join(format!(
            "echoscribe-history-empty-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = Store::open(&path).expect("opening a bare file is not an error");
        assert!(
            store.page("acct_one", None, None).is_err(),
            "the history store created `dictation` itself. It reads that table \
             and the dictate feature creates it"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_dictation_finished_just_now_is_there_the_next_time_history_is_read() {
        // covers: AC-4. The list is what History held when it was opened and
        // does not refresh itself, so what this promises is that a fresh read
        // finds a row written after the last one. Nothing is cached here, which
        // is exactly what makes that true.
        let store = open_in_memory();
        save(&store, "acct_one", "before", "2026-09-01T10:00:00Z", "en");
        let first = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&first), vec!["before"]);

        save(&store, "acct_one", "just now", "2026-09-01T11:00:00Z", "en");
        let second = store.page("acct_one", None, None).unwrap();
        assert_eq!(texts(&second), vec!["just now", "before"]);
        assert_eq!(store.count("acct_one", None).unwrap(), 2);
    }

    #[test]
    fn an_empty_history_is_no_rows_rather_than_a_failure() {
        // covers: AC-8, AC-10. An empty history and an unreadable one are
        // different things and must never look alike: this is the one the
        // screen answers with its empty state, and it comes back as success.
        let store = open_in_memory();
        let page = store.page("acct_one", None, None).unwrap();
        assert!(page.rows.is_empty());
        assert!(!page.has_older);
        assert_eq!(store.count("acct_one", None).unwrap(), 0);
    }

    /// This file's own source, minus its tests.
    fn this_file_without_tests() -> &'static str {
        include_str!("store.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the file has a non-test half")
    }

    #[test]
    fn this_feature_never_writes_to_the_dictation_table() {
        // covers: AC-12, and record 0007's data model. `dictation` is the
        // dictate feature's, and this one is a reader. A guard rather than a
        // convention, because the temptation is a delete: record 0002 says
        // history is kept "until the person deletes it", and the route for that
        // is not designed. If it ever is, it is a decision and an amendment,
        // and this test is what makes writing one impossible to do by accident.
        for forbidden in [
            "CREATE TABLE",
            "INSERT INTO",
            "UPDATE ",
            "DELETE FROM",
            "ALTER TABLE",
            "DROP ",
        ] {
            assert!(
                !this_file_without_tests().contains(forbidden),
                "the history store's own code contains {forbidden}. This \
                 feature reads `dictation` and never writes it: the dictate \
                 feature owns that table"
            );
        }
    }

    #[test]
    fn nothing_a_person_typed_is_ever_part_of_the_sql() {
        // covers: AC-5, and record 0007's Risk. The query is bound, always.
        // `format!` appears once in the non-test half, building the cursor
        // clause out of parameter placeholders, and this guard is what stops a
        // later edit reaching for it with a value instead.
        let body = this_file_without_tests();
        for placeholder in ["{started}", "{id}"] {
            assert!(
                body.contains(placeholder),
                "the cursor clause no longer uses the {placeholder} \
                 placeholder. If it now interpolates a value, that value is a \
                 person's search or a row identifier going into SQL text"
            );
        }
        assert!(
            !body.contains("{query}") && !body.contains("{account"),
            "a search or an account id is being formatted into the SQL text. \
             Both are bound as parameters and neither may be joined in"
        );
    }
}
