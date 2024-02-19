use egui::{Image, Widget};

pub struct ImageViewer {
    img: Image<'static>,
}

impl ImageViewer {
    pub fn new(id: impl Into<String>, bytes: &[u8]) -> Self {
        let bytes = Vec::from(bytes);
        let img = Image::from_bytes(id.into(), bytes);

        Self { img }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.img.clone().ui(ui);
    }
}

pub fn is_supported_image_fmt(ext: &str) -> bool {
    // todo see if this list is incomplete
    const IMG_FORMATS: [&str; 7] = ["png", "jpeg", "jpg", "gif", "webp", "bmp", "ico"];
    IMG_FORMATS.contains(&ext)
}
