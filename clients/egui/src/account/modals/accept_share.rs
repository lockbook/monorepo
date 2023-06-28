use std::sync::Arc;

use eframe::egui;
use egui_winit::egui::Color32;
use lb::File;

use crate::{theme::Icon, widgets::Button};

pub struct AcceptShareModal {
    requests: Vec<lb::File>,
    username: String,
    err_msg: Option<String>,
}

impl AcceptShareModal {
    pub fn new(core: &Arc<lb::Core>) -> Self {
        Self {
            requests: core.get_pending_shares().unwrap_or_default(),
            username: core.get_account().unwrap().username,
            err_msg: None,
        }
    }
}

impl super::Modal for AcceptShareModal {
    type Response = Option<()>;

    fn title(&self) -> &str {
        "Incoming Share Requests"
    }

    fn show(&mut self, ui: &mut egui::Ui) -> Self::Response {
        ui.set_max_height(400.0);
        egui::ScrollArea::vertical()
            // .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                self.requests.iter().for_each(|req| {
                    sharer_info(ui, req, self.username.clone());
                });
            });

        None
    }
}
fn sharer_info(ui: &mut egui::Ui, req: &File, username: String) {
    let sharer = req
        .shares
        .iter()
        .find(|s| s.shared_with == username)
        .unwrap()
        .clone()
        .shared_by;

    egui::Frame::default()
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke { width: 0.1, color: ui.visuals().text_color() })
        .inner_margin(egui::Margin::same(15.0))
        .outer_margin(egui::Margin::same(15.0))
        .rounding(egui::Rounding::same(5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let icon = match req.file_type {
                    lb::FileType::Document => Icon::DOC_TEXT,
                    lb::FileType::Folder => Icon::FOLDER,
                    lb::FileType::Link { target: _ } => todo!(),
                };

                icon.size(40.0).show(ui);
                ui.vertical(|ui| {
                    ui.label(&req.name);

                    ui.label(
                        egui::RichText::new(format!("shared by {}", sharer))
                            .size(15.0)
                            // todo: use a color defined in the theme (ui.visuals)
                            .color(egui::Color32::GRAY),
                    );
                });

                let others_with_access = req
                    .shares
                    .iter()
                    .filter(|f| f.shared_with != sharer && f.shared_with != username)
                    .map(|s| s.shared_with.clone())
                    .collect::<Vec<String>>();

                if !others_with_access.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        Icon::GROUP
                            .color(egui::Color32::GRAY)
                            .show(ui)
                            .on_hover_text(format!("shared with {}", others_with_access.join(", ")))
                    });
                }
            });

            ui.add_space(30.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                //reject share
                ui.spacing_mut().button_padding = egui::vec2(25.0, 5.0);
                ui.button("Accept");
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.spacing_mut().button_padding = egui::vec2(5.0, 5.0);

                    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.inactive.fg_stroke =
                        egui::Stroke { color: egui::Color32::GRAY, ..Default::default() };

                    Button::default().text("Del ").icon(&Icon::DELETE).show(ui);
                });
            })
        });
}
