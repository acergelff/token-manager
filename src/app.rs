use std::collections::HashSet;

use crate::db::Db;
use crate::model::{Entry, EntryDraft, Kind};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

struct EditorState {
    id: Option<i64>,
    draft: EntryDraft,
    error: String,
}

pub struct TokenApp {
    db: Db,
    entries: Vec<Entry>,
    search: String,
    filter: Option<Kind>,
    status: String,
    revealed: HashSet<i64>,
    editor: Option<EditorState>,
    confirm_delete: Option<Entry>,
}

impl TokenApp {
    pub fn new(db: Db, font_warning: Option<String>) -> Self {
        let mut app = Self {
            db,
            entries: Vec::new(),
            search: String::new(),
            filter: None,
            status: font_warning.unwrap_or_default(),
            revealed: HashSet::new(),
            editor: None,
            confirm_delete: None,
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
                if ui.button("新建").clicked() {
                    self.editor = Some(EditorState {
                        id: None,
                        draft: EntryDraft::default(),
                        error: String::new(),
                    });
                }
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
                .column(Column::auto().at_least(220.0))
                .header(24.0, |mut h| {
                    h.col(|ui| { ui.strong("名称"); });
                    h.col(|ui| { ui.strong("类型"); });
                    h.col(|ui| { ui.strong("服务"); });
                    h.col(|ui| { ui.strong("密钥值"); });
                    h.col(|ui| { ui.strong("操作"); });
                })
                .body(|body| {
                    body.rows(26.0, entries.len(), |mut row| {
                        let e = &entries[row.index()];
                        row.col(|ui| { ui.label(&e.name); });
                        row.col(|ui| { ui.label(e.kind.label()); });
                        row.col(|ui| { ui.label(&e.service); });
                        row.col(|ui| {
                            if self.revealed.contains(&e.id) {
                                ui.label(&e.value);
                            } else {
                                ui.label("••••••");
                            }
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                let revealed = self.revealed.contains(&e.id);
                                if ui.button(if revealed { "隐藏" } else { "显示" }).clicked() {
                                    if revealed {
                                        self.revealed.remove(&e.id);
                                    } else {
                                        self.revealed.insert(e.id);
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
                    });
                });
        });

        if let Some(mut editor) = self.editor.take() {
            let mut open = true;
            let mut save = false;
            let mut cancel = false;
            egui::Window::new(if editor.id.is_some() { "编辑条目" } else { "新建条目" })
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
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
                            cancel = true;
                        }
                    });
                });
            if cancel {
                open = false;
            }
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

        if let Some(entry) = self.confirm_delete.take() {
            let mut open = true;
            let mut confirmed = false;
            let mut cancel = false;
            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("确定删除「{}」吗？此操作不可恢复。", entry.name));
                    ui.horizontal(|ui| {
                        if ui.button("删除").clicked() {
                            confirmed = true;
                        }
                        if ui.button("取消").clicked() {
                            cancel = true;
                        }
                    });
                });
            if cancel {
                open = false;
            }
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
    }
}
