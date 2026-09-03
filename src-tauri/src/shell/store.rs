//! The shell's own connection to the shared database, and the one table this
//! feature owns: `shell_window`.
//!
//! Everything EchoScribe persists lives in one SQLite file on the person's own
//! machine (AGENTS.md data rules). The sign-in feature opens that file first
//! and creates `account` and `session`; the dictate feature opens its own
//! connection beside it; this module does the same and owns record 0004's one
//! table.
//!
//! `shell_window` holds a window's size and place for one account, and nothing
//! else ever goes in it (record 0004, Risk). Four numbers and a screen name. No
//! transcript, no setting, no secret.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

/// Where the window's centre sits by default: the middle of the working area.
/// Record 0004's data model.
pub const DEFAULT_CENTER_X: f64 = 0.5;
pub const DEFAULT_CENTER_Y: f64 = 0.5;

/// What was remembered for an account, read back and made safe to act on.
/// A missing row is `None`, which means a first ever open.
#[derive(Debug, Clone, PartialEq)]
pub struct RememberedWindow {
    /// Logical pixels, already floored to the 960x640 minimum. The cap against
    /// the chosen screen's working area is `geometry`'s, because only it knows
    /// which screen that is.
    pub width: f64,
    pub height: f64,
    /// Fractions in 0..=1 locating the window's centre within a screen's
    /// working area. Read as nonsense rather than as instructions: anything
    /// outside that range comes back as the default, exactly as `pill_x` and
    /// `pill_y` do in record 0002.
    pub center_x: f64,
    pub center_y: f64,
    /// The name Windows gave the screen it was left on, as a best effort. May
    /// be empty, and a name matching no screen is not an error.
    pub screen_name: String,
}

