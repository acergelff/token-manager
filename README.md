# TokenManager

一个本地桌面小工具，用于集中管理各种 **API Key、密钥、序列号**。Rust + egui 编写，SQLite 存储，绿色便携——整个文件夹拷走即完成迁移。

## 功能

- 📋 记录管理：名称、类型（API Key / 密钥 / 序列号）、密钥值、所属服务、备注
- 🔍 实时搜索（匹配名称 / 服务 / 备注）+ 按类型筛选
- 👁 密钥值默认掩码显示（`••••••`），每行可切换显示 / 隐藏
- 📎 一键复制到剪贴板
- ✏️ 新建 / 编辑浮动表单，名称与密钥值必填校验
- 🗑 删除二次确认
- 🇨🇳 中文界面（自动加载系统中文字体）

## 快速开始

### 直接运行（推荐）

从 [Releases](https://github.com/acergelff/token-manager/releases) 下载最新版压缩包，解压后双击 `token_manager.exe` 即可。

### 从源码构建

需要 Rust 工具链（1.96+，含 cargo）：

```bash
git clone https://github.com/acergelff/token-manager.git
cd token-manager
cargo run --release
```

首次编译较慢（eframe 依赖较多 + SQLite 源码编译），请耐心等待。

## 数据存储

- 数据库文件 `tokens.db` 自动创建在 **exe 同目录**
- 备份 / 迁移：复制整个程序文件夹（exe + tokens.db）即可
- 开发期 `cargo run` 时数据库位于 `target/debug/` 或 `target/release/`

## 安全说明

⚠️ **当前版本为明文存储，不做加密。** 请注意：

- 不要将程序文件夹放入网盘同步目录或分享给他人
- 不要在公用电脑上使用
- 加密存储计划在后续版本考虑

## 技术栈

| 组件 | 选型 |
|------|------|
| GUI | [eframe / egui](https://github.com/emilk/egui) 0.37 |
| 数据库 | [rusqlite](https://github.com/rusqlite/rusqlite) 0.43（bundled SQLite） |
| 语言 | Rust（edition 2021） |

## 开发

```bash
cargo test    # 单元测试（模型 + 内存 SQLite 数据层）
cargo build --release
```

源码结构：`src/model.rs`（数据模型）、`src/db.rs`（数据库层）、`src/app.rs`（界面）、`src/main.rs`（启动 / 字体 / 路径）。

## 许可证

[MIT](LICENSE)