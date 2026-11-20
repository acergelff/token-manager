# TokenManager 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个 Rust+egui 的 Windows 桌面工具，管理 API Key/密钥/序列号，SQLite 明文存储于 exe 同目录。

**Architecture:** 单 crate 四文件：`model.rs`（数据模型）、`db.rs`（rusqlite 同步 CRUD/搜索）、`app.rs`（egui 界面状态与绘制）、`main.rs`（启动、中文字体、数据库定位）。UI 每帧直接读内存中的条目列表，增删改/搜索后重新查询刷新。

**Tech Stack:** Rust (edition 2021)、eframe 0.37、egui_extras 0.37、rusqlite 0.43 (bundled)。

**Spec:** `docs/superpowers/specs/2026-11-20-token-manager-design.md`

## Global Constraints

- 仅面向 Windows；界面文案全部中文。
- 数据库文件 `tokens.db` 必须位于 exe 同目录（`std::env::current_exe()` 父目录）。
- 明文存储，本期不做加密、不做导入导出、不做主题切换。
- 依赖仅 3 个：`eframe = "0.37"`、`egui_extras = "0.37"`、`rusqlite = { version = "0.43", features = ["bundled"] }`。
- 中文字体：egui 的 ab_glyph 后端不支持 .ttc；按顺序尝试 `C:\Windows\Fonts` 下 `simhei.ttf`、`Deng.ttf`、`simkai.ttf`、`simfang.ttf`，全部缺失时状态栏提示"未找到中文字体，中文可能显示为方框"。
- 字段：名称、类型（API Key/密钥/序列号）、密钥值、服务、备注、created_at/updated_at（SQLite `datetime('now','localtime')` 生成）。
- 名称与密钥值必填；删除需确认；密钥值默认显示 `••••••`。
- 每个任务结束提交一次 git。工作目录 `C:\Users\acerg\workspace\TokenManager`（已是 git 仓库，master 分支）。
- Windows + Git Bash 环境；cargo 命令直接在仓库根目录运行。

---

### Task 1: 项目脚手架与数据模型

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `src/main.rs`（临时桩，Task 4 替换）
- Create: `src/model.rs`（含单元测试）

**Interfaces:**
- Consumes: 无
- Produces:
  - `Kind` 枚举：`Kind::ApiKey | Kind::Secret | Kind::Serial`，`Copy + Clone + PartialEq + Eq + Debug + Default`（默认 `ApiKey`）
  - `Kind::ALL: [Kind; 3]`、`Kind::as_str(self) -> &'static str`（`"api_key"/"secret"/"serial"`）、`Kind::from_str(&str) -> Option<Kind>`、`Kind::label(self) -> &'static str`（`"API Key"/"密钥"/"序列号"`）
  - `Entry { id: i64, name: String, kind: Kind, value: String, service: String, notes: String, created_at: String, updated_at: String }`（`Clone + Debug`）
  - `EntryDraft { name: String, kind: Kind, value: String, service: String, notes: String }`（`Clone + Debug + Default`）

- [ ] **Step 1: 创建 Cargo.toml、.gitignore、main.rs 桩**

`Cargo.toml`（本任务只加 rusqlite，eframe 在 Task 4 加入，避免早期编译过慢）：

```toml
[package]
name = "token_manager"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.43", features = ["bundled"] }
```

`.gitignore`：

```
/target
*.db
*.db-journal
```

`src/main.rs`（桩）：

```rust
mod model;

fn main() {}
```

- [ ] **Step 2: 写 model.rs 的失败测试**

