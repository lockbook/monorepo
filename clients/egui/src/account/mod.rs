mod modals;
mod syncing;
mod tabs;
mod tree;
mod workspace;

use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

use eframe::egui;

use crate::model::{AccountScreenInitData, Usage};
use crate::settings::Settings;
use crate::theme::Icon;
use crate::util::NUM_KEYS;
use crate::widgets::{separator, sidebar_button};

use self::modals::*;
use self::syncing::{SyncPanel, SyncUpdate};
use self::tabs::{Drawing, ImageViewer, Markdown, PlainText, Tab, TabContent, TabFailure};
use self::tree::FileTree;
use self::workspace::Workspace;

pub struct AccountScreen {
    settings: Arc<RwLock<Settings>>,
    core: Arc<lb::Core>,

    update_tx: mpsc::Sender<AccountUpdate>,
    update_rx: mpsc::Receiver<AccountUpdate>,

    tree: FileTree,
    sync: SyncPanel,
    usage: Result<Usage, String>,
    workspace: Workspace,
    modals: Modals,
}

impl AccountScreen {
    pub fn new(
        settings: Arc<RwLock<Settings>>, core: Arc<lb::Core>, acct_data: AccountScreenInitData,
        ctx: &egui::Context,
    ) -> Self {
        let (update_tx, update_rx) = mpsc::channel();

        let AccountScreenInitData { files, sync_status, usage } = acct_data;

        start_auto_syncing(&settings, &core, &update_tx, ctx);

        Self {
            settings,
            core,
            update_tx,
            update_rx,
            tree: FileTree::new(files),
            sync: SyncPanel::new(sync_status),
            usage,
            workspace: Workspace::new(),
            modals: Modals::default(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.process_updates(ctx, frame);
        self.process_keys(ctx, frame);

        let sidebar_width = egui::SidePanel::left("sidebar_panel")
            .frame(egui::Frame::none().fill(ctx.style().visuals.faint_bg_color))
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.set_enabled(!self.is_any_modal_open());

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    self.show_sync_panel(ui);

                    separator(ui);

                    if sidebar_button(ui, &Icon::SETTINGS, "Settings").clicked() {
                        self.update_tx.send(OpenModal::Settings.into()).unwrap();
                        ctx.request_repaint();
                    }

                    separator(ui);

                    self.show_tree(ui);
                });
            })
            .response
            .rect
            .max
            .x;

        egui::CentralPanel::default().show(ctx, |ui| self.show_workspace(frame, ui));

        self.show_any_modals(ctx, 0.0 - (sidebar_width / 2.0));
    }

    fn process_updates(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                AccountUpdate::BgUpdate(update) => match update {
                    BgUpdate::AutoSync => self.perform_sync(ctx),
                },
                AccountUpdate::OpenModal(open_modal) => match open_modal {
                    OpenModal::NewFile(maybe_parent) => self.open_new_file_modal(maybe_parent),
                    OpenModal::Settings => {
                        self.modals.settings = SettingsModal::open(&self.core, &self.settings);
                    }
                    OpenModal::ConfirmDelete(files) => {
                        self.modals.confirm_delete = ConfirmDeleteModal::open(files);
                    }
                },
                AccountUpdate::FileCreated(result) => {
                    if let Some(content) = &mut self.modals.new_file {
                        match result {
                            Ok(meta) => {
                                self.modals.new_file = None;
                                let (id, is_doc) = (meta.id, meta.is_document());
                                self.tree.root.insert(meta);
                                if is_doc {
                                    self.open_file(id, ctx);
                                }
                            }
                            Err(msg) => content.err_msg = Some(msg),
                        }
                    }
                }
                AccountUpdate::FileLoaded(id, content_result) => {
                    if let Some(tab) = self.workspace.get_mut_tab_by_id(id) {
                        frame.set_window_title(&tab.name);
                        match content_result {
                            Ok(content) => tab.content = Some(content),
                            Err(fail) => tab.failure = Some(fail),
                        }
                    }
                }
                AccountUpdate::FileRenamed(id, name) => {
                    if let Some(node) = self.tree.root.find_mut(id) {
                        node.file.name = name;
                    }
                }
                AccountUpdate::FileDeleted(f) => self.tree.remove(&f),
                AccountUpdate::SyncUpdate(update) => self.process_sync_update(ctx, update),
                AccountUpdate::DoneDeleting => self.modals.confirm_delete = None,
            }
        }
    }

    fn process_keys(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        const ALT: egui::Modifiers = egui::Modifiers::ALT;
        const CTRL: egui::Modifiers = egui::Modifiers::CTRL;

        // Escape (without modifiers) to close something such as an open modal.
        // We don't want to consume it unless something is closed.
        if ctx.input().key_pressed(egui::Key::Escape)
            && ctx.input().modifiers.is_none()
            && self.close_something()
        {
            ctx.input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::Escape);
        }

        // Ctrl-N pressed while new file modal is not open.
        if self.modals.new_file.is_none() && ctx.input_mut().consume_key(CTRL, egui::Key::N) {
            self.open_new_file_modal(None);
        }

        // Ctrl-S to save current tab.
        if ctx.input_mut().consume_key(CTRL, egui::Key::S) {
            self.save_current_tab();
        }

        // Ctrl-W to close current tab.
        if ctx.input_mut().consume_key(CTRL, egui::Key::W) && !self.workspace.is_empty() {
            self.save_current_tab();
            self.workspace.close_current_tab();
            frame.set_window_title(
                self.workspace
                    .current_tab()
                    .map(|tab| tab.name.as_str())
                    .unwrap_or("Lockbook"),
            );
        }

        // Ctrl-Space or Ctrl-L pressed while search modal is not open.
        let is_search_open = {
            let mut input = ctx.input_mut();
            input.consume_key(CTRL, egui::Key::Space) || input.consume_key(CTRL, egui::Key::L)
        };
        if is_search_open {
            if let Some(search) = &mut self.modals.search {
                search.focus_select_all();
            } else {
                self.modals.search = SearchModal::open(&self.core, ctx);
            }
        }

        // Ctrl-, to open settings modal.
        if self.modals.settings.is_none() && consume_key(ctx, ',') {
            self.modals.settings = SettingsModal::open(&self.core, &self.settings);
        }

        // Alt-H pressed to toggle the help modal.
        if ctx.input_mut().consume_key(ALT, egui::Key::H) {
            let d = &mut self.modals.help;
            *d = match d {
                Some(_) => None,
                None => Some(Box::new(HelpModal::default())),
            };
        }

        // Alt-{1-9} to easily navigate tabs (9 will always go to the last tab).
        for i in 1..10 {
            if ctx.input_mut().consume_key(ALT, NUM_KEYS[i - 1]) {
                self.workspace.goto_tab(i);
                if let Some(tab) = self.workspace.current_tab() {
                    frame.set_window_title(&tab.name);
                }
                break;
            }
        }
    }

    fn show_tree(&mut self, ui: &mut egui::Ui) {
        let resp = egui::ScrollArea::both()
            .show(ui, |ui| self.tree.show(ui))
            .inner;

        if let Some(file) = resp.new_file_modal {
            self.update_tx
                .send(OpenModal::NewFile(Some(file)).into())
                .unwrap();
            ui.ctx().request_repaint();
        }

        if let Some(rename_req) = resp.rename_request {
            self.rename_file(rename_req, ui.ctx());
        }

        for id in resp.open_requests {
            self.open_file(id, ui.ctx());
        }

        if resp.delete_request {
            let selected_files = self.tree.get_selected_files();
            if !selected_files.is_empty() {
                self.update_tx
                    .send(OpenModal::ConfirmDelete(selected_files).into())
                    .unwrap();
            }
        }

        if let Some(id) = resp.dropped_on {
            self.move_selected_files_to(ui.ctx(), id);
        }
    }

    fn save_settings(&mut self) {
        if let Err(err) = self.settings.read().unwrap().to_file() {
            self.modals.error = ErrorModal::open(err);
        }
    }

    fn open_new_file_modal(&mut self, maybe_parent: Option<lb::File>) {
        let parent_id = match maybe_parent {
            Some(f) => match f.is_folder() {
                true => f.id,
                false => f.parent,
            },
            None => self.core.get_root().unwrap().id,
        };

        let parent_path = self.core.get_path_by_id(parent_id).unwrap();

        self.modals.new_file = Some(Box::new(NewFileModal::new(parent_path)));
    }

    fn create_file(&mut self, params: modals::NewFileParams) {
        let parent = self.core.get_by_path(&params.parent_path).unwrap();

        let core = self.core.clone();
        let update_tx = self.update_tx.clone();
        thread::spawn(move || {
            let result = core
                .create_file(&params.name, parent.id, params.ftype)
                .map_err(|err| format!("{:?}", err));
            update_tx.send(AccountUpdate::FileCreated(result)).unwrap();
        });
    }

    fn open_file(&mut self, id: lb::Uuid, ctx: &egui::Context) {
        if self.workspace.goto_tab_id(id) {
            ctx.request_repaint();
            return;
        }

        let fname = self
            .core
            .get_file_by_id(id)
            .unwrap() // TODO
            .name;

        self.workspace.open_tab(id, &fname);

        let core = self.core.clone();
        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let ext = fname.split('.').last().unwrap_or_default();

            let content = if ext == "draw" {
                core.get_drawing(id)
                    .map_err(TabFailure::from)
                    .map(|drawing| TabContent::Drawing(Drawing::boxed(drawing)))
            } else {
                core.read_document(id)
                    .map_err(TabFailure::from)
                    .map(|bytes| {
                        if ext == "md" {
                            TabContent::Markdown(Markdown::boxed(&bytes))
                        } else if is_supported_image_fmt(ext) {
                            TabContent::Image(ImageViewer::boxed(id.to_string(), &bytes))
                        } else {
                            TabContent::PlainText(PlainText::boxed(&bytes))
                        }
                    })
            };

            update_tx
                .send(AccountUpdate::FileLoaded(id, content))
                .unwrap();
            ctx.request_repaint();
        });
    }

    fn move_selected_files_to(&mut self, ctx: &egui::Context, target: lb::Uuid) {
        let files = self.tree.get_selected_files();

        for f in files {
            if f.parent == target {
                continue;
            }
            if let Err(err) = self.core.move_file(f.id, target) {
                println!("{:?}", err);
                return;
            } else {
                let parent = self.tree.root.find_mut(f.parent).unwrap();
                let node = parent.remove(f.id).unwrap();
                let target_node = self.tree.root.find_mut(target).unwrap();
                target_node.insert_node(node);
                ctx.request_repaint();
            }
        }

        ctx.request_repaint();
    }

    fn rename_file(&self, req: (lb::Uuid, String), ctx: &egui::Context) {
        let core = self.core.clone();
        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let (id, new_name) = req;
            core.rename_file(id, &new_name).unwrap(); // TODO
            update_tx
                .send(AccountUpdate::FileRenamed(id, new_name))
                .unwrap();
            ctx.request_repaint();
        });
    }

    fn delete_files(&self, ctx: &egui::Context, files: Vec<lb::File>) {
        let core = self.core.clone();
        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            for f in &files {
                core.delete_file(f.id).unwrap(); // TODO
                update_tx
                    .send(AccountUpdate::FileDeleted(f.clone()))
                    .unwrap();
            }
            update_tx.send(AccountUpdate::DoneDeleting).unwrap();
            ctx.request_repaint();
        });
    }
}

