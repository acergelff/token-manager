# TokenManager 设计文档

日期：2026-11-20
状态：已获用户批准

## 概述

一个 Windows 桌面小工具，用 Rust + egui 编写，管理各种 API Key、密钥、序列号。
数据用 SQLite 明文保存（本期不做加密）。中文界面，单窗口。

## 技术选型

- GUI：`eframe` / `egui`（含 `egui_extras` 表格组件）
- 数据库：`rusqlite`，启用 `bundled` feature（自带 SQLite 源码编译，无需系统安装）
- 同步数据库调用：本地小数据量毫秒级完成，不引入异步运行时
- 依赖共 3 个：`eframe`、`rusqlite`、`egui_extras`

## 架构与文件结构

单 crate（binary，名为 `token_manager`），4 个源文件：

| 文件 | 职责 |
|------|------|
| `src/main.rs` | eframe 启动、加载中文字体、定位数据库路径 |
| `src/model.rs` | `Entry` 结构体、`Kind` 枚举 |
| `src/db.rs` | 打开数据库、建表、CRUD、搜索筛选 |
| `src/app.rs` | UI 状态与绘制 |

### 中文字体

egui 默认字体不含 CJK 字形。启动时从 `C:\Windows\Fonts\msyh.ttc`（微软雅黑）
读取并插入 egui 字体族首位。文件不存在时回退默认字体并在状态栏提示
"未找到中文字体，中文可能显示为方框"。

## 数据模型与存储

### 数据库位置（便携模式）

`tokens.db` 放在 **exe 同目录**，通过 `std::env::current_exe()` 的父目录定位。
整个程序文件夹拷走即完成迁移。开发期 `cargo run` 时数据库落在 `target/debug/`。
`current_exe()` 失败时启动报错并退出。

### 表结构

```sql
CREATE TABLE IF NOT EXISTS entries (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,              -- 名称
    kind       TEXT NOT NULL,              -- 'api_key' | 'secret' | 'serial'
    value      TEXT NOT NULL,              -- 密钥值（明文）
    service    TEXT NOT NULL DEFAULT '',   -- 所属服务/平台
    notes      TEXT NOT NULL DEFAULT '',   -- 备注
    created_at TEXT NOT NULL,              -- datetime('now','localtime')
    updated_at TEXT NOT NULL
);
```

时间戳由 SQLite 生成，不引入 chrono。

### Rust 模型

```rust
enum Kind { ApiKey, Secret, Serial }  // 与 TEXT 值互转，显示为 "API Key"/"密钥"/"序列号"

struct Entry {
    id: i64,
    name: String,
    kind: Kind,
    value: String,
    service: String,
    notes: String,
    created_at: String,
    updated_at: String,
}
```

## UI 与交互

单窗口，中文界面，自上而下：

1. **工具栏**：搜索框（实时匹配名称/服务/备注，不区分大小写）、
   类型筛选下拉（全部/API Key/密钥/序列号）、"新建"按钮。
2. **表格**（`egui_extras::TableBuilder`）：列 = 名称、类型、服务、密钥值、操作。
   - 密钥值默认显示 `••••••`，每行 👁 按钮切换显示/隐藏（状态仅存内存，不落库）
   - 操作列：复制、编辑、删除
   - 复制用 `egui::Context::copy_text`，状态栏提示"已复制 <名称>"
   - 删除弹确认窗口，确认后才执行
3. **新建/编辑**：浮动表单窗口（`egui::Window`），字段 = 名称、类型、密钥值、
   服务、备注。名称与密钥值为必填，为空时禁止保存并提示。
4. **状态栏**：记录条数、最近操作提示、错误信息。

搜索/筛选变化或增删改后重新查询刷新列表。

## 错误处理

- 数据库打开/建表失败：启动即报错退出，给出明确信息。
- 运行期 CRUD 错误：不崩溃，错误文本显示在状态栏。
- 表单校验失败：窗口内提示，不关闭。

## 测试

- `db.rs` 单元测试：用内存 SQLite（`Connection::open_in_memory`）覆盖
  建表、插入、查询、更新、删除、搜索匹配、类型筛选。
- UI：`cargo run` 实际运行，手动验证新建/编辑/删除/复制/显隐/搜索 golden path。

## 明确不做（本期）

- 加密存储
- 导入/导出
- 主题切换
- 标签、过期日期、账号等扩展字段
- 跨平台打包（仅面向 Windows 本机使用）