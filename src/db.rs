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

    pub fn update(&self, id: i64, d: &EntryDraft) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE entries
             SET name=?1, kind=?2, value=?3, service=?4, notes=?5,
                 updated_at=datetime('now','localtime')
             WHERE id=?6",
            params![d.name, d.kind.as_str(), d.value, d.service, d.notes, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM entries WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list(&self, search: &str, kind: Option<Kind>) -> rusqlite::Result<Vec<Entry>> {
        let pattern = format!("%{search}%");
        let mut sql = String::from(
            "SELECT id, name, kind, value, service, notes, created_at, updated_at
             FROM entries WHERE (name LIKE ?1 OR service LIKE ?1 OR notes LIKE ?1)",
        );
        if kind.is_some() {
            sql.push_str(" AND kind = ?2");
        }
        sql.push_str(" ORDER BY updated_at DESC, id DESC");
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if let Some(k) = kind {
            stmt.query_map(params![pattern, k.as_str()], row_to_entry)?
        } else {
            stmt.query_map(params![pattern], row_to_entry)?
        };
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

    #[test]
    fn update_changes_fields() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert(&draft("old", Kind::ApiKey, "v1")).unwrap();
        let mut d = draft("new", Kind::Secret, "v2");
        d.service = "Azure".into();
        d.notes = "备注".into();
        db.update(id, &d).unwrap();
        let entries = db.list("", None).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.name, "new");
        assert_eq!(e.kind, Kind::Secret);
        assert_eq!(e.value, "v2");
        assert_eq!(e.service, "Azure");
        assert_eq!(e.notes, "备注");
        assert!(e.updated_at >= e.created_at);
    }

    #[test]
    fn delete_removes_entry() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert(&draft("gone", Kind::Serial, "x")).unwrap();
        db.delete(id).unwrap();
        assert!(db.list("", None).unwrap().is_empty());
    }

    #[test]
    fn search_matches_name_service_notes() {
        let db = Db::open_in_memory().unwrap();
        let mut a = draft("OpenAI 主账号", Kind::ApiKey, "sk-a");
        a.service = "openai.com".into();
        let mut b = draft("公司授权", Kind::Serial, "SN-1");
        b.notes = "给 OpenAI 用的".into();
        db.insert(&a).unwrap();
        db.insert(&b).unwrap();
        db.insert(&draft("无关", Kind::Secret, "z")).unwrap();
        let hits = db.list("openai", None).unwrap();
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn kind_filter_narrow_results() {
        let db = Db::open_in_memory().unwrap();
        db.insert(&draft("k1", Kind::ApiKey, "a")).unwrap();
        db.insert(&draft("k2", Kind::ApiKey, "b")).unwrap();
        db.insert(&draft("s1", Kind::Serial, "c")).unwrap();
        let hits = db.list("", Some(Kind::ApiKey)).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|e| e.kind == Kind::ApiKey));
    }

    #[test]
    fn search_and_kind_filter_combine() {
        let db = Db::open_in_memory().unwrap();
        db.insert(&draft("alpha", Kind::ApiKey, "a")).unwrap();
        db.insert(&draft("alpha", Kind::Serial, "b")).unwrap();
        let hits = db.list("alpha", Some(Kind::Serial)).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, Kind::Serial);
    }
}
