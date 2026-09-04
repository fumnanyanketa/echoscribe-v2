//! Local storage for the custom vocabulary: record 0005's one table and the
//! narrow reads and writes this feature needs.
//!
//! Everything EchoScribe persists lives in one SQLite file on the person's own
//! machine (AGENTS.md data rules). The sign-in feature opens that file first
//! and creates `account`; this module opens its own connection to the same
//! file and owns one table, the way the dictate feature's store and the
//! shell's store already do beside it.
//!
//! **Every row belongs to exactly one account.** No read and no write here
//! takes place without an account id, and the commands above get it from the
//! session rather than from the interface. That is AC-9, and AGENTS.md's data
//! rules say it applies to custom words exactly as it does to history.
//!
//! **The rules are not here.** Trimming, the character check, the duplicate
//! check and the budget all live in `rules.rs`, which is pure and testable on
//! its own. This file writes what it is given and refuses at the schema as a
//! second guard, never as the first.
//!
//! **No term is ever logged**, on any path, including a failure. `set_hotkey`
//! in record 0002 set that rule for a value arriving from outside: what it was
//! is not information worth a log line, and a log is a file.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

/// One stored word or phrase, with the identifier a remove needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub id: i64,
    /// Exactly as the person typed it, trimmed, capitals untouched.
    pub term: String,
}

