//! Encrypted local cache.
//!
//! One SQLCipher database per account, keyed by a BLAKE3 derivation of the
//! user's group key. It holds mail headers, cached bodies, folder snapshots
//! and the full-text index. Nothing is readable without the Tuta session, and
//! bodies of deleted mail are overwritten by SQLCipher's secure-delete.
//!
//! SQLite operations are synchronous and short; callers hold the connection
//! for one statement batch at a time.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};
use zeroize::Zeroize;

use crate::error::{CoreError, Result};
use crate::models::{MailSummary, SearchHit};

pub struct Store {
    connection: Mutex<Connection>,
    account: String,
    fts: bool,
}

impl Store {
    /// Opens (or creates) the encrypted cache. `key` must be 32 bytes.
    pub fn open(path: &Path, account: &str, key: &mut [u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(CoreError::CacheUnavailable);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| CoreError::Storage)?;
        }
        let connection = Connection::open(path).map_err(|e| {
            log::error!("cannot open cache: {e}");
            CoreError::CacheUnavailable
        })?;
        let hex = hex_string(key);
        connection
            .execute_batch(&format!(
                "PRAGMA key = \"x'{hex}'\";  PRAGMA cipher_memory_security = ON;  PRAGMA journal_mode = WAL;  PRAGMA synchronous = NORMAL;  PRAGMA secure_delete = ON;  PRAGMA foreign_keys = ON;  PRAGMA temp_store = MEMORY;"
            ))
            .map_err(|_| CoreError::CacheUnavailable)?;
        // A wrong key or corrupt file fails here, not on the first query.
        connection
            .query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|_| CoreError::CacheLocked)?;
        let fts = connection
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS mail_fts USING fts5(  account UNINDEXED, mail_id UNINDEXED, folder_id UNINDEXED,  subject, sender, body, tokenize = 'unicode61 remove_diacritics 2');",
            )
            .is_ok();
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS mails (  account TEXT NOT NULL,  id TEXT NOT NULL,  folder_id TEXT NOT NULL,  received_ms INTEGER NOT NULL,  unread INTEGER NOT NULL,  json TEXT NOT NULL,  body_text TEXT NOT NULL DEFAULT '',  updated_ms INTEGER NOT NULL,  PRIMARY KEY (account, id, folder_id)  );  CREATE INDEX IF NOT EXISTS mails_folder_time  ON mails(account, folder_id, received_ms DESC);  CREATE TABLE IF NOT EXISTS bodies (  account TEXT NOT NULL, id TEXT NOT NULL, html TEXT NOT NULL,  text TEXT NOT NULL, cached_ms INTEGER NOT NULL,  PRIMARY KEY (account, id)  );  CREATE TABLE IF NOT EXISTS folders (  account TEXT PRIMARY KEY, json TEXT NOT NULL, updated_ms INTEGER NOT NULL  );  CREATE TABLE IF NOT EXISTS event_cursors (  account TEXT NOT NULL, group_id TEXT NOT NULL, batch_id TEXT NOT NULL,  updated_ms INTEGER NOT NULL, PRIMARY KEY (account, group_id)  );  CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
            )
            .map_err(|e| {
                log::error!("cache schema failed: {e}");
                CoreError::CacheUnavailable
            })?;
        Ok(Self {
            connection: Mutex::new(connection),
            account: account.to_ascii_lowercase(),
            fts,
        })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        match self.connection.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // A panic in a previous query must not brick the cache; SQLite
                // itself is still consistent after a failed statement.
                poisoned.into_inner()
            }
        }
    }

    /// Inserts or refreshes list rows. `mails` carries the summary and the
    /// already-known body text (empty when the body is not cached).
    pub fn upsert_mails(&self, folder_id: &str, mails: &[(MailSummary, String)]) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(CoreError::from)?;
        let now = now_ms();
        for (mail, body) in mails {
            let json = serde_json::to_string(mail).map_err(|_| CoreError::Storage)?;
            tx.execute(
                "INSERT INTO mails(account,id,folder_id,received_ms,unread,json,body_text,updated_ms)  VALUES(?1,?2,?3,?4,?5,?6,?7,?8)  ON CONFLICT(account,id,folder_id) DO UPDATE SET  received_ms=excluded.received_ms, unread=excluded.unread,  json=excluded.json, body_text=excluded.body_text, updated_ms=excluded.updated_ms",
                params![
                    self.account,
                    mail.id,
                    folder_id,
                    mail.received_ms,
                    i64::from(mail.unread),
                    json,
                    body,
                    now
                ],
            )
            .map_err(CoreError::from)?;
            if self.fts {
                tx.execute(
                    "DELETE FROM mail_fts WHERE account=?1 AND mail_id=?2 AND folder_id=?3",
                    params![self.account, mail.id, folder_id],
                )
                .map_err(CoreError::from)?;
                tx.execute(
                    "INSERT INTO mail_fts(account,mail_id,folder_id,subject,sender,body)  VALUES(?1,?2,?3,?4,?5,?6)",
                    params![
                        self.account,
                        mail.id,
                        folder_id,
                        mail.subject,
                        format!("{} {}", mail.sender.name, mail.sender.address),
                        body
                    ],
                )
                .map_err(CoreError::from)?;
            }
        }
        tx.commit().map_err(CoreError::from)?;
        Ok(())
    }

    pub fn list_folder(
        &self,
        folder_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MailSummary>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT json FROM mails WHERE account=?1 AND folder_id=?2  ORDER BY received_ms DESC LIMIT ?3 OFFSET ?4",
            )
            .map_err(CoreError::from)?;
        let rows = stmt
            .query_map(
                params![
                    self.account,
                    folder_id,
                    i64::try_from(limit).unwrap_or(i64::MAX),
                    i64::try_from(offset).unwrap_or(0)
                ],
                |row| row.get::<_, String>(0),
            )
            .map_err(CoreError::from)?;
        let mut out = Vec::new();
        for row in rows {
            let json = row.map_err(CoreError::from)?;
            if let Ok(mail) = serde_json::from_str(&json) {
                out.push(mail);
            }
        }
        Ok(out)
    }

    pub fn get_mail(&self, id: &str) -> Result<Option<MailSummary>> {
        let conn = self.conn();
        let row: Option<(String, bool)> = conn
            .query_row(
                "SELECT json, unread FROM mails WHERE account=?1 AND id=?2 LIMIT 1",
                params![self.account, id],
                |row| Ok((row.get(0)?, row.get::<_, i64>(1)? != 0)),
            )
            .optional()
            .map_err(CoreError::from)?;
        Ok(row.and_then(|(json, unread)| {
            serde_json::from_str::<MailSummary>(&json)
                .ok()
                .map(|mut mail| {
                    mail.unread = unread;
                    mail
                })
        }))
    }

    pub fn set_unread(&self, id: &str, unread: bool) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE mails SET unread=?1 WHERE account=?2 AND id=?3",
            params![i64::from(unread), self.account, id],
        )
        .map_err(CoreError::from)?;
        Ok(())
    }

    pub fn remove_mail(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM mails WHERE account=?1 AND id=?2",
            params![self.account, id],
        )
        .map_err(CoreError::from)?;
        conn.execute(
            "DELETE FROM bodies WHERE account=?1 AND id=?2",
            params![self.account, id],
        )
        .map_err(CoreError::from)?;
        if self.fts {
            conn.execute(
                "DELETE FROM mail_fts WHERE account=?1 AND mail_id=?2",
                params![self.account, id],
            )
            .map_err(CoreError::from)?;
        }
        Ok(())
    }

    pub fn put_body(&self, id: &str, html: &str, text: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO bodies(account,id,html,text,cached_ms) VALUES(?1,?2,?3,?4,?5)  ON CONFLICT(account,id) DO UPDATE SET html=excluded.html,text=excluded.text,cached_ms=excluded.cached_ms",
            params![self.account, id, html, text, now_ms()],
        )
        .map_err(CoreError::from)?;
        conn.execute(
            "UPDATE mails SET body_text=?1 WHERE account=?2 AND id=?3",
            params![text, self.account, id],
        )
        .map_err(CoreError::from)?;
        if self.fts {
            conn.execute(
                "UPDATE mail_fts SET body=?1 WHERE account=?2 AND mail_id=?3",
                params![text, self.account, id],
            )
            .map_err(CoreError::from)?;
        }
        Ok(())
    }

    pub fn get_body(&self, id: &str) -> Result<Option<(String, String)>> {
        let conn = self.conn();
        conn.query_row(
            "SELECT html,text FROM bodies WHERE account=?1 AND id=?2",
            params![self.account, id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(CoreError::from)
    }

    pub fn save_folders(&self, json: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO folders(account,json,updated_ms) VALUES(?1,?2,?3)  ON CONFLICT(account) DO UPDATE SET json=excluded.json,updated_ms=excluded.updated_ms",
            params![self.account, json, now_ms()],
        )
        .map_err(CoreError::from)?;
        Ok(())
    }

    pub fn load_folders(&self) -> Result<Option<String>> {
        let conn = self.conn();
        conn.query_row(
            "SELECT json FROM folders WHERE account=?1",
            params![self.account],
            |row| row.get(0),
        )
        .optional()
        .map_err(CoreError::from)
    }

    pub fn save_cursor(&self, group_id: &str, batch_id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO event_cursors(account,group_id,batch_id,updated_ms) VALUES(?1,?2,?3,?4)  ON CONFLICT(account,group_id) DO UPDATE SET batch_id=excluded.batch_id,updated_ms=excluded.updated_ms",
            params![self.account, group_id, batch_id, now_ms()],
        )
        .map_err(CoreError::from)?;
        Ok(())
    }

    pub fn load_cursors(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT group_id,batch_id FROM event_cursors WHERE account=?1")
            .map_err(CoreError::from)?;
        let rows = stmt
            .query_map(params![self.account], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(CoreError::from)?;
        let mut out = std::collections::HashMap::new();
        for row in rows {
            let (group, batch) = row.map_err(CoreError::from)?;
            out.insert(group, batch);
        }
        Ok(out)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.conn();
        if self.fts {
            let fts_query = fts_query(trimmed);
            if !fts_query.is_empty() {
                let mut stmt = conn
                    .prepare(
                        "SELECT m.json, snippet(mail_fts, 5, '', '', '…', 14) FROM mail_fts f JOIN mails m                           ON m.account=f.account AND m.id=f.mail_id AND m.folder_id=f.folder_id WHERE f.account=?1 AND mail_fts MATCH ?2 ORDER BY rank LIMIT ?3",
                    )
                    .map_err(CoreError::from)?;
                let rows = stmt
                    .query_map(
                        params![self.account, fts_query, i64::try_from(limit).unwrap_or(50)],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )
                    .map_err(CoreError::from)?;
                let mut hits: Vec<SearchHit> = Vec::new();
                for row in rows {
                    let (json, snippet) = row.map_err(CoreError::from)?;
                    if let Ok(mail) = serde_json::from_str::<MailSummary>(&json) {
                        if hits.iter().any(|hit| hit.mail.id == mail.id) {
                            continue;
                        }
                        hits.push(SearchHit { mail, snippet });
                    }
                }
                return Ok(hits);
            }
        }
        let like = format!("%{}%", trimmed.replace('%', ""));
        let mut stmt = conn
            .prepare(
                "SELECT json,body_text FROM mails WHERE account=?1  AND (json LIKE ?2 OR body_text LIKE ?2) ORDER BY received_ms DESC LIMIT ?3",
            )
            .map_err(CoreError::from)?;
        let rows = stmt
            .query_map(
                params![self.account, like, i64::try_from(limit).unwrap_or(50)],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(CoreError::from)?;
        let mut hits = Vec::new();
        for row in rows {
            let (json, text) = row.map_err(CoreError::from)?;
            if let Ok(mail) = serde_json::from_str::<MailSummary>(&json) {
                let snippet = text.chars().take(160).collect();
                hits.push(SearchHit { mail, snippet });
            }
        }
        Ok(hits)
    }

    /// Unread counts per folder, keyed by folder id. One row per placement,
    /// so a mail in two folders counts once in each.
    pub fn unread_counts(&self) -> Result<std::collections::HashMap<String, usize>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT folder_id, count(*) FROM mails WHERE account=?1 AND unread=1 GROUP BY folder_id",
            )
            .map_err(CoreError::from)?;
        let rows = stmt
            .query_map(params![self.account], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(CoreError::from)?;
        let mut out = std::collections::HashMap::new();
        for row in rows {
            let (folder, count) = row.map_err(CoreError::from)?;
            out.insert(folder, usize::try_from(count).unwrap_or(0));
        }
        Ok(out)
    }

    pub fn counts(&self) -> Result<(usize, usize)> {
        let conn = self.conn();
        let mails: i64 = conn
            .query_row(
                "SELECT count(*) FROM mails WHERE account=?1",
                params![self.account],
                |row| row.get(0),
            )
            .map_err(CoreError::from)?;
        let bodies: i64 = conn
            .query_row(
                "SELECT count(*) FROM bodies WHERE account=?1",
                params![self.account],
                |row| row.get(0),
            )
            .map_err(CoreError::from)?;
        Ok((
            usize::try_from(mails).unwrap_or(0),
            usize::try_from(bodies).unwrap_or(0),
        ))
    }

    pub fn clear(&self) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(CoreError::from)?;
        for table in ["mails", "bodies", "event_cursors"] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE account=?1"),
                params![self.account],
            )
            .map_err(CoreError::from)?;
        }
        tx.execute(
            "DELETE FROM folders WHERE account=?1",
            params![self.account],
        )
        .map_err(CoreError::from)?;
        if self.fts {
            tx.execute(
                "DELETE FROM mail_fts WHERE account=?1",
                params![self.account],
            )
            .map_err(CoreError::from)?;
        }
        tx.commit().map_err(CoreError::from)?;
        Ok(())
    }
}