创建 `src/model.rs`，先只写测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_str_roundtrip() {
        for k in Kind::ALL {
            assert_eq!(Kind::from_str(k.as_str()), Some(k));
        }
    }

    #[test]
    fn kind_from_str_unknown_is_none() {
        assert_eq!(Kind::from_str("bogus"), None);
    }

    #[test]
    fn kind_labels_are_chinese() {
        assert_eq!(Kind::ApiKey.label(), "API Key");
        assert_eq!(Kind::Secret.label(), "密钥");
        assert_eq!(Kind::Serial.label(), "序列号");
    }

    #[test]
    fn draft_default_kind_is_api_key() {
        assert_eq!(EntryDraft::default().kind, Kind::ApiKey);
    }
}
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cargo test`
Expected: 编译失败，`Kind`/`EntryDraft` 未定义。

- [ ] **Step 4: 实现 model.rs**

在测试模块之前写入：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    ApiKey,
    Secret,
    Serial,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::ApiKey, Kind::Secret, Kind::Serial];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::ApiKey => "api_key",
            Kind::Secret => "secret",
            Kind::Serial => "serial",
        }
    }

    pub fn from_str(s: &str) -> Option<Kind> {
        match s {
            "api_key" => Some(Kind::ApiKey),
            "secret" => Some(Kind::Secret),
            "serial" => Some(Kind::Serial),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::ApiKey => "API Key",
            Kind::Secret => "密钥",
            Kind::Serial => "序列号",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub name: String,
    pub kind: Kind,
    pub value: String,
    pub service: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct EntryDraft {
    pub name: String,
    pub kind: Kind,
    pub value: String,
    pub service: String,
    pub notes: String,
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test`
Expected: 4 个测试全部 PASS。

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock .gitignore src/main.rs src/model.rs
git commit -m "feat: 项目脚手架与数据模型（Kind/Entry/EntryDraft）"
```

---

### Task 2: 数据库打开、建表、插入、列表

**Files:**
- Create: `src/db.rs`（含单元测试）
- Modify: `src/main.rs`（加 `mod db;`）

**Interfaces:**
- Consumes: Task 1 的 `Entry`、`EntryDraft`、`Kind`
- Produces:
  - `Db::open(path: &std::path::Path) -> rusqlite::Result<Db>`（打开文件并建表）
  - `Db::open_in_memory() -> rusqlite::Result<Db>`（测试用）
  - `Db::insert(&self, d: &EntryDraft) -> rusqlite::Result<i64>`（返回新 id）
  - `Db::list(&self, search: &str, kind: Option<Kind>) -> rusqlite::Result<Vec<Entry>>`（本任务先实现为忽略参数的全量查询，Task 3 补搜索筛选；排序 `updated_at DESC, id DESC`）

- [ ] **Step 1: 写失败测试**

创建 `src/db.rs`，先只写测试：

```rust
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
```

并在 `src/main.rs` 顶部加 `mod db;`。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test`
Expected: 编译失败，`Db` 未定义。

- [ ] **Step 3: 实现 db.rs**

在测试模块之前写入：

```rust
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
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test`
Expected: 7 个测试全部 PASS（model 4 + db 3）。

- [ ] **Step 5: Commit**

```bash
git add src/db.rs src/main.rs
git commit -m "feat: SQLite 建表、插入与列表查询"
```

---

### Task 3: 更新、删除、搜索与类型筛选

**Files:**
- Modify: `src/db.rs`（替换 `list` 实现，新增 `update`/`delete`，追加测试）

**Interfaces:**
- Consumes: Task 2 的 `Db`、`row_to_entry`
- Produces:
  - `Db::update(&self, id: i64, d: &EntryDraft) -> rusqlite::Result<()>`（刷新 updated_at）
  - `Db::delete(&self, id: i64) -> rusqlite::Result<()>`
  - `Db::list(&self, search: &str, kind: Option<Kind>) -> rusqlite::Result<Vec<Entry>>`（完整语义：search 对 name/service/notes 做 `LIKE %search%` 匹配，空串匹配全部；kind 为 Some 时追加 `AND kind = ?`；排序不变）

- [ ] **Step 1: 追加失败测试**

在 `src/db.rs` 的 `mod tests` 内追加（保留已有测试与 `draft` 辅助函数）：

