//! Local storage for the sign-in feature: the `account` and `session` tables,
//! their one migration, and the narrow read and write paths this feature needs.
//!
//! Everything that persists for EchoScribe lives in one SQLite file on the
//! person's own machine (see AGENTS.md data rules). This module opens that file
//! and owns the `account` and `session` schema. Record 0002 (dictate with a
//! hotkey) owns its own tables in its own module (`dictate::store`), on its own
//! connection to the same file, created after this one has run.
//!
//! No token is ever written here. The refresh token and access token live only
//! in the OS credential store; `session.credential_target` holds just the name
//! of that entry.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

/// Ordered schema migrations. Index `n` is applied when the database's
/// `PRAGMA user_version` is `<= n`, and sets `user_version` to `n + 1`.
///
/// Element 0 is this feature's migration: it creates `account` and `session`.
/// It runs before record 0002's tables are created, which is why the decision
/// record says the sign-in migration "runs before record 0002's".
const MIGRATIONS: &[&str] = &[
    // v0 -> v1: the sign-in feature.
    "
    CREATE TABLE account (
        id                 TEXT PRIMARY KEY NOT NULL,
        email              TEXT NOT NULL,
        display_name       TEXT NOT NULL DEFAULT '',
        first_signed_in_at TEXT NOT NULL,
        last_signed_in_at  TEXT NOT NULL
    );

    -- One row, id is always 1. NULL columns mean signed out: SQLite skips the
    -- foreign key check on a NULL child, so an empty session can coexist with
    -- the REFERENCES constraint on a real account id.
    CREATE TABLE session (
        id                INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
        account_id        TEXT REFERENCES account(id),
        credential_target TEXT,
        established_at     TEXT,
        last_verified_at  TEXT
    );
    ",
];

/// One row of the `account` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub first_signed_in_at: String,
    pub last_signed_in_at: String,
}

/// The single `session` row, when one exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub account_id: String,
    pub credential_target: String,
    pub established_at: String,
    pub last_verified_at: String,
}

