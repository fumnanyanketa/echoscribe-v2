//! Local storage for the dictate feature: record 0002's three tables, and the
//! narrow reads and writes milestone 1 needs (the pill's remembered position
//! and whether the two sounds are on).
//!
//! Everything EchoScribe persists lives in one SQLite file on the person's own
//! machine (AGENTS.md data rules). The sign-in feature opens that file first and
//! creates `account` and `session`; this module opens its own connection to the
//! same file and owns record 0002's schema:
//!
//!   * `dictation` - one finished dictation. Filled from milestone 5.
//!   * `dictation_setting` - one row per account: hotkey, sounds, pill spot.
//!   * `deepgram_credential` - the masked key and its vault entry name, from
//!     milestone 3.
//!
//! Record 0002's data model calls for "one migration, creating all three
//! tables", so all three are created together here even though milestone 1 only
//! touches `dictation_setting`. No audio, and no partial transcript, is ever
//! written to any of them.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

/// The pill's default spot: horizontally centred (0.5) and on the bottom edge
/// (1.0) of the screen's working area. Record 0002 AC-23.
pub const DEFAULT_PILL_X: f64 = 0.5;
pub const DEFAULT_PILL_Y: f64 = 1.0;

/// One row's worth of the settings this feature keeps for an account.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DictationSetting {
    /// Fraction of the working area width where the pill's centre sits, 0..=1.
    pub pill_x: f64,
    /// Fraction of the working area height where the pill's centre sits, 0..=1.
    pub pill_y: f64,
    /// Whether the open and close sounds play. On for a new account.
    pub sounds_enabled: bool,
    /// The chosen hotkey, `"double_tap_ctrl"` or `"double_tap_alt"`. Milestone 1
    /// has no screen to change it, but the hook reads it so milestone 5 only
    /// adds the setter.
    pub hotkey: Hotkey,
}

impl Default for DictationSetting {
    fn default() -> Self {
        Self {
            pill_x: DEFAULT_PILL_X,
            pill_y: DEFAULT_PILL_Y,
            sounds_enabled: true,
            hotkey: Hotkey::DoubleTapCtrl,
        }
    }
}

/// The two hotkeys record 0002 allows, and nothing else. A stored value that is
/// neither is read back as the default, never acted on as-is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hotkey {
    DoubleTapCtrl,
    DoubleTapAlt,
}

impl Hotkey {
    fn from_stored(value: &str) -> Self {
        match value {
            "double_tap_alt" => Hotkey::DoubleTapAlt,
            _ => Hotkey::DoubleTapCtrl,
        }
    }
}

/// Owns this feature's connection to the shared database file.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open the shared database file at `path` and make sure record 0002's
    /// tables exist. The sign-in feature has already created `account` and
    /// `session` by the time this runs.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Self::prepare(conn)
    }

    /// An in-memory database with a stand-in `account` table, for tests.
    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);
             INSERT INTO account (id) VALUES ('acct_test');",
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

    /// Record 0002's one migration. Idempotent: every statement is
    /// `IF NOT EXISTS`, so opening an already-migrated file is a no-op.
    fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS dictation (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id  TEXT NOT NULL REFERENCES account(id),
                text        TEXT NOT NULL,
                started_at  TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            );

            -- Plan row 5 only ever reads this newest-first for one account.
            CREATE INDEX IF NOT EXISTS dictation_by_account_recent
                ON dictation (account_id, started_at DESC);

            CREATE TABLE IF NOT EXISTS dictation_setting (
                account_id     TEXT PRIMARY KEY NOT NULL REFERENCES account(id),
                hotkey         TEXT NOT NULL DEFAULT 'double_tap_ctrl'
                               CHECK (hotkey IN ('double_tap_ctrl', 'double_tap_alt')),
                sounds_enabled INTEGER NOT NULL DEFAULT 1,
                pill_x         REAL NOT NULL DEFAULT 0.5,
                pill_y         REAL NOT NULL DEFAULT 1.0,
                updated_at     TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS deepgram_credential (
                account_id        TEXT PRIMARY KEY NOT NULL REFERENCES account(id),
                key_last_four     TEXT NOT NULL,
                credential_target TEXT NOT NULL,
                saved_at          TEXT NOT NULL,
                last_validated_at TEXT
            );
            ",
        )
    }

    /// The names of record 0002's tables, for tests and diagnostics.
    #[cfg(test)]
    pub fn table_names(&self) -> rusqlite::Result<Vec<String>> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        let mut names: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table'
                 AND name IN ('dictation', 'dictation_setting', 'deepgram_credential')",
            )?
            .query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        names.sort();
        Ok(names)
    }

    /// The settings for `account_id`. A missing row, or any stored value out of
    /// range, comes back as the default rather than as an instruction: the pill
    /// fractions are clamped to 0..=1 and an unknown hotkey falls back to Ctrl
    /// (record 0002 data rules).
    pub fn setting_for(&self, account_id: &str) -> rusqlite::Result<DictationSetting> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.query_row(
            "SELECT pill_x, pill_y, sounds_enabled, hotkey
             FROM dictation_setting WHERE account_id = ?1",
            [account_id],
            |row| {
                let pill_x: f64 = row.get(0)?;
                let pill_y: f64 = row.get(1)?;
                let sounds_enabled: i64 = row.get(2)?;
                let hotkey: String = row.get(3)?;
                Ok(DictationSetting {
                    pill_x: clamp_fraction(pill_x, DEFAULT_PILL_X),
                    pill_y: clamp_fraction(pill_y, DEFAULT_PILL_Y),
                    sounds_enabled: sounds_enabled != 0,
                    hotkey: Hotkey::from_stored(&hotkey),
                })
            },
        )
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(DictationSetting::default()),
            other => Err(other),
        })
    }

    /// Remember where the pill was dragged to, as fractions of the working area.
    /// Values are clamped to 0..=1 before they are written. Leaves the hotkey
    /// and sound choice untouched if a row is already there; a fresh row takes
    /// the table defaults for those.
    pub fn save_pill_spot(
        &self,
        account_id: &str,
        pill_x: f64,
        pill_y: f64,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let x = clamp_fraction(pill_x, DEFAULT_PILL_X);
        let y = clamp_fraction(pill_y, DEFAULT_PILL_Y);
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO dictation_setting (account_id, pill_x, pill_y, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(account_id) DO UPDATE SET
                 pill_x = excluded.pill_x,
                 pill_y = excluded.pill_y,
                 updated_at = excluded.updated_at",
            rusqlite::params![account_id, x, y, now_utc],
        )?;
        Ok(())
    }
}