```rust
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
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test`
Expected: 编译失败，`update`/`delete` 未定义。

- [ ] **Step 3: 实现 update/delete，替换 list**

在 `impl Db` 中追加：

```rust
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
```

将 Task 2 的 `list` 整体替换为：

```rust
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
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test`
Expected: 12 个测试全部 PASS。

- [ ] **Step 5: Commit**

```bash
git add src/db.rs
git commit -m "feat: 更新、删除、搜索与类型筛选"
```

---

### Task 4: 只读界面骨架（工具栏 + 表格 + 状态栏）

**Files:**
- Modify: `Cargo.toml`（加 eframe、egui_extras）
- Rewrite: `src/main.rs`（启动、字体、数据库定位）
- Create: `src/app.rs`

**Interfaces:**
- Consumes: `Db::open`、`Db::list`、`Kind::ALL/label`、`Entry`
- Produces:
  - `app::TokenApp::new(db: Db, font_warning: Option<String>) -> TokenApp`（实现 `eframe::App`）
  - `TokenApp` 字段（Task 5 在其上扩展）：`db: Db`、`entries: Vec<Entry>`、`search: String`、`filter: Option<Kind>`、`status: String`；私有方法 `refresh(&mut self)`
  - `main.rs` 中 `load_chinese_font(ctx: &egui::Context) -> Option<String>`（成功返回 None，失败返回警告文案）

- [ ] **Step 1: 添加 GUI 依赖**

`Cargo.toml` 的 `[dependencies]` 追加：

```toml
eframe = "0.37"
egui_extras = "0.37"
```

- [ ] **Step 2: 重写 src/main.rs**

```rust
mod app;
mod db;
mod model;

use std::path::PathBuf;

fn db_path() -> PathBuf {
    let exe = std::env::current_exe().expect("无法定位 exe 路径");
    exe.parent().expect("exe 无父目录").join("tokens.db")
}

fn load_chinese_font(ctx: &egui::Context) -> Option<String> {
    const CANDIDATES: [&str; 4] = ["simhei.ttf", "Deng.ttf", "simkai.ttf", "simfang.ttf"];
    let fonts_dir = PathBuf::from(r"C:\Windows\Fonts");
    for name in CANDIDATES {
        if let Ok(bytes) = std::fs::read(fonts_dir.join(name)) {
            ctx.fonts_mut(|fonts| {
                let mut defs = egui::FontDefinitions::default();
                defs.font_data
                    .insert("chinese".to_owned(), egui::FontData::from_owned(bytes).into());
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    defs.families
                        .get_mut(&family)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                }
                fonts.set_fonts(defs);
            });
            return None;
        }
    }
    Some("未找到中文字体，中文可能显示为方框".to_owned())
}

fn main() -> eframe::Result<()> {
    let path = db_path();
    let db = db::Db::open(&path)
        .unwrap_or_else(|e| panic!("无法打开数据库 {}：{e}", path.display()));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "TokenManager 密钥管理",
        options,
        Box::new(move |cc| {
            let warning = load_chinese_font(&cc.egui_ctx);
            Ok(Box::new(app::TokenApp::new(db, warning)))
        }),
    )
}
```

- [ ] **Step 3: 创建 src/app.rs（只读界面）**