/// Owns the one database connection, behind a mutex so Tauri commands on
/// different threads can share it.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open the database file at `path`, creating it if needed, and bring the
    /// schema up to date. Foreign keys are enforced on every connection.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Self::prepare(conn)
    }

    /// An in-memory database, for tests.
    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::prepare(conn)
    }

    fn prepare(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn migrate(conn: &Connection) -> rusqlite::Result<()> {
        // `user_version` is a signed 32-bit value; rusqlite reads it as i64.
        let mut version: i64 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;
        while (version as usize) < MIGRATIONS.len() {
            conn.execute_batch(MIGRATIONS[version as usize])?;
            version += 1;
            // `user_version` does not accept a bound parameter.
            conn.execute_batch(&format!("PRAGMA user_version = {version};"))?;
        }
        Ok(())
    }

    /// The current schema version, for tests and diagnostics.
    #[cfg(test)]
    pub fn schema_version(&self) -> rusqlite::Result<usize> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        let version: i64 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;
        Ok(version as usize)
    }

    /// Read the single session row. `None` means nobody has ever signed in on
    /// this machine; a row whose `account_id` is empty means signed out. NULL
    /// columns are read back as empty strings.
    pub fn read_session(&self) -> rusqlite::Result<Option<Session>> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.query_row(
            "SELECT account_id, credential_target, established_at, last_verified_at
             FROM session WHERE id = 1",
            [],
            |row| {
                Ok(Session {
                    account_id: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    credential_target: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    established_at: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    last_verified_at: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                })
            },
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
    }

    /// Read one account by its Clerk user id.
    pub fn read_account(&self, id: &str) -> rusqlite::Result<Option<Account>> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.query_row(
            "SELECT id, email, display_name, first_signed_in_at, last_signed_in_at
             FROM account WHERE id = ?1",
            [id],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    display_name: row.get(2)?,
                    first_signed_in_at: row.get(3)?,
                    last_signed_in_at: row.get(4)?,
                })
            },
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
    }

    /// Insert the account if new, otherwise update only `last_signed_in_at`,
    /// `email` and `display_name`. Never touches an existing account's other
    /// data.
    pub fn upsert_account(
        &self,
        id: &str,
        email: &str,
        display_name: &str,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO account
                 (id, email, display_name, first_signed_in_at, last_signed_in_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 email = excluded.email,
                 display_name = excluded.display_name,
                 last_signed_in_at = excluded.last_signed_in_at",
            (id, email, display_name, now_utc),
        )?;
        Ok(())
    }

    /// Point the single session row at `account_id` with the given credential
    /// entry name. `last_verified_at` starts empty: the first successful silent
    /// refresh fills it in.
    pub fn set_session(
        &self,
        account_id: &str,
        credential_target: &str,
        now_utc: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT INTO session
                 (id, account_id, credential_target, established_at, last_verified_at)
             VALUES (1, ?1, ?2, ?3, NULL)
             ON CONFLICT(id) DO UPDATE SET
                 account_id = excluded.account_id,
                 credential_target = excluded.credential_target,
                 established_at = excluded.established_at,
                 last_verified_at = NULL",
            (account_id, credential_target, now_utc),
        )?;
        Ok(())
    }

    /// Empty the session row: signed out. Leaves every `account` row and all
    /// account-owned data untouched. Used by `sign_out` (built next).
    #[allow(dead_code)] // exercised by tests now; wired to a command with the handoff
    pub fn clear_session(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "UPDATE session
             SET account_id = NULL, credential_target = NULL,
                 established_at = NULL, last_verified_at = NULL
             WHERE id = 1",
            [],
        )?;
        Ok(())
    }

    /// Record that Clerk confirmed the refresh token still works, just now.
    /// Only ever touches `last_verified_at`, and only while somebody is signed
    /// in, so a refresh landing after a sign-out cannot revive a dead session.
    pub fn mark_verified(&self, now_utc: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "UPDATE session SET last_verified_at = ?1
             WHERE id = 1 AND account_id IS NOT NULL AND account_id <> ''",
            [now_utc],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_both_tables_at_version_one() {
        let store = Store::open_in_memory().expect("open");
        assert_eq!(store.schema_version().expect("version"), 1);

        let conn = store.conn.lock().unwrap();
        let mut names: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('account', 'session')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        names.sort();
        assert_eq!(names, vec!["account".to_string(), "session".to_string()]);
    }

    #[test]
    fn migration_is_idempotent_across_reopens() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "echoscribe-signin-test-{}-{}.sqlite3",
            std::process::id(),
            now_nanos()
        ));
        {
            let s = Store::open(&path).expect("first open");
            assert_eq!(s.schema_version().unwrap(), 1);
        }
        {
            let s = Store::open(&path).expect("second open");
            assert_eq!(s.schema_version().unwrap(), 1);
            assert!(s.read_session().unwrap().is_none());
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn session_round_trips_and_clears() {
        let store = Store::open_in_memory().expect("open");
        assert!(store.read_session().unwrap().is_none());

        store
            .upsert_account(
                "user_123",
                "sam@example.com",
                "Sam Rivera",
                "2026-08-28T10:00:00Z",
            )
            .unwrap();
        store
            .set_session(
                "user_123",
                "echoscribe:session:user_123",
                "2026-08-28T10:00:00Z",
            )
            .unwrap();

        let session = store.read_session().unwrap().expect("a session");
        assert_eq!(session.account_id, "user_123");
        assert_eq!(session.credential_target, "echoscribe:session:user_123");

        store.clear_session().unwrap();
        let session = store.read_session().unwrap().expect("row still present");
        assert_eq!(session.account_id, "");
        // The account survives sign-out.
        assert!(store.read_account("user_123").unwrap().is_some());
    }

    #[test]
    fn upsert_account_keeps_first_signed_in_at() {
        let store = Store::open_in_memory().expect("open");
        store
            .upsert_account("u1", "a@example.com", "A", "2026-01-01T00:00:00Z")
            .unwrap();
        store
            .upsert_account("u1", "a@example.com", "A Renamed", "2026-06-01T00:00:00Z")
            .unwrap();

        let acct = store.read_account("u1").unwrap().unwrap();
        assert_eq!(acct.first_signed_in_at, "2026-01-01T00:00:00Z");
        assert_eq!(acct.last_signed_in_at, "2026-06-01T00:00:00Z");
        assert_eq!(acct.display_name, "A Renamed");
    }

    #[test]
    fn a_second_account_switches_in_without_disturbing_the_first() {
        // AC-10 and AC-11 from the storage side: signing in as someone else
        // leaves the first person's row exactly as it was, and switching back
        // finds it unchanged.
        let store = Store::open_in_memory().expect("open");

        store
            .upsert_account("user_a", "a@example.com", "Ada", "2026-08-01T09:00:00Z")
            .unwrap();
        store
            .set_session("user_a", "session:user_a", "2026-08-01T09:00:00Z")
            .unwrap();
        store.clear_session().unwrap();

        store
            .upsert_account("user_b", "b@example.com", "Bo", "2026-08-29T09:00:00Z")
            .unwrap();
        store
            .set_session("user_b", "session:user_b", "2026-08-29T09:00:00Z")
            .unwrap();

        // Two accounts on the machine, one session, pointed at the second.
        let session = store.read_session().unwrap().unwrap();
        assert_eq!(session.account_id, "user_b");
        assert_eq!(session.credential_target, "session:user_b");

        // The first person's row is untouched, down to the date they joined.
        let ada = store
            .read_account("user_a")
            .unwrap()
            .expect("Ada is still here");
        assert_eq!(ada.email, "a@example.com");
        assert_eq!(ada.display_name, "Ada");
        assert_eq!(ada.first_signed_in_at, "2026-08-01T09:00:00Z");
        assert_eq!(ada.last_signed_in_at, "2026-08-01T09:00:00Z");

        // Switching back updates only when Ada signed in last.
        store.clear_session().unwrap();
        store
            .upsert_account("user_a", "a@example.com", "Ada", "2026-08-29T10:00:00Z")
            .unwrap();
        store
            .set_session("user_a", "session:user_a", "2026-08-29T10:00:00Z")
            .unwrap();

        let ada = store.read_account("user_a").unwrap().unwrap();
        assert_eq!(ada.first_signed_in_at, "2026-08-01T09:00:00Z");
        assert_eq!(ada.last_signed_in_at, "2026-08-29T10:00:00Z");
        // Bo is still on the machine, waiting for their next sign-in.
        assert!(store.read_account("user_b").unwrap().is_some());
    }

    #[test]
    fn mark_verified_fills_the_column_only_while_signed_in() {
        let store = Store::open_in_memory().expect("open");
        store
            .upsert_account("u1", "a@example.com", "A", "2026-08-28T10:00:00Z")
            .unwrap();
        store
            .set_session("u1", "target", "2026-08-28T10:00:00Z")
            .unwrap();
        assert_eq!(store.read_session().unwrap().unwrap().last_verified_at, "");

        store.mark_verified("2026-08-28T11:00:00Z").unwrap();
        assert_eq!(
            store.read_session().unwrap().unwrap().last_verified_at,
            "2026-08-28T11:00:00Z"
        );

        // A refresh that lands after sign-out must not write anything back.
        store.clear_session().unwrap();
        store.mark_verified("2026-08-28T12:00:00Z").unwrap();
        let session = store.read_session().unwrap().unwrap();
        assert_eq!(session.account_id, "");
        assert_eq!(session.last_verified_at, "");
    }

    fn now_nanos() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
