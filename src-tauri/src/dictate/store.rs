//! Local storage for the dictate feature: record 0002's three tables and the
//! narrow reads and writes this feature needs. Reading history back is plan
//! row 5's own feature and is deliberately not here.
//!
//! Everything EchoScribe persists lives in one SQLite file on the person's own
//! machine (AGENTS.md data rules). The sign-in feature opens that file first and
//! creates `account` and `session`; this module opens its own connection to the
//! same file and owns record 0002's schema:
//!
//!   * `dictation` - one row per finished dictation that typed something.
//!     Written from milestone 5, read by plan row 5.
//!   * `dictation_setting` - one row per account: hotkey, sounds, pill spot.
//!   * `deepgram_credential` - the masked key and its vault entry name. Filled
//!     from milestone 3. The key itself is never here: it lives in Windows
//!     Credential Manager, behind `key_vault.rs`.
//!
//! Record 0002's data model calls for "one migration, creating all three
//! tables", so all three are created together here. No audio, and no partial
//! transcript, is ever written to any of them.

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
    /// The chosen hotkey, one of exactly two. The hook is armed from this on
    /// sign-in and re-armed from it the moment `set_hotkey` changes it, so
    /// AC-19's "works immediately, with no restart" needs nothing else.
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
    /// The two the interface may offer, in the order the record names them.
    /// Fixed here, not a setting and not extendable at runtime, which is what
    /// AC-22 asks for: there is nowhere else a third could come from.
    pub const CHOICES: [Hotkey; 2] = [Hotkey::DoubleTapCtrl, Hotkey::DoubleTapAlt];

    /// A stored value, read leniently. Anything that is neither of the two is
    /// the default, never an instruction to the hook (record 0002 data rules).
    fn from_stored(value: &str) -> Self {
        match value {
            "double_tap_alt" => Hotkey::DoubleTapAlt,
            _ => Hotkey::DoubleTapCtrl,
        }
    }

    /// A value the interface asked for, read strictly. `None` is `set_hotkey`'s
    /// one refusal (AC-22), and it is deliberately not the same reading as
    /// [`Hotkey::from_stored`]: a row we cannot understand falls back so the
    /// hotkey still works, but a write we cannot understand is refused outright
    /// rather than quietly turned into Ctrl.
    pub fn from_chosen(value: &str) -> Option<Self> {
        match value {
            "double_tap_ctrl" => Some(Hotkey::DoubleTapCtrl),
            "double_tap_alt" => Some(Hotkey::DoubleTapAlt),
            _ => None,
        }
    }

    /// How this hotkey is written down. The only two strings that ever reach
    /// the column, so the record's "refused on write if it is not one of the
    /// two named values" holds by construction and the table's CHECK is the
    /// second guard rather than the first.
    pub fn as_stored(self) -> &'static str {
        match self {
            Hotkey::DoubleTapCtrl => "double_tap_ctrl",
            Hotkey::DoubleTapAlt => "double_tap_alt",
        }
    }
}

