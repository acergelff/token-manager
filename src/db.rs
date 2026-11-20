use crate::model::{Entry, EntryDraft, Kind};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

const CREATE_SQL: &str = "
CREATE TABLE IF NOT EXISTS entries (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    kind       TEXT NOT NULL,
    value      TEXT NOT NULL,
    service    TEXT NOT NULL DEFAULT '',
    notes      TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);";

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(CREATE_SQL)?;
        Ok(Self { conn })
    }

    pub fn insert(&self, d: &EntryDraft) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO entries (name, kind, value, service, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now','localtime'), datetime('now','localtime'))",
            params![d.name, d.kind.as_str(), d.value, d.service, d.notes],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list(&self, _search: &str, _kind: Option<Kind>) -> rusqlite::Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, value, service, notes, created_at, updated_at
             FROM entries ORDER BY updated_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([], row_to_entry)?;
        rows.collect()
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    let kind_str: String = row.get(2)?;
    Ok(Entry {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: Kind::from_str(&kind_str).unwrap_or(Kind::ApiKey),
        value: row.get(3)?,
        service: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(name: &str, kind: Kind, value: &str) -> EntryDraft {
        EntryDraft {
            name: name.into(),
            kind,
            value: value.into(),
            service: String::new(),
            notes: String::new(),
        }
    }

    #[test]
    fn insert_then_list_returns_entry() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert(&draft("OpenAI", Kind::ApiKey, "sk-123")).unwrap();
        let entries = db.list("", None).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.id, id);
        assert_eq!(e.name, "OpenAI");
        assert_eq!(e.kind, Kind::ApiKey);
        assert_eq!(e.value, "sk-123");
        assert!(!e.created_at.is_empty());
        assert!(!e.updated_at.is_empty());
    }

    #[test]
    fn list_orders_newest_first() {
        let db = Db::open_in_memory().unwrap();
        db.insert(&draft("first", Kind::Secret, "a")).unwrap();
        db.insert(&draft("second", Kind::Serial, "b")).unwrap();
        let entries = db.list("", None).unwrap();
        assert_eq!(entries[0].name, "second");
        assert_eq!(entries[1].name, "first");
    }

    #[test]
    fn empty_db_lists_nothing() {
        let db = Db::open_in_memory().unwrap();
        assert!(db.list("", None).unwrap().is_empty());
    }
}