/// Keep a fraction inside 0..=1. A NaN or out-of-range value becomes `fallback`,
/// so a corrupt row can never push the pill off screen.
fn clamp_fraction(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_one_migration_creates_all_three_tables() {
        let store = Store::open_in_memory().expect("open");
        assert_eq!(
            store.table_names().unwrap(),
            vec![
                "deepgram_credential".to_string(),
                "dictation".to_string(),
                "dictation_setting".to_string(),
            ]
        );
    }

    #[test]
    fn opening_twice_is_a_no_op() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "echoscribe-dictate-test-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch("CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);")
                .unwrap();
        }
        Store::open(&path).expect("first open");
        Store::open(&path).expect("second open");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_missing_row_reads_back_as_the_default_spot() {
        let store = Store::open_in_memory().unwrap();
        let setting = store.setting_for("acct_test").unwrap();
        assert_eq!(setting, DictationSetting::default());
        assert_eq!(setting.pill_x, 0.5);
        assert_eq!(setting.pill_y, 1.0);
        assert!(setting.sounds_enabled);
        assert_eq!(setting.hotkey, Hotkey::DoubleTapCtrl);
    }

    #[test]
    fn a_dragged_spot_round_trips() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_pill_spot("acct_test", 0.25, 0.8, "2026-08-29T10:00:00Z")
            .unwrap();
        let setting = store.setting_for("acct_test").unwrap();
        assert_eq!(setting.pill_x, 0.25);
        assert_eq!(setting.pill_y, 0.8);

        // A second drag overwrites, and leaves sounds and hotkey alone.
        store
            .save_pill_spot("acct_test", 0.9, 0.1, "2026-08-29T11:00:00Z")
            .unwrap();
        let setting = store.setting_for("acct_test").unwrap();
        assert_eq!(setting.pill_x, 0.9);
        assert_eq!(setting.pill_y, 0.1);
        assert!(setting.sounds_enabled);
    }

    #[test]
    fn out_of_range_values_are_clamped_on_write_and_read() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_pill_spot("acct_test", 5.0, -2.0, "2026-08-29T10:00:00Z")
            .unwrap();
        let setting = store.setting_for("acct_test").unwrap();
        // Off-range on write falls back to the default for that axis.
        assert_eq!(setting.pill_x, DEFAULT_PILL_X);
        assert_eq!(setting.pill_y, DEFAULT_PILL_Y);
    }

    #[test]
    fn a_corrupt_stored_fraction_reads_as_the_default() {
        let store = Store::open_in_memory().unwrap();
        {
            let conn = store.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO dictation_setting (account_id, pill_x, pill_y, updated_at)
                 VALUES ('acct_test', 9.9, 0.5, '2026-08-29T10:00:00Z')",
                [],
            )
            .unwrap();
        }
        let setting = store.setting_for("acct_test").unwrap();
        assert_eq!(setting.pill_x, DEFAULT_PILL_X);
        assert_eq!(setting.pill_y, 0.5);
    }

    #[test]
    fn an_unknown_stored_hotkey_reads_as_ctrl() {
        assert_eq!(Hotkey::from_stored("double_tap_alt"), Hotkey::DoubleTapAlt);
        assert_eq!(
            Hotkey::from_stored("double_tap_ctrl"),
            Hotkey::DoubleTapCtrl
        );
        assert_eq!(Hotkey::from_stored("f13"), Hotkey::DoubleTapCtrl);
        assert_eq!(Hotkey::from_stored(""), Hotkey::DoubleTapCtrl);
    }
}
