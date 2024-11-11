use egui::os::OperatingSystem;
use egui::{
    scroll_area, Color32, Context, EventFilter, Frame, Id, Margin, Rect, ScrollArea, Stroke, Ui,
};

use lb_rs::blocking::Lb;
use lb_rs::model::file_metadata::DocumentHmac;
use lb_rs::text::buffer::Buffer;
use lb_rs::text::offset_types::DocCharOffset;
use lb_rs::Uuid;

use crate::tab::ExtendedInput as _;
use crate::tab::{markdown_editor, ExtendedOutput as _};
use markdown_editor::layers::cursor;
use markdown_editor::layers::cursor::Cursor;
use markdown_editor::utils::Bound;
use markdown_editor::widgets::find::Find;
use markdown_editor::Event;

use serde::Serialize;
use std::time::{Duration, Instant};

use super::layers::ast::Ast;
use super::layers::canonical_input::CanonicalInput;
use super::layers::capture::Capture;
use super::layers::images::Images;
use super::layers::paragraphs::Paragraphs;
use super::layers::words::Words;

#[derive(Debug, Serialize, Default)]
pub struct Response {
    // state changes
    pub text_updated: bool,
    pub selection_updated: bool,
    pub scroll_updated: bool,

    // actions taken
    pub suggest_rename: Option<String>,
}

pub struct Editor {
    // dependencies
    pub core: Lb,
    pub client: reqwest::blocking::Client,

    // input
    pub file_id: Uuid,
    pub hmac: Option<DocumentHmac>,

    // layers
    pub canonical_input: CanonicalInput,
    pub cursor: Cursor,
    pub buffer: Buffer,
    pub paragraphs: Paragraphs,
    pub ast: Ast,
    pub capture: Capture,
    pub words: Words,
    pub images: Images,

    // widgets
    pub find: Find,

    pub virtual_keyboard_shown: bool,
    pub started_scrolling: Option<Instant>,
}

impl Editor {
    pub fn new(
        core: Lb,
        content: &str,
        file_id: Uuid,
        hmac: Option<DocumentHmac>,
        _needs_name: bool,
        _plaintext_mode: bool, // todo
    ) -> Self {
        Self {
            core,
            client: Default::default(),

            file_id,
            hmac,

            canonical_input: todo!(),
            cursor: Default::default(),
            buffer: content.into(),
            paragraphs: Default::default(),
            ast: Default::default(),
            capture: todo!(),
            words: todo!(),
            images: Default::default(),

            find: Default::default(),

            virtual_keyboard_shown: false,
            started_scrolling: None,
        }
    }

    pub fn past_first_frame(&self) -> bool {
        todo!()
    }

    pub fn reload(&mut self, text: String) {
        self.buffer.reload(text)
    }

    pub fn id(&self) -> Id {
        Id::new(self.file_id)
    }

    pub fn focus(&mut self, ctx: &Context) {
        ctx.memory_mut(|m| {
            m.request_focus(self.id());
        });
    }

    pub fn focus_lock(&mut self, ctx: &Context) {
        ctx.memory_mut(|m| {
            m.set_focus_lock_filter(
                self.id(),
                EventFilter {
                    tab: true,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: true,
                },
            );
        });
    }

    pub fn focused(&self, ctx: &Context) -> bool {
        ctx.memory(|m| m.has_focus(self.id()))
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let touch_mode = matches!(ui.ctx().os(), OperatingSystem::Android | OperatingSystem::IOS);
        ui.vertical(|ui| {
            if touch_mode {
                // touch devices: show find...
                let find_resp = self.find.show(&self.buffer, ui);
                if let Some(term) = find_resp.term {
                    ui.ctx()
                        .push_markdown_event(Event::Find { term, backwards: find_resp.backwards });
                }

                // ...then show editor content...
                let resp = ui
                    .allocate_ui(
                        egui::vec2(
                            ui.available_width(),
                            ui.available_height(), // todo: ... - MOBILE_TOOL_BAR_SIZE,
                        ),
                        |ui| self.show_inner(touch_mode, ui),
                    )
                    .inner;

                // ...then show toolbar at the bottom
                todo!();

                resp
            } else {
                // non-touch devices: show toolbar...
                todo!();

                // ...then show find...
                let find_resp = self.find.show(&self.buffer, ui);
                if let Some(term) = find_resp.term {
                    ui.ctx()
                        .push_markdown_event(Event::Find { term, backwards: find_resp.backwards });
                }

                // ...then show editor content
                self.show_inner(touch_mode, ui)
            }
        })
        .inner
    }