```rust
use crate::db::Db;
use crate::model::{Entry, Kind};
use egui_extras::{Column, TableBuilder};

pub struct TokenApp {
    db: Db,
    entries: Vec<Entry>,
    search: String,
    filter: Option<Kind>,
    status: String,
}

impl TokenApp {
    pub fn new(db: Db, font_warning: Option<String>) -> Self {
        let mut app = Self {
            db,
            entries: Vec::new(),
            search: String::new(),
            filter: None,
            status: font_warning.unwrap_or_default(),
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        match self.db.list(&self.search, self.filter) {
            Ok(entries) => self.entries = entries,
            Err(e) => self.status = format!("查询失败：{e}"),
        }
    }
}

impl eframe::App for TokenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("搜索：");
                if ui.text_edit_singleline(&mut self.search).changed() {
                    self.refresh();
                }
                ui.label("类型：");
                let selected = self.filter.map(|k| k.label()).unwrap_or("全部");
                egui::ComboBox::from_id_salt("kind_filter")
                    .selected_text(selected)
                    .show_ui(ui, |ui| {
                        let mut changed = ui.selectable_value(&mut self.filter, None, "全部").changed();
                        for k in Kind::ALL {
                            changed |= ui.selectable_value(&mut self.filter, Some(k), k.label()).changed();
                        }
                        if changed {
                            self.refresh();
                        }
                    });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("共 {} 条", self.entries.len()));
                if !self.status.is_empty() {
                    ui.separator();
                    ui.label(&self.status);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.entries.is_empty() {
                ui.label("暂无记录");
                return;
            }
            let entries = self.entries.clone();
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto().at_least(140.0))
                .column(Column::auto().at_least(80.0))
                .column(Column::auto().at_least(140.0))
                .column(Column::remainder())
                .header(24.0, |mut h| {
                    h.col(|ui| { ui.strong("名称"); });
                    h.col(|ui| { ui.strong("类型"); });
                    h.col(|ui| { ui.strong("服务"); });
                    h.col(|ui| { ui.strong("密钥值"); });
                })
                .body(|body| {
                    body.rows(26.0, entries.len(), |mut row| {
                        let e = &entries[row.index()];
                        row.col(|ui| { ui.label(&e.name); });
                        row.col(|ui| { ui.label(e.kind.label()); });
                        row.col(|ui| { ui.label(&e.service); });
                        row.col(|ui| { ui.label("••••••"); });
                    });
                });
        });
    }
}
```

- [ ] **Step 4: 编译与单元测试**

Run: `cargo test`
Expected: 编译通过（首次编译 eframe 较慢，属正常），12 个测试 PASS。

- [ ] **Step 5: 运行并手动验证只读界面**

Run: `cargo run`（窗口打开后手动检查，然后关闭）

验证清单：
1. 窗口标题"TokenManager 密钥管理"，中文正常显示（非方框）。
2. 空库显示"暂无记录"，状态栏"共 0 条"。
3. 用任意 SQLite 工具或临时代码向 `target/debug/tokens.db` 插入一行后重启程序，表格显示该行且密钥值为 `••••••`。（若不便插入，可跳过此条，Task 5 有新建功能后补验。）
4. `target/debug/` 下出现 `tokens.db` 文件。

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/app.rs
git commit -m "feat: egui 只读界面（搜索工具栏、表格、状态栏）与中文字体加载"
```

---

### Task 5: 完整交互（显隐、复制、新建/编辑、删除确认）

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: Task 4 的 `TokenApp` 全部字段与 `refresh`；`Db::insert/update/delete`；`EntryDraft`
- Produces: 完整可用的应用。新增私有类型 `EditorState { id: Option<i64>, draft: EntryDraft, error: String }`；`TokenApp` 新增字段 `revealed: std::collections::HashSet<i64>`、`editor: Option<EditorState>`、`confirm_delete: Option<Entry>`

- [ ] **Step 1: 扩展 TokenApp 状态**

`src/app.rs` 顶部加 `use std::collections::HashSet;` 与 `use crate::model::EntryDraft;`（合并进已有 use）。

在 `TokenApp` 定义前加：

```rust
struct EditorState {
    id: Option<i64>,
    draft: EntryDraft,
    error: String,
}
```

`TokenApp` 增加三个字段：

```rust
    revealed: HashSet<i64>,
    editor: Option<EditorState>,
    confirm_delete: Option<Entry>,