fn hex_string(key: &[u8]) -> String {
    let mut out = String::with_capacity(key.len() * 2);
    for byte in key {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Turn user input into a safe FTS5 query: each alphanumeric run becomes a
/// quoted prefix token. Quoting prevents syntax errors from `:`, `*`, `-`.
fn fts_query(input: &str) -> String {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in input.chars() {
        if ch.is_alphanumeric() || ch == '@' || ch == '.' || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
        .into_iter()
        .map(|token| format!("\"{}\"*", token.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Ensures an in-memory key copy is wiped after use.
pub fn wipe(key: &mut Vec<u8>) {
    key.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Address;

    fn summary(id: &str, subject: &str, unread: bool) -> MailSummary {
        MailSummary {
            id: id.into(),
            element_id: id.into(),
            list_id: "list".into(),
            subject: subject.into(),
            sender: Address {
                name: "Ada".into(),
                address: "ada@example.com".into(),
            },
            received_ms: 1_700_000_000_000,
            unread,
            has_attachments: false,
            attachment_count: 0,
            confidential: false,
            is_draft: false,
            folder_ids: vec!["inbox".into()],
            conversation_id: "conv".into(),
            reply_type: 0,
            phishing_status: 0,
        }
    }

    fn store(dir: &std::path::Path) -> Store {
        let mut key = [7u8; 32];
        Store::open(&dir.join("cache.db"), "me@tuta.com", &mut key).unwrap()
    }

    #[test]
    fn round_trips_headers_and_bodies() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let mails = vec![(summary("m1", "Quarterly report", true), "body".to_string())];
        store.upsert_mails("inbox", &mails).unwrap();
        assert_eq!(store.list_folder("inbox", 10, 0).unwrap().len(), 1);
        store.put_body("m1", "<p>hello</p>", "hello world").unwrap();
        assert_eq!(store.get_body("m1").unwrap().unwrap().1, "hello world");
        store.set_unread("m1", false).unwrap();
        assert!(!store.get_mail("m1").unwrap().unwrap().unread);
        store.remove_mail("m1").unwrap();
        assert!(store.get_mail("m1").unwrap().is_none());
    }

    #[test]
    fn search_finds_subject_and_body() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        store
            .upsert_mails(
                "inbox",
                &[
                    (summary("m1", "Quarterly report", false), String::new()),
                    (summary("m2", "Lunch", false), String::new()),
                ],
            )
            .unwrap();
        store
            .put_body("m1", "<p>x</p>", "revenue grew in berlin")
            .unwrap();
        let hits = store.search("berlin", 10).unwrap();
        assert_eq!(hits.len(), 1);
        let hits = store.search("quarterly", 10).unwrap();
        assert_eq!(hits[0].mail.id, "m1");
    }

    #[test]
    fn a_wrong_key_cannot_open_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.db");
        let mut key = [7u8; 32];
        Store::open(&path, "me@tuta.com", &mut key).unwrap();
        drop(path);
        let mut wrong = [9u8; 32];
        let path = dir.path().join("cache.db");
        assert!(Store::open(&path, "me@tuta.com", &mut wrong).is_err());
    }

    #[test]
    fn fts_query_quotes_hostile_input() {
        assert_eq!(fts_query("hello AND *"), "\"hello\"* AND \"AND\"*");
        assert_eq!(fts_query(""), "");
    }
}