/// What is kept about a saved Deepgram key. Never the key itself: that lives in
/// Windows Credential Manager under `credential_target`, and only Rust reads it
/// (record 0002 data model, AC-12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedKey {
    /// The last four characters, and the only part of the key ever stored here.
    pub key_last_four: String,
    /// The name of the Credential Manager entry holding the real key.
    pub credential_target: String,
    pub saved_at: String,
    /// When Deepgram last accepted it. Set on save; milestone 4 refreshes it
    /// from the live stream.
    pub last_validated_at: Option<String>,
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

    /// Remember the hotkey this account chose (record 0002 AC-19, AC-22).
    ///
    /// Takes a [`Hotkey`], not a string, so there is no path by which a third
    /// value reaches the column. Leaves the sound choice and the pill spot
    /// alone; a fresh row takes the table defaults for those.
    pub fn save_hotkey(
        &self,
        account_id: &str,
        hotkey: Hotkey,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO dictation_setting (account_id, hotkey, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id) DO UPDATE SET
                 hotkey = excluded.hotkey,
                 updated_at = excluded.updated_at",
            rusqlite::params![account_id, hotkey.as_stored(), now_utc],
        )?;
        Ok(())
    }

    /// Remember whether the opening and closing sounds play (record 0002
    /// AC-21). Leaves the hotkey and the pill spot alone.
    pub fn save_sounds_enabled(
        &self,
        account_id: &str,
        enabled: bool,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO dictation_setting (account_id, sounds_enabled, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id) DO UPDATE SET
                 sounds_enabled = excluded.sounds_enabled,
                 updated_at = excluded.updated_at",
            rusqlite::params![account_id, enabled as i64, now_utc],
        )?;
        Ok(())
    }

    /// Keep a finished dictation against the account that spoke it (record 0002
    /// AC-17, AC-18).
    ///
    /// **A dictation that typed nothing is not saved, and that is the whole of
    /// what AC-17's "completed" means.** Settled with the user on 2026-09-02
    /// under `/develop`'s gate: a row is written when, and only when, at least
    /// one finalised phrase reached the cursor, whatever ended the dictation.
    /// It follows the data rule in AGENTS.md that transcribed text goes to two
    /// places only, the cursor it was dictated into and the local history, so
    /// text that never reached a cursor has no history to be in. That makes a
    /// silent dictation and a password refusal save nothing, and it makes a
    /// dictation cut short by a dropped connection save the words it did type,
    /// because those words are sitting in the person's document.
    ///
    /// The rule lives here rather than in the caller so that no caller can
    /// write a blank row, however dictation came to end.
    ///
    /// `text` is the finalised phrases exactly as they were typed, joining
    /// spaces included. Nothing unfinished ever reaches here: AC-33 keeps
    /// interim wording on the pill and out of every table.
    pub fn save_dictation(
        &self,
        account_id: &str,
        text: &str,
        started_at: &str,
        duration_ms: i64,
    ) -> rusqlite::Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO dictation (account_id, text, started_at, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![account_id, text, started_at, duration_ms],
        )?;
        Ok(())
    }

    /// Whether this account has a Deepgram key saved at all (record 0002 AC-9).
    ///
    /// The presence of the row is the whole answer, which is what the record's
    /// value sourcing names. Nothing here reads the credential store or asks
    /// Deepgram anything: the hotkey path runs through this on every press and
    /// must stay cheap.
    pub fn deepgram_key_for(&self, account_id: &str) -> rusqlite::Result<Option<SavedKey>> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.query_row(
            "SELECT key_last_four, credential_target, saved_at, last_validated_at
             FROM deepgram_credential WHERE account_id = ?1",
            [account_id],
            |row| {
                Ok(SavedKey {
                    key_last_four: row.get(0)?,
                    credential_target: row.get(1)?,
                    saved_at: row.get(2)?,
                    last_validated_at: row.get(3)?,
                })
            },
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
    }

    /// Record that Deepgram accepted a key for this account.
    ///
    /// Only ever called after Deepgram has accepted it and after the key itself
    /// has reached the credential store, so a row here always means there is a
    /// key to find. Replacing a key overwrites the row rather than adding one:
    /// an account has zero or one, per the record's data model.
    ///
    /// `key_last_four` is the only part of the key this takes, and the caller
    /// has no way to pass more: the full key is not a parameter.
    pub fn save_deepgram_key(
        &self,
        account_id: &str,
        key_last_four: &str,
        credential_target: &str,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO deepgram_credential
                 (account_id, key_last_four, credential_target, saved_at, last_validated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(account_id) DO UPDATE SET
                 key_last_four     = excluded.key_last_four,
                 credential_target = excluded.credential_target,
                 saved_at          = excluded.saved_at,
                 last_validated_at = excluded.last_validated_at",
            rusqlite::params![account_id, key_last_four, credential_target, now_utc],
        )?;
        Ok(())
    }

    /// Forget this account's key. Succeeds when there was nothing to forget.
    /// The credential entry is removed by the caller; this only drops the row.
    pub fn clear_deepgram_key(&self, account_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "DELETE FROM deepgram_credential WHERE account_id = ?1",
            [account_id],
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

    /// A fresh database file holding the `account` table and row the sign-in
    /// feature creates before this module ever opens it.
    fn a_fresh_database_file() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "echoscribe-dictate-test-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);
             INSERT INTO account (id) VALUES ('acct_test');",
        )
        .unwrap();
        path
    }

    /// Every dictation this store holds for one account, oldest first. Reading
    /// history back is plan row 5's feature, so this stays a test helper.
    fn dictations_of(store: &Store, account_id: &str) -> Vec<(String, String, i64)> {
        let conn = store.conn.lock().unwrap();
        let rows = conn
            .prepare(
                "SELECT text, started_at, duration_ms FROM dictation
                 WHERE account_id = ?1 ORDER BY id",
            )
            .unwrap()
            .query_map([account_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows
    }

    #[test]
    fn opening_twice_is_a_no_op() {
        let path = a_fresh_database_file();
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

    /// AC-9: whether a key exists at all is the presence of the row, and a
    /// fresh account has none, so the first hotkey press gets the setup screen.
    #[test]
    fn a_new_account_has_no_deepgram_key() {
        let store = Store::open_in_memory().unwrap();
        assert!(store.deepgram_key_for("acct_test").unwrap().is_none());
        assert_eq!(store.deepgram_key_for("acct_test").unwrap(), None);
    }

    /// AC-10: a saved key survives a restart, which for this table means it
    /// reads back exactly as written.
    #[test]
    fn a_saved_key_round_trips_and_keeps_only_four_characters() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_deepgram_key(
                "acct_test",
                "cdef",
                "deepgram:acct_test",
                "2026-08-30T10:00:00Z",
            )
            .unwrap();

        assert!(store.deepgram_key_for("acct_test").unwrap().is_some());
        let saved = store.deepgram_key_for("acct_test").unwrap().unwrap();
        assert_eq!(saved.key_last_four, "cdef");
        assert_eq!(saved.credential_target, "deepgram:acct_test");
        assert_eq!(saved.saved_at, "2026-08-30T10:00:00Z");
        assert_eq!(
            saved.last_validated_at,
            Some("2026-08-30T10:00:00Z".to_string())
        );
    }

    /// An account has zero or one, per the data model. Replacing a key
    /// overwrites rather than adding a second row.
    #[test]
    fn replacing_a_key_overwrites_the_one_row() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_deepgram_key(
                "acct_test",
                "cdef",
                "deepgram:acct_test",
                "2026-08-30T10:00:00Z",
            )
            .unwrap();
        store
            .save_deepgram_key(
                "acct_test",
                "wxyz",
                "deepgram:acct_test",
                "2026-08-30T11:00:00Z",
            )
            .unwrap();

        let saved = store.deepgram_key_for("acct_test").unwrap().unwrap();
        assert_eq!(saved.key_last_four, "wxyz");
        assert_eq!(saved.saved_at, "2026-08-30T11:00:00Z");

        let conn = store.conn.lock().unwrap();
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM deepgram_credential", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1);
    }

    /// After clearing, AC-9 holds again: the next hotkey press gets the setup
    /// screen rather than the microphone.
    #[test]
    fn clearing_a_key_leaves_no_row() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_deepgram_key(
                "acct_test",
                "cdef",
                "deepgram:acct_test",
                "2026-08-30T10:00:00Z",
            )
            .unwrap();
        store.clear_deepgram_key("acct_test").unwrap();
        assert!(store.deepgram_key_for("acct_test").unwrap().is_none());
        assert_eq!(store.deepgram_key_for("acct_test").unwrap(), None);
    }

    #[test]
    fn clearing_a_key_that_was_never_there_is_fine() {
        let store = Store::open_in_memory().unwrap();
        store.clear_deepgram_key("acct_test").unwrap();
        assert!(store.deepgram_key_for("acct_test").unwrap().is_none());
    }

    /// AC-18: a second account on the same machine sees none of the first
    /// account's key, and clearing one leaves the other alone.
    #[test]
    fn a_second_account_cannot_see_the_first_accounts_key() {
        let store = Store::open_in_memory().unwrap();
        {
            let conn = store.conn.lock().unwrap();
            conn.execute("INSERT INTO account (id) VALUES ('acct_other')", [])
                .unwrap();
        }
        store
            .save_deepgram_key(
                "acct_test",
                "cdef",
                "deepgram:acct_test",
                "2026-08-30T10:00:00Z",
            )
            .unwrap();

        assert!(store.deepgram_key_for("acct_other").unwrap().is_none());
        assert_eq!(store.deepgram_key_for("acct_other").unwrap(), None);

        store
            .save_deepgram_key(
                "acct_other",
                "wxyz",
                "deepgram:acct_other",
                "2026-08-30T11:00:00Z",
            )
            .unwrap();
        store.clear_deepgram_key("acct_other").unwrap();

        // The first account is untouched by the second's arrival and departure.
        let saved = store.deepgram_key_for("acct_test").unwrap().unwrap();
        assert_eq!(saved.key_last_four, "cdef");
        assert_eq!(saved.credential_target, "deepgram:acct_test");
    }

    /// The data rules say the key itself is never in this database. Nothing in
    /// the table's shape allows it, and this is the test that says so.
    #[test]
    fn the_table_has_no_column_that_could_hold_the_key() {
        let store = Store::open_in_memory().unwrap();
        let conn = store.conn.lock().unwrap();
        let mut columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('deepgram_credential')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        columns.sort();
        assert_eq!(
            columns,
            vec![
                "account_id".to_string(),
                "credential_target".to_string(),
                "key_last_four".to_string(),
                "last_validated_at".to_string(),
                "saved_at".to_string(),
            ]
        );
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

    /// AC-22: exactly two, and nowhere for a third to come from. The interface
    /// renders this list rather than deciding for itself what is allowed.
    #[test]
    fn there_are_exactly_two_hotkeys_to_choose_from() {
        assert_eq!(
            Hotkey::CHOICES,
            [Hotkey::DoubleTapCtrl, Hotkey::DoubleTapAlt]
        );
        assert_eq!(Hotkey::DoubleTapCtrl.as_stored(), "double_tap_ctrl");
        assert_eq!(Hotkey::DoubleTapAlt.as_stored(), "double_tap_alt");
    }

    /// AC-22: anything that is not one of the two is refused outright, rather
    /// than quietly becoming Ctrl the way an unreadable stored row does.
    #[test]
    fn a_hotkey_nobody_offered_is_refused_rather_than_defaulted() {
        assert_eq!(
            Hotkey::from_chosen("double_tap_ctrl"),
            Some(Hotkey::DoubleTapCtrl)
        );
        assert_eq!(
            Hotkey::from_chosen("double_tap_alt"),
            Some(Hotkey::DoubleTapAlt)
        );
        for refused in ["", "f13", "double_tap_shift", "DOUBLE_TAP_ALT", "ctrl"] {
            assert_eq!(
                Hotkey::from_chosen(refused),
                None,
                "`{refused}` was accepted as a hotkey. Exactly two exist and \
                 there is no way to set another (record 0002 AC-22)"
            );
        }
    }

    /// AC-19: picking the other hotkey is remembered, and picking it back again
    /// works. Nothing else on the row moves.
    #[test]
    fn the_chosen_hotkey_round_trips_and_leaves_everything_else_alone() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_pill_spot("acct_test", 0.25, 0.8, "2026-09-02T10:00:00Z")
            .unwrap();
        store
            .save_sounds_enabled("acct_test", false, "2026-09-02T10:01:00Z")
            .unwrap();

        store
            .save_hotkey("acct_test", Hotkey::DoubleTapAlt, "2026-09-02T10:02:00Z")
            .unwrap();
        let setting = store.setting_for("acct_test").unwrap();
        assert_eq!(setting.hotkey, Hotkey::DoubleTapAlt);
        assert_eq!(setting.pill_x, 0.25);
        assert_eq!(setting.pill_y, 0.8);
        assert!(!setting.sounds_enabled);

        store
            .save_hotkey("acct_test", Hotkey::DoubleTapCtrl, "2026-09-02T10:03:00Z")
            .unwrap();
        assert_eq!(
            store.setting_for("acct_test").unwrap().hotkey,
            Hotkey::DoubleTapCtrl
        );
    }

    /// AC-21: one switch, on to begin with, and off is remembered.
    #[test]
    fn the_sound_switch_round_trips_and_leaves_everything_else_alone() {
        let store = Store::open_in_memory().unwrap();
        // On for a new account, which for this table means with no row at all.
        assert!(store.setting_for("acct_test").unwrap().sounds_enabled);

        store
            .save_hotkey("acct_test", Hotkey::DoubleTapAlt, "2026-09-02T10:00:00Z")
            .unwrap();
        store
            .save_sounds_enabled("acct_test", false, "2026-09-02T10:01:00Z")
            .unwrap();
        let setting = store.setting_for("acct_test").unwrap();
        assert!(!setting.sounds_enabled);
        assert_eq!(setting.hotkey, Hotkey::DoubleTapAlt);

        store
            .save_sounds_enabled("acct_test", true, "2026-09-02T10:02:00Z")
            .unwrap();
        assert!(store.setting_for("acct_test").unwrap().sounds_enabled);
    }

    /// AC-17: a finished dictation is kept with its text, its start time and
    /// how long it lasted.
    #[test]
    fn a_finished_dictation_is_kept_with_its_text_start_and_duration() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_dictation(
                "acct_test",
                "Hello there. This is a test.",
                "2026-09-02T10:00:00Z",
                4200,
            )
            .unwrap();

        assert_eq!(
            dictations_of(&store, "acct_test"),
            vec![(
                "Hello there. This is a test.".to_string(),
                "2026-09-02T10:00:00Z".to_string(),
                4200
            )]
        );
    }

    /// Every dictation is its own row, in the order they happened. Nothing here
    /// overwrites, unlike the one-per-account settings and key rows.
    #[test]
    fn a_second_dictation_is_a_second_row() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_dictation("acct_test", "first", "2026-09-02T10:00:00Z", 1000)
            .unwrap();
        store
            .save_dictation("acct_test", "second", "2026-09-02T10:05:00Z", 2000)
            .unwrap();
        let rows = dictations_of(&store, "acct_test");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "first");
        assert_eq!(rows[1].0, "second");
    }

    /// AC-17, and what "completed" was settled to mean on 2026-09-02: a
    /// dictation that typed nothing has nothing to keep. A silent dictation and
    /// a password refusal both land here, and neither leaves a blank row for
    /// plan row 5's history screen to show.
    #[test]
    fn a_dictation_that_typed_nothing_is_not_saved() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_dictation("acct_test", "", "2026-09-02T10:00:00Z", 31_000)
            .unwrap();
        assert!(dictations_of(&store, "acct_test").is_empty());
    }

    /// AC-18: a second account on the same machine sees none of the first
    /// account's dictations.
    #[test]
    fn a_second_account_cannot_see_the_first_accounts_dictations() {
        let store = Store::open_in_memory().unwrap();
        {
            let conn = store.conn.lock().unwrap();
            conn.execute("INSERT INTO account (id) VALUES ('acct_other')", [])
                .unwrap();
        }
        store
            .save_dictation("acct_test", "mine", "2026-09-02T10:00:00Z", 1000)
            .unwrap();

        assert!(dictations_of(&store, "acct_other").is_empty());

        store
            .save_dictation("acct_other", "theirs", "2026-09-02T10:01:00Z", 1000)
            .unwrap();
        let mine = dictations_of(&store, "acct_test");
        let theirs = dictations_of(&store, "acct_other");
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].0, "mine");
        assert_eq!(theirs.len(), 1);
        assert_eq!(theirs[0].0, "theirs");
    }

    /// AC-17's second half, and AC-19's and AC-21's: still there after closing
    /// and reopening the app. For this module that means a real file, closed
    /// and opened again.
    #[test]
    fn dictations_and_both_settings_survive_reopening_the_file() {
        let path = a_fresh_database_file();
        {
            let store = Store::open(&path).expect("first open");
            store
                .save_dictation("acct_test", "said before", "2026-09-02T10:00:00Z", 3500)
                .unwrap();
            store
                .save_hotkey("acct_test", Hotkey::DoubleTapAlt, "2026-09-02T10:00:01Z")
                .unwrap();
            store
                .save_sounds_enabled("acct_test", false, "2026-09-02T10:00:02Z")
                .unwrap();
        }

        let store = Store::open(&path).expect("second open");
        assert_eq!(
            dictations_of(&store, "acct_test"),
            vec![(
                "said before".to_string(),
                "2026-09-02T10:00:00Z".to_string(),
                3500
            )]
        );
        let setting = store.setting_for("acct_test").unwrap();
        assert_eq!(setting.hotkey, Hotkey::DoubleTapAlt);
        assert!(!setting.sounds_enabled);

        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    /// The data rules say no audio and no partial transcript is ever written.
    /// Nothing in this table's shape allows either, and this is the test that
    /// says so: an id and four columns, none of them a blob.
    #[test]
    fn the_dictation_table_has_nowhere_to_put_audio() {
        let store = Store::open_in_memory().unwrap();
        let conn = store.conn.lock().unwrap();
        let mut columns: Vec<(String, String)> = conn
            .prepare("SELECT name, type FROM pragma_table_info('dictation')")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        columns.sort();
        assert_eq!(
            columns,
            vec![
                ("account_id".to_string(), "TEXT".to_string()),
                ("duration_ms".to_string(), "INTEGER".to_string()),
                ("id".to_string(), "INTEGER".to_string()),
                ("started_at".to_string(), "TEXT".to_string()),
                ("text".to_string(), "TEXT".to_string()),
            ]
        );
    }
}