```

`new` 中初始化：`revealed: HashSet::new(), editor: None, confirm_delete: None,`。

- [ ] **Step 2: 工具栏加"新建"按钮**

在工具栏 `ui.horizontal` 闭包末尾（ComboBox 之后）追加：

```rust
                if ui.button("新建").clicked() {
                    self.editor = Some(EditorState {
                        id: None,
                        draft: EntryDraft::default(),
                        error: String::new(),
                    });
                }
```

- [ ] **Step 3: 表格加操作列与显隐/复制**

表格构建处：在最后一个 `.column(Column::remainder())` 之后追加一列 `.column(Column::auto().at_least(220.0))`；header 闭包末尾追加 `h.col(|ui| { ui.strong("操作"); });`。

body 行闭包中，密钥值列替换为：

```rust
                        row.col(|ui| {
                            if self.revealed.contains(&e.id) {
                                ui.label(&e.value);
                            } else {
                                ui.label("••••••");
                            }
                        });
```

并在其后追加操作列：

```rust
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                let revealed = self.revealed.contains(&e.id);
                                if ui.button(if revealed { "隐藏" } else { "显示" }).clicked() {
                                    if revealed {
                                        self.revealed.remove(&e.id);
                                    } else {
                                        self.revealed.insert(&e.id);
                                    }
                                }
                                if ui.button("复制").clicked() {
                                    ui.ctx().copy_text(e.value.clone());
                                    self.status = format!("已复制 {}", e.name);
                                }
                                if ui.button("编辑").clicked() {
                                    self.editor = Some(EditorState {
                                        id: Some(e.id),
                                        draft: EntryDraft {
                                            name: e.name.clone(),
                                            kind: e.kind,
                                            value: e.value.clone(),
                                            service: e.service.clone(),
                                            notes: e.notes.clone(),
                                        },
                                        error: String::new(),
                                    });
                                }
                                if ui.button("删除").clicked() {
                                    self.confirm_delete = Some(e.clone());
                                }
                            });
                        });
```

- [ ] **Step 4: 新建/编辑表单窗口**

在 `update` 方法末尾（CentralPanel 之后）追加：

```rust
        if let Some(mut editor) = self.editor.take() {
            let mut open = true;
            let mut save = false;
            egui::Window::new(if editor.id.is_some() { "编辑条目" } else { "新建条目" })
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    egui::Grid::new("editor_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                        ui.label("名称：");
                        ui.text_edit_singleline(&mut editor.draft.name);
                        ui.end_row();
                        ui.label("类型：");
                        egui::ComboBox::from_id_salt("editor_kind")
                            .selected_text(editor.draft.kind.label())
                            .show_ui(ui, |ui| {
                                for k in Kind::ALL {
                                    ui.selectable_value(&mut editor.draft.kind, k, k.label());
                                }
                            });
                        ui.end_row();
                        ui.label("密钥值：");
                        ui.text_edit_singleline(&mut editor.draft.value);
                        ui.end_row();
                        ui.label("服务：");
                        ui.text_edit_singleline(&mut editor.draft.service);
                        ui.end_row();
                        ui.label("备注：");
                        ui.text_edit_singleline(&mut editor.draft.notes);
                        ui.end_row();
                    });
                    if !editor.error.is_empty() {
                        ui.colored_label(egui::Color32::RED, &editor.error);
                    }
                    ui.horizontal(|ui| {
                        if ui.button("保存").clicked() {
                            save = true;
                        }
                        if ui.button("取消").clicked() {
                            open = false;
                        }
                    });
                });
            if save {
                if editor.draft.name.trim().is_empty() || editor.draft.value.trim().is_empty() {
                    editor.error = "名称和密钥值不能为空".to_owned();
                    self.editor = Some(editor);
                } else {
                    let result = match editor.id {
                        Some(id) => self.db.update(id, &editor.draft),
                        None => self.db.insert(&editor.draft).map(|_| ()),
                    };
                    match result {
                        Ok(()) => {
                            self.status = "已保存".to_owned();
                            self.refresh();
                        }
                        Err(e) => {
                            editor.error = format!("保存失败：{e}");
                            self.editor = Some(editor);
                        }
                    }
                }
            } else if open {
                self.editor = Some(editor);
            }
        }
