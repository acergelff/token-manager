mod app;
mod db;
mod model;

use eframe::egui;
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
            let mut defs = egui::FontDefinitions::default();
            defs.font_data
                .insert("chinese".to_owned(), egui::FontData::from_owned(bytes).into());
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                defs.families
                    .get_mut(&family)
                    .unwrap()
                    .insert(0, "chinese".to_owned());
            }
            ctx.set_fonts(defs);
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
