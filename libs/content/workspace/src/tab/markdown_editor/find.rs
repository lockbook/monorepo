use egui;

pub struct FindState {
    pub visible: bool,
    pub visible_last_frame: bool,
    pub focused: bool,
    pub focused_last_frame: bool,
    pub term: String,
    pub term_id: egui::Id,
}

impl Default for FindState {
    fn default() -> Self {
        Self {
            visible: false,
            visible_last_frame: false,
            focused: false,
            focused_last_frame: false,
            term: String::new(),
            term_id: egui::Id::new("find_term"),
        }
    }
}
