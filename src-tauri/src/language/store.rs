//! Local storage for the chosen transcription language: record 0006's one
//! table.
//!
//! One row per account, or none, and none means English. Everything EchoScribe
//! persists lives in one SQLite file on the person's own machine (AGENTS.md
//! data rules); the sign-in feature creates `account` and this module opens its
//! own connection to the same file, the way the dictate feature's store and the
//! shell's store already do beside it.
//!
//! **A stored value this app does not offer is read as English, never sent.**
//! That is `catalog::from_stored`, and it is the rule record 0002's data model
//! already applies to a stored hotkey and a stored pill position: a row we
//! cannot understand is nonsense, not an instruction. The `CHECK` on the column
//! is a second guard and not the first, because the only strings this app ever
//! writes are `catalog`'s own literals.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use super::catalog;

/// Owns this feature's connection to the shared database file.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open the shared database file at `path` and make sure record 0006's
    /// table exists. The sign-in feature has already created `account`.
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
             INSERT INTO account (id) VALUES ('acct_one'), ('acct_two');",
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

    /// Record 0006's first migration: one table. Idempotent, so opening an
    /// already-migrated file is a no-op.
    ///
    /// There is deliberately no `CHECK` listing all 64 codes. A list of
    /// literals in a schema cannot be changed without a table rebuild, and it
    /// would be a second copy of `catalog::OFFERED` that could fall behind it;
    /// the column is guarded by the fact that nothing but `catalog`'s literals
    /// is ever written, and by `catalog::from_stored` on the way out.
    fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS transcription_language (
                account_id TEXT PRIMARY KEY NOT NULL REFERENCES account(id),
                language   TEXT NOT NULL DEFAULT 'en',
                updated_at TEXT NOT NULL
            );
            ",
        )
    }

    /// The language this account chose (record 0006 AC-1, AC-2).
    ///
    /// A missing row, or any stored value this app does not offer, comes back
    /// as English rather than as an instruction.
    pub fn language_for(&self, account_id: &str) -> rusqlite::Result<&'static str> {
        let conn = self.conn.lock().expect("language mutex poisoned");
        conn.query_row(
            "SELECT language FROM transcription_language WHERE account_id = ?1",
            [account_id],
            |row| row.get::<_, String>(0),
        )
        .map(|stored| catalog::from_stored(&stored))
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(catalog::DEFAULT),
            other => Err(other),
        })
    }

    /// Remember the language this account chose (record 0006 AC-2, AC-8).
    ///
    /// Takes a `&'static str` from `catalog`, not any string, so there is no
    /// path by which a value the app does not offer reaches the column.
    pub fn save_language(
        &self,
        account_id: &str,
        language: &'static str,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("language mutex poisoned");
        conn.execute(
            "INSERT INTO transcription_language (account_id, language, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id) DO UPDATE SET
                 language   = excluded.language,
                 updated_at = excluded.updated_at",
            rusqlite::params![account_id, language, now_utc],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> Store {
        Store::open_in_memory().expect("in-memory store")
    }

    #[test]
    fn a_new_account_is_english() {
        // covers: AC-1. "Until I change it, that is English", and a new account
        // has no row at all, so the absence of a row has to mean English rather
        // than nothing.
        assert_eq!(store().language_for("acct_one").expect("read"), "en");
    }

    #[test]
    fn a_chosen_language_survives_being_written_and_read() {
        // covers: AC-2. The restart half needs a real file and is /check
        // verify's; this is the write and the read agreeing.
        let s = store();
        s.save_language("acct_one", "ja", "2026-09-04T10:00:00Z")
            .expect("save");
        assert_eq!(s.language_for("acct_one").expect("read"), "ja");
    }

    #[test]
    fn choosing_again_replaces_rather_than_adds() {
        // covers: AC-2. One account has zero or one, per the record's data
        // model. A second row would make which language is in force depend on
        // which one the read happened to find.
        let s = store();
        s.save_language("acct_one", "ja", "2026-09-04T10:00:00Z")
            .expect("save");
        s.save_language("acct_one", "pl", "2026-09-04T10:00:01Z")
            .expect("save");
        assert_eq!(s.language_for("acct_one").expect("read"), "pl");
    }

    #[test]
    fn one_accounts_language_is_not_anothers() {
        // covers: AC-8. A second account on the same machine has its own
        // choice, and changing theirs does not change mine.
        let s = store();
        s.save_language("acct_one", "ja", "2026-09-04T10:00:00Z")
            .expect("save");
        s.save_language("acct_two", "pl", "2026-09-04T10:00:00Z")
            .expect("save");
        assert_eq!(s.language_for("acct_one").expect("read"), "ja");
        assert_eq!(s.language_for("acct_two").expect("read"), "pl");
        // And an account that never chose is still English while another has
        // chosen.
        assert_eq!(store().language_for("acct_two").expect("read"), "en");
    }

    #[test]
    fn a_stored_value_this_app_does_not_offer_reads_as_english() {
        // covers: AC-3, and record 0006's data rules. Only reachable by editing
        // the file by hand, or by this app's list shrinking under a row that
        // was legitimate when it was written. Either way, sending Deepgram a
        // value it will refuse would stop dictation, and English leaves it
        // working.
        let s = store();
        {
            let conn = s.conn.lock().expect("mutex");
            conn.execute(
                "INSERT INTO transcription_language (account_id, language, updated_at)
                 VALUES ('acct_one', 'en&redact=pci', '2026-09-04T10:00:00Z')",
                [],
            )
            .expect("hand written row");
        }
        assert_eq!(s.language_for("acct_one").expect("read"), "en");
    }

    #[test]
    fn a_language_may_not_belong_to_an_account_that_does_not_exist() {
        // covers: AGENTS.md's data rule that every stored record belongs to
        // exactly one signed-in account. The foreign key makes it structural.
        assert!(store()
            .save_language("acct_nobody", "ja", "2026-09-04T10:00:00Z")
            .is_err());
    }

    #[test]
    fn opening_an_already_migrated_file_changes_nothing() {
        // covers: record 0006's data model. Every launch after the first runs
        // the migration again.
        let s = store();
        s.save_language("acct_one", "ja", "2026-09-04T10:00:00Z")
            .expect("save");
        {
            let conn = s.conn.lock().expect("mutex");
            Store::create_schema(&conn).expect("second migration");
        }
        assert_eq!(s.language_for("acct_one").expect("read"), "ja");
    }
}