/// Owns this feature's connection to the shared database file.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open the shared database file at `path` and make sure record 0005's
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

    /// Record 0005's one migration: one table and its two indexes. Idempotent,
    /// so opening an already-migrated file is a no-op.
    ///
    /// The unique index is the second guard on AC-7, after `rules::check`. It
    /// uses SQLite's `NOCASE` collation, which folds ASCII only, so it cannot
    /// catch every duplicate the rules can: that is why the check in Rust runs
    /// first and this is a backstop against a second writer rather than the
    /// mechanism.
    fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS vocabulary_term (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL REFERENCES account(id),
                term       TEXT NOT NULL,
                added_at   TEXT NOT NULL
            );

            -- The one order the screen reads it in (record 0005 AC-1).
            CREATE INDEX IF NOT EXISTS vocabulary_by_account_recent
                ON vocabulary_term (account_id, added_at DESC, id DESC);

            -- AC-7's second guard.
            CREATE UNIQUE INDEX IF NOT EXISTS vocabulary_one_term_per_account
                ON vocabulary_term (account_id, term COLLATE NOCASE);
            ",
        )
    }

    /// This account's terms, newest first (record 0005 AC-1).
    ///
    /// `id` breaks a tie on `added_at`, because two adds inside the same second
    /// share a timestamp and a list that reordered itself between two reads
    /// would make a remove land on a row a person was not looking at.
    pub fn terms_for(&self, account_id: &str) -> rusqlite::Result<Vec<Term>> {
        let conn = self.conn.lock().expect("vocabulary mutex poisoned");
        let terms = conn
            .prepare(
                "SELECT id, term FROM vocabulary_term
                 WHERE account_id = ?1
                 ORDER BY added_at DESC, id DESC",
            )?
            .query_map([account_id], |row| {
                Ok(Term {
                    id: row.get(0)?,
                    term: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<Term>, _>>()?;
        Ok(terms)
    }

    /// Keep one word against this account (record 0005 AC-2).
    ///
    /// Takes the term already checked by `rules::check`, so nothing here trims,
    /// shortens or alters it. A duplicate the unique index catches comes back as
    /// an error rather than being swallowed, because the caller has a sentence
    /// for exactly that and quietly succeeding would show a person a list that
    /// did not change.
    pub fn add_term(&self, account_id: &str, term: &str, now_utc: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("vocabulary mutex poisoned");
        conn.execute(
            "INSERT INTO vocabulary_term (account_id, term, added_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![account_id, term, now_utc],
        )?;
        Ok(())
    }

    /// Forget one of this account's words (record 0005 AC-4).
    ///
    /// **Scoped to the account, always.** An identifier belonging to somebody
    /// else's row matches nothing and removes nothing, which is AC-9 holding
    /// even against an interface asking for a row it should not know about.
    /// Removing nothing is not an error: there is nothing to tell a person and
    /// nothing went wrong.
    pub fn remove_term(&self, account_id: &str, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("vocabulary mutex poisoned");
        conn.execute(
            "DELETE FROM vocabulary_term WHERE account_id = ?1 AND id = ?2",
            rusqlite::params![account_id, id],
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

    fn words(store: &Store, account: &str) -> Vec<String> {
        store
            .terms_for(account)
            .expect("read")
            .into_iter()
            .map(|t| t.term)
            .collect()
    }

    #[test]
    fn a_new_account_has_no_words() {
        // covers: AC-10. The empty state is the first thing every person sees,
        // so it has to be what a fresh account actually reads back.
        assert!(words(&store(), "acct_one").is_empty());
    }

    #[test]
    fn a_word_is_kept_exactly_as_it_was_given() {
        // covers: AC-2, AC-7. Capitals are information: Deepgram's own guidance
        // is to keep them for proper nouns. Nothing here may normalise.
        let s = store();
        s.add_term("acct_one", "Fumnanya Nketa", "2026-09-04T10:00:00Z")
            .expect("add");
        assert_eq!(words(&s, "acct_one"), vec!["Fumnanya Nketa"]);
    }

    #[test]
    fn the_list_comes_back_newest_first() {
        // covers: AC-1, AC-2. The word you just added is the one you are
        // looking for, so it is at the top where an add can be seen to have
        // worked (record 0005's value sourcing).
        let s = store();
        s.add_term("acct_one", "first", "2026-09-04T10:00:00Z")
            .expect("add");
        s.add_term("acct_one", "second", "2026-09-04T10:00:01Z")
            .expect("add");
        s.add_term("acct_one", "third", "2026-09-04T10:00:02Z")
            .expect("add");
        assert_eq!(words(&s, "acct_one"), vec!["third", "second", "first"]);
    }

    #[test]
    fn two_adds_in_the_same_second_still_have_one_stable_order() {
        // covers: AC-1, AC-4. Timestamps here are whole seconds, so a burst of
        // adds shares one. Without the id breaking the tie the order could
        // differ between two reads, and a remove would land on a row the person
        // was not looking at.
        let s = store();
        for word in ["a", "b", "c"] {
            s.add_term("acct_one", word, "2026-09-04T10:00:00Z")
                .expect("add");
        }
        let once = words(&s, "acct_one");
        assert_eq!(once, vec!["c", "b", "a"]);
        assert_eq!(words(&s, "acct_one"), once);
    }

    #[test]
    fn the_same_word_twice_is_refused_by_the_schema_as_well() {
        // covers: AC-7. `rules::check` refuses it first; this is the second
        // guard, so a duplicate cannot reach the table by any route. It comes
        // back as an error rather than quietly succeeding, because the caller
        // has a sentence for it and a silent success would show a person a list
        // that did not change.
        let s = store();
        s.add_term("acct_one", "Deepgram", "2026-09-04T10:00:00Z")
            .expect("add");
        assert!(s
            .add_term("acct_one", "deepgram", "2026-09-04T10:00:01Z")
            .is_err());
        assert_eq!(words(&s, "acct_one"), vec!["Deepgram"]);
    }

    #[test]
    fn one_accounts_words_are_invisible_to_another() {
        // covers: AC-9. The same protection record 0002's AC-18 gives history.
        let s = store();
        s.add_term("acct_one", "mine", "2026-09-04T10:00:00Z")
            .expect("add");
        s.add_term("acct_two", "theirs", "2026-09-04T10:00:00Z")
            .expect("add");
        assert_eq!(words(&s, "acct_one"), vec!["mine"]);
        assert_eq!(words(&s, "acct_two"), vec!["theirs"]);
    }

    #[test]
    fn the_same_word_may_be_added_by_two_different_accounts() {
        // covers: AC-9. The unique index is per account, not global. Two people
        // on one machine both dictating about Deepgram is ordinary.
        let s = store();
        s.add_term("acct_one", "Deepgram", "2026-09-04T10:00:00Z")
            .expect("add");
        s.add_term("acct_two", "Deepgram", "2026-09-04T10:00:00Z")
            .expect("the second account was refused a word the first one has");
    }

    #[test]
    fn removing_takes_only_that_one_word() {
        // covers: AC-4.
        let s = store();
        for word in ["one", "two", "three"] {
            s.add_term("acct_one", word, "2026-09-04T10:00:00Z")
                .expect("add");
        }
        let terms = s.terms_for("acct_one").expect("read");
        let two = terms
            .iter()
            .find(|t| t.term == "two")
            .expect("two is there");
        s.remove_term("acct_one", two.id).expect("remove");
        assert_eq!(words(&s, "acct_one"), vec!["three", "one"]);
    }

    #[test]
    fn one_account_cannot_remove_anothers_word() {
        // covers: AC-9, and it is the interesting half of it. The identifier is
        // a number the interface could guess, so the delete is scoped to the
        // account as well as the row. Removing nothing is not an error: there
        // is nothing to tell a person and nothing went wrong.
        let s = store();
        s.add_term("acct_one", "mine", "2026-09-04T10:00:00Z")
            .expect("add");
        let id = s.terms_for("acct_one").expect("read")[0].id;
        s.remove_term("acct_two", id)
            .expect("a foreign remove is not an error");
        assert_eq!(words(&s, "acct_one"), vec!["mine"]);
    }

    #[test]
    fn removing_a_word_that_is_not_there_is_not_a_failure() {
        // covers: AC-4. Two removes of one row, which a double press can cause,
        // leave the same state and no error a person has to read.
        let s = store();
        s.remove_term("acct_one", 4321).expect("remove");
    }

    #[test]
    fn a_word_may_not_belong_to_an_account_that_does_not_exist() {
        // covers: AGENTS.md's data rule that every stored record belongs to
        // exactly one signed-in account. The foreign key is what makes that
        // structural rather than a habit.
        let s = store();
        assert!(s
            .add_term("acct_nobody", "orphan", "2026-09-04T10:00:00Z")
            .is_err());
    }

    /// A real file on disk, with the `account` table the sign-in feature makes.
    /// The in-memory store cannot answer "after closing and reopening the app",
    /// because there is nothing to close. Same helper, same shape, as the
    /// dictate feature's store.
    fn a_fresh_database_file() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "echoscribe-vocabulary-test-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE account (id TEXT PRIMARY KEY NOT NULL);
             INSERT INTO account (id) VALUES ('acct_one'), ('acct_two');",
        )
        .unwrap();
        path
    }

    #[test]
    fn words_survive_closing_and_reopening_the_app() {
        // covers: AC-2's second half and AC-4's third. "Still there after
        // closing and reopening the app" and "still gone after a restart" are
        // the two halves of the same promise, and neither can be answered by an
        // in-memory database, because there is nothing to close. So this one
        // uses a real file, closes it, and opens it again, which is as close to
        // a restart as a test gets.
        let path = a_fresh_database_file();
        let removed_id;
        {
            let store = Store::open(&path).expect("first open");
            store
                .add_term("acct_one", "Fumnanya Nketa", "2026-09-04T10:00:00Z")
                .unwrap();
            store
                .add_term("acct_one", "EchoScribe", "2026-09-04T10:00:01Z")
                .unwrap();
            store
                .add_term("acct_one", "gone by morning", "2026-09-04T10:00:02Z")
                .unwrap();
            removed_id = store
                .terms_for("acct_one")
                .unwrap()
                .into_iter()
                .find(|t| t.term == "gone by morning")
                .expect("it was just added")
                .id;
            store.remove_term("acct_one", removed_id).unwrap();
        }

        let store = Store::open(&path).expect("second open");
        // The two that were added are there, still newest first, and the one
        // that was removed is still gone.
        assert_eq!(
            words(&store, "acct_one"),
            vec!["EchoScribe", "Fumnanya Nketa"]
        );

        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn opening_an_already_migrated_file_changes_nothing() {
        // covers: record 0005's data model, "one migration, creating one table
        // and its two indexes", idempotent. Every launch after the first runs
        // it again.
        let s = store();
        s.add_term("acct_one", "kept", "2026-09-04T10:00:00Z")
            .expect("add");
        {
            let conn = s.conn.lock().expect("mutex");
            Store::create_schema(&conn).expect("second migration");
        }
        assert_eq!(words(&s, "acct_one"), vec!["kept"]);
    }
}