    pub fn show_inner(&mut self, touch_mode: bool, ui: &mut Ui) -> Response {
        if ui.style_mut().visuals.dark_mode {
            // #282828 raisin black
            ui.style_mut().visuals.code_bg_color = Color32::from_rgb(40, 40, 40);
        } else {
            // #F5F5F5 white smoke
            ui.style_mut().visuals.code_bg_color = Color32::from_rgb(245, 245, 245);
        }

        let scroll_area_id = ui.id().with("child").with(egui::Id::new(self.file_id));
        let prev_scroll_area_offset = ui.data_mut(|d| {
            d.get_persisted(scroll_area_id)
                .map(|s: scroll_area::State| s.offset)
                .unwrap_or_default()
        });

        if touch_mode {
            ui.ctx().style_mut(|style| {
                style.spacing.scroll = egui::style::ScrollStyle::solid();
            });
        }
        Frame::canvas(ui.style())
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                let scroll_area_output = ScrollArea::vertical()
                    .drag_to_scroll(touch_mode)
                    .id_source(self.file_id)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            // register widget id
                            ui.ctx().check_for_id_clash(self.id(), Rect::NOTHING, "");

                            Frame::canvas(ui.style())
                                .stroke(Stroke::NONE)
                                .inner_margin(Margin::same(15.))
                                .show(ui, |ui| self.show_inner_inner(ui, touch_mode))
                                .inner
                        })
                        .inner
                    });
                let mut resp = scroll_area_output.inner;

                resp.scroll_updated = scroll_area_output.state.offset != prev_scroll_area_offset;

                if resp.scroll_updated {
                    if self.started_scrolling.is_none() {
                        self.started_scrolling = Some(Instant::now());
                    }
                } else {
                    self.started_scrolling = None;
                }
                if self.started_scrolling.unwrap_or(Instant::now()).elapsed()
                    > Duration::from_millis(300)
                {
                    ui.ctx().set_virtual_keyboard_shown(false);
                }

                resp
            })
            .inner
    }

    fn show_inner_inner(&mut self, ui: &mut Ui, touch_mode: bool) -> Response {
        self.canonical_input(ui.ctx());
        // self.cursor();
        // self.buffer();
        self.paragraphs();
        self.ast();
        self.words();

        // if text_updated || selection_updated || self.capture.mark_changes_processed() {
        //     self.bounds.text = bounds::calc_text(
        //         &self.ast,
        //         &self.bounds.ast,
        //         &self.appearance,
        //         &self.buffer.current.segs,
        //         self.buffer.current.selection,
        //         ui.ctx().input(|i| i.pointer.primary_down()),
        //         &self.capture,
        //     );
        //     self.bounds.links = bounds::calc_links(&self.buffer, &self.bounds.text, &self.ast);
        // }
        // if text_updated || selection_updated || theme_updated {
        //     self.images =
        //         images::calc(&self.ast, &self.images, &self.client, &self.core, self.file_id, ui);
        // }
        // self.galleys = galleys::calc(
        //     &self.ast,
        //     &self.buffer,
        //     &self.bounds,
        //     &self.images,
        //     &self.appearance,
        //     touch_mode,
        //     ui,
        // );
        // self.bounds.lines = bounds::calc_lines(&self.galleys, &self.bounds.ast, &self.bounds.text);
        // self.capture.update(
        //     ui.input(|i| i.pointer.latest_pos()),
        //     Instant::now(),
        //     &self.galleys,
        //     &self.buffer.current.segs,
        //     &self.bounds,
        //     &self.ast,
        // );

        // repaint conditions
        let mut repaints = Vec::new();
        if self.images.any_loading() {
            // repaint every 50ms until images load
            repaints.push(Duration::from_millis(50));
        }
        if let Some(recalc_after) = self.capture.recalc_after() {
            // repaint when capture state needs it
            repaints.push(recalc_after);
        }
        if let Some(&repaint_after) = repaints.iter().min() {
            ui.ctx().request_repaint_after(repaint_after);
        }

        // draw
        // self.draw_text(ui);
        // if self.focused(ui.ctx()) && !cfg!(target_os = "ios") {
        //     self.draw_cursor(ui, touch_mode);
        // }
        // if self.debug.draw_enabled {
        //     self.draw_debug(ui);
        // }

        // scroll
        // let all_selection = {
        //     DocCharOffset(0)
        //         .range_bound(Bound::Doc, false, false, &self.bounds)
        //         .unwrap() // there's always a document
        // };
        // if selection_updated && self.buffer.current.selection != all_selection {
        //     let cursor_end_line = cursor::line(
        //         self.buffer.current.selection.1,
        //         &self.galleys,
        //         &self.bounds.text,
        //         &self.appearance,
        //     );
        //     let rect = Rect { min: cursor_end_line[0], max: cursor_end_line[1] };
        //     ui.scroll_to_rect(rect.expand(rect.height()), None);
        // }

        // let suggested_title = self.get_suggested_title();
        // let suggest_rename =
        //     if suggested_title != prior_suggested_title { suggested_title } else { None };

        // focus editor by default
        if ui.memory(|m| m.focused().is_none()) {
            self.focus(ui.ctx());
        }
        if self.focused(ui.ctx()) {
            self.focus_lock(ui.ctx());
        }

        // Response {
        //     text_updated,
        //     selection_updated,
        //     scroll_updated: false, // set by scroll_ui
        //     suggest_rename,
        // }

        Default::default()
    }
}
