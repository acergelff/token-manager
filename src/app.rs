use crate::db::Db;
use crate::model::{Entry, Kind};
use eframe::egui;
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| {
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

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("共 {} 条", self.entries.len()));
                if !self.status.is_empty() {
                    ui.separator();
                    ui.label(&self.status);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
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