/// Owns this feature's connection to the shared database file.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open the shared database file at `path` and make sure record 0004's one
    /// table exists. The sign-in feature has already created `account`, and the
    /// dictate feature `deepgram_credential`, by the time this runs.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Self::prepare(conn)
    }

    /// An in-memory database with stand-in `account` and `deepgram_credential`
    /// tables, for tests.
    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);
             INSERT INTO account (id) VALUES ('acct_test');
             CREATE TABLE deepgram_credential (
                 account_id TEXT PRIMARY KEY NOT NULL REFERENCES account(id)
             );",
        )?;
        Self::prepare(conn)
    }

    fn prepare(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::create_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Record 0004's one migration. Idempotent, like record 0002's: opening an
    /// already-migrated file is a no-op.
    ///
    /// The CHECK on the two fractions is the second guard, not the first: every
    /// write goes through [`Store::save`], which clamps. It is here so a row
    /// this app did not write cannot become an instruction either.
    fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS shell_window (
                account_id  TEXT PRIMARY KEY NOT NULL REFERENCES account(id),
                width       INTEGER NOT NULL,
                height      INTEGER NOT NULL,
                center_x    REAL NOT NULL DEFAULT 0.5
                            CHECK (center_x >= 0.0 AND center_x <= 1.0),
                center_y    REAL NOT NULL DEFAULT 0.5
                            CHECK (center_y >= 0.0 AND center_y <= 1.0),
                screen_name TEXT NOT NULL DEFAULT '',
                updated_at  TEXT NOT NULL
            );
            ",
        )
    }

    /// The name of record 0004's table, for tests and diagnostics.
    #[cfg(test)]
    pub fn table_names(&self) -> rusqlite::Result<Vec<String>> {
        let conn = self.conn.lock().expect("shell store mutex poisoned");
        let mut statement = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table'
             AND name = 'shell_window'",
        )?;
        let names = statement.query_map([], |r| r.get(0))?.collect();
        names
    }

    /// What was remembered for `account_id`, or `None` for a first ever open.
    ///
    /// Nothing here is trusted as written. A size below the floor comes back at
    /// the floor, a size that is zero or negative comes back as `None` so the
    /// window opens at its first-open size, and a fraction outside 0..=1 comes
    /// back as the default. A row holding a size that cannot be shown produces
    /// a usable window, never an unusable one (record 0004 data rules).
    pub fn read(
        &self,
        account_id: &str,
        floor_w: f64,
        floor_h: f64,
    ) -> rusqlite::Result<Option<RememberedWindow>> {
        let conn = self.conn.lock().expect("shell store mutex poisoned");
        let row = conn
            .query_row(
                "SELECT width, height, center_x, center_y, screen_name
                 FROM shell_window WHERE account_id = ?1",
                [account_id],
                |row| {
                    let width: i64 = row.get(0)?;
                    let height: i64 = row.get(1)?;
                    let center_x: f64 = row.get(2)?;
                    let center_y: f64 = row.get(3)?;
                    let screen_name: String = row.get(4)?;
                    Ok((width, height, center_x, center_y, screen_name))
                },
            )
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok((0, 0, 0.0, 0.0, String::new())),
                other => Err(other),
            })?;

        let (width, height, center_x, center_y, screen_name) = row;
        if width <= 0 || height <= 0 {
            // No row, or a row whose size means nothing. Either way this is a
            // first open.
            return Ok(None);
        }
        Ok(Some(RememberedWindow {
            width: (width as f64).max(floor_w),
            height: (height as f64).max(floor_h),
            center_x: clamp_fraction(center_x, DEFAULT_CENTER_X),
            center_y: clamp_fraction(center_y, DEFAULT_CENTER_Y),
            screen_name,
        }))
    }

    /// Remember where and how big the window was left. Called when a move or a
    /// resize has finished, never on every pixel of a drag (record 0004 data
    /// rules). Fractions are clamped before they are written, so the column
    /// CHECK never has to refuse one of our own writes.
    pub fn save(
        &self,
        account_id: &str,
        window: &RememberedWindow,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        // Nothing is written without knowing whose it is (AGENTS.md data rules).
        if account_id.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().expect("shell store mutex poisoned");
        conn.execute(
            "INSERT INTO shell_window
                 (account_id, width, height, center_x, center_y, screen_name, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(account_id) DO UPDATE SET
                 width = excluded.width,
                 height = excluded.height,
                 center_x = excluded.center_x,
                 center_y = excluded.center_y,
                 screen_name = excluded.screen_name,
                 updated_at = excluded.updated_at",
            rusqlite::params![
                account_id,
                window.width.round().max(1.0) as i64,
                window.height.round().max(1.0) as i64,
                clamp_fraction(window.center_x, DEFAULT_CENTER_X),
                clamp_fraction(window.center_y, DEFAULT_CENTER_Y),
                window.screen_name,
                now_utc
            ],
        )?;
        Ok(())
    }

    /// Whether a Deepgram key is saved for `account_id`.
    ///
    /// `deepgram_credential` is record 0002's table and stays record 0002's:
    /// this only ever counts rows in it and never reads a column, so nothing
    /// about the key, not even its last four characters, passes through here.
    /// Record 0004's value sourcing names this row as the source for whether
    /// the dashboard may be shown at all, and it is read through this feature's
    /// own connection rather than by reaching into the dictate feature, because
    /// AGENTS.md forbids one feature folder importing another.
    pub fn deepgram_key_exists(&self, account_id: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("shell store mutex poisoned");
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM deepgram_credential WHERE account_id = ?1",
            [account_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

/// A stored fraction, made safe. Anything outside 0..=1, and anything that is
/// not a number at all, is the default rather than an instruction.
fn clamp_fraction(value: f64, default: f64) -> f64 {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        value
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOOR_W: f64 = 960.0;
    const FLOOR_H: f64 = 640.0;

    fn left_at(
        width: f64,
        height: f64,
        center_x: f64,
        center_y: f64,
        screen_name: &str,
    ) -> RememberedWindow {
        RememberedWindow {
            width,
            height,
            center_x,
            center_y,
            screen_name: screen_name.to_string(),
        }
    }

    #[test]
    fn the_migration_creates_the_one_table() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.table_names().unwrap(), vec!["shell_window"]);
    }

    #[test]
    fn a_missing_row_is_a_first_open() {
        // covers: AC-3. Nothing remembered, so nothing to place from.
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.read("acct_test", FLOOR_W, FLOOR_H).unwrap(), None);
    }

    #[test]
    fn a_size_and_place_round_trip() {
        // covers: AC-4.
        let store = Store::open_in_memory().unwrap();
        store
            .save(
                "acct_test",
                &left_at(1400.0, 900.0, 0.25, 0.75, r"\\.\DISPLAY2"),
                "2026-09-03T10:00:00Z",
            )
            .unwrap();
        let back = store.read("acct_test", FLOOR_W, FLOOR_H).unwrap().unwrap();
        assert_eq!(back.width, 1400.0);
        assert_eq!(back.height, 900.0);
        assert_eq!(back.center_x, 0.25);
        assert_eq!(back.center_y, 0.75);
        assert_eq!(back.screen_name, r"\\.\DISPLAY2");
    }

    #[test]
    fn a_size_below_the_floor_comes_back_at_the_floor() {
        // covers: AC-3, AC-5. A row holding an unusable size produces a usable
        // window.
        let store = Store::open_in_memory().unwrap();
        store
            .save(
                "acct_test",
                &left_at(400.0, 300.0, 0.5, 0.5, ""),
                "2026-09-03T10:00:00Z",
            )
            .unwrap();
        let back = store.read("acct_test", FLOOR_W, FLOOR_H).unwrap().unwrap();
        assert_eq!(back.width, FLOOR_W);
        assert_eq!(back.height, FLOOR_H);
    }

    #[test]
    fn a_fraction_outside_the_range_is_written_as_the_default() {
        // covers: AC-5. Clamped on the way in, so the column CHECK never has to
        // refuse one of our own writes.
        let store = Store::open_in_memory().unwrap();
        store
            .save(
                "acct_test",
                &left_at(1200.0, 800.0, 9.0, f64::NAN, ""),
                "2026-09-03T10:00:00Z",
            )
            .unwrap();
        let back = store.read("acct_test", FLOOR_W, FLOOR_H).unwrap().unwrap();
        assert_eq!(back.center_x, DEFAULT_CENTER_X);
        assert_eq!(back.center_y, DEFAULT_CENTER_Y);
    }

    #[test]
    fn nothing_is_written_without_an_account() {
        // covers: AGENTS.md data rules. Nothing is stored until we know whose
        // it is.
        let store = Store::open_in_memory().unwrap();
        store
            .save(
                "",
                &left_at(1200.0, 800.0, 0.5, 0.5, ""),
                "2026-09-03T10:00:00Z",
            )
            .unwrap();
        let conn = store.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM shell_window", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn a_second_account_never_sees_the_first_ones_place() {
        // covers: AC-7. The row is keyed by account, so an account with no row
        // gets the defaults and never the other account's window.
        let store = Store::open_in_memory().unwrap();
        {
            let conn = store.conn.lock().unwrap();
            conn.execute("INSERT INTO account (id) VALUES ('acct_two')", [])
                .unwrap();
        }
        store
            .save(
                "acct_test",
                &left_at(1400.0, 900.0, 0.1, 0.1, ""),
                "2026-09-03T10:00:00Z",
            )
            .unwrap();
        assert_eq!(store.read("acct_two", FLOOR_W, FLOOR_H).unwrap(), None);
    }

    #[test]
    fn the_key_check_counts_rows_and_reads_no_column() {
        // covers: AC-1, AC-7. The presence of the row is the whole answer.
        let store = Store::open_in_memory().unwrap();
        assert!(!store.deepgram_key_exists("acct_test").unwrap());
        {
            let conn = store.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO deepgram_credential (account_id) VALUES ('acct_test')",
                [],
            )
            .unwrap();
        }
        assert!(store.deepgram_key_exists("acct_test").unwrap());
    }
}