```

- [ ] **Step 5: 删除确认窗口**

紧接其后追加：

```rust
        if let Some(entry) = self.confirm_delete.take() {
            let mut open = true;
            let mut confirmed = false;
            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(format!("确定删除「{}」吗？此操作不可恢复。", entry.name));
                    ui.horizontal(|ui| {
                        if ui.button("删除").clicked() {
                            confirmed = true;
                        }
                        if ui.button("取消").clicked() {
                            open = false;
                        }
                    });
                });
            if confirmed {
                match self.db.delete(entry.id) {
                    Ok(()) => {
                        self.revealed.remove(&entry.id);
                        self.status = format!("已删除 {}", entry.name);
                        self.refresh();
                    }
                    Err(e) => self.status = format!("删除失败：{e}"),
                }
            } else if open {
                self.confirm_delete = Some(entry);
            }
        }
```

- [ ] **Step 6: 编译与单元测试**

Run: `cargo test`
Expected: 编译通过，12 个测试 PASS。

- [ ] **Step 7: 运行并手动验证完整交互**

Run: `cargo run`

验证清单（golden path + 边界）：
1. 新建：点"新建"，填名称"OpenAI"、类型 API Key、密钥值"sk-test-123"、服务"openai.com"、备注"测试"，保存 → 表格出现该行，状态栏"已保存"，"共 1 条"。
2. 必填校验：新建时名称或密钥值留空点保存 → 红字"名称和密钥值不能为空"，窗口不关闭。
3. 显隐：点"显示"→ 明文可见；点"隐藏"→ 恢复 `••••••`。
4. 复制：点"复制"→ 状态栏"已复制 OpenAI"；打开记事本 Ctrl+V 得到 `sk-test-123`。
5. 编辑：点"编辑"，改备注后保存 → 表格更新。
6. 搜索：输入"openai"命中（匹配名称/服务/备注）；输入"zzz"显示"暂无记录"；清空恢复。
7. 筛选：类型选"序列号"→ 无结果；选"API Key"→ 命中。
8. 删除：点"删除"→ 确认窗口；点"取消"→ 记录仍在；再删除并确认 → 记录消失，状态栏"已删除 OpenAI"。
9. 持久化：再新建一条，关闭程序，重新 `cargo run` → 记录仍在。

- [ ] **Step 8: Commit**

```bash
git add src/app.rs
git commit -m "feat: 显隐/复制/新建编辑/删除确认完整交互"
```

---

### Task 6: Release 构建与便携性验证

**Files:**
- 无源码修改（仅构建与验证；如验证暴露问题则修复相应文件）

**Interfaces:**
- Consumes: 全部前序任务成果
- Produces: `target/release/token_manager.exe` + 同目录 `tokens.db` 的可分发文件夹

- [ ] **Step 1: Release 构建**

Run: `cargo build --release`
Expected: 编译成功，生成 `target/release/token_manager.exe`。

- [ ] **Step 2: 运行 release 版并验证便携性**

Run: `./target/release/token_manager.exe`（手动操作后关闭）

验证清单：
1. 新建一条记录并保存，关闭程序。
2. `target/release/` 下存在 `tokens.db`（与 exe 同目录）。
3. 重新启动 exe，记录仍在。
4. 将 `token_manager.exe` 与 `tokens.db` 一起复制到临时目录（如 `/tmp/tm-test/`）运行，记录仍在 → 迁移成立。验证后删除临时目录。

- [ ] **Step 3: 最终测试与提交**

Run: `cargo test`
Expected: 12 个测试 PASS。

若前面步骤无源码改动则无需新提交；有修复则：

```bash
git add -u
git commit -m "fix: release 验证中发现的问题"
```