enum AccountUpdate {
    BgUpdate(BgUpdate),

    /// To open some modals, we queue an update for the next frame so that the actions used to open
    /// each modal (such as the release of a click that would then be in the "outside" area of the
    /// modal) don't automatically close the modal during the same frame.
    OpenModal(OpenModal),

    FileCreated(Result<lb::File, String>),
    FileLoaded(lb::Uuid, Result<TabContent, TabFailure>),
    FileRenamed(lb::Uuid, String),
    FileDeleted(lb::File),

    SyncUpdate(SyncUpdate),

    DoneDeleting,
}

enum BgUpdate {
    AutoSync,
}

enum OpenModal {
    NewFile(Option<lb::File>),
    Settings,
    ConfirmDelete(Vec<lb::File>),
}

impl From<BgUpdate> for AccountUpdate {
    fn from(v: BgUpdate) -> Self {
        Self::BgUpdate(v)
    }
}

impl From<OpenModal> for AccountUpdate {
    fn from(v: OpenModal) -> Self {
        Self::OpenModal(v)
    }
}

impl From<SyncUpdate> for AccountUpdate {
    fn from(v: SyncUpdate) -> Self {
        Self::SyncUpdate(v)
    }
}

fn start_auto_syncing(
    settings: &Arc<RwLock<Settings>>, core: &Arc<lb::Core>,
    update_tx: &mpsc::Sender<AccountUpdate>, ctx: &egui::Context,
) {
    let settings = settings.clone();
    let core = core.clone();
    let update_tx = update_tx.clone();
    let ctx = ctx.clone();

    thread::spawn(move || loop {
        if settings.read().unwrap().auto_sync {
            match core.calculate_work() {
                Ok(work) => {
                    if !work.work_units.is_empty() {
                        update_tx.send(BgUpdate::AutoSync.into()).unwrap();
                        ctx.request_repaint();
                    }
                }
                Err(err) => println!("{:?}", err),
            }
        }
        thread::sleep(Duration::from_secs(10));
    });
}

fn is_supported_image_fmt(ext: &str) -> bool {
    const IMG_FORMATS: [&str; 7] = ["png", "jpeg", "jpg", "gif", "webp", "bmp", "ico"];
    IMG_FORMATS.contains(&ext)
}

fn consume_key(ctx: &egui::Context, key: char) -> bool {
    let mut input = ctx.input_mut();
    let m = &input.modifiers;
    if m.ctrl && !m.alt && !m.shift {
        if let Some(index) = input
            .events
            .iter()
            .position(|evt| *evt == egui::Event::Text(key.to_string()))
        {
            input.events.remove(index);
            return true;
        }
    }
    false
}
