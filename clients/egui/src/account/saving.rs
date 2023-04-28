use std::thread;
use std::time::Instant;

use eframe::egui;

use crate::account::AccountUpdate;

pub struct SaveRequest {
    pub id: lb::Uuid,
    pub content: SaveRequestContent,
}

pub enum SaveRequestContent {
    Text(String),
    Draw(lb::Drawing),
}

impl super::AccountScreen {
    pub fn process_save_requests(&mut self, ctx: &egui::Context) {
        let save_req_rx = self.save_req_rx.take().unwrap();
        let update_tx = self.update_tx.clone();
        let core = self.core.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            while let Ok(req) = save_req_rx.recv() {
                println!("processing request for {} ...", req.id);
                let result = match req.content {
                    SaveRequestContent::Text(s) => core.write_document(req.id, s.as_bytes()),
                    SaveRequestContent::Draw(d) => core.save_drawing(req.id, &d),
                };
                let update = match result {
                    Ok(()) => AccountUpdate::Saved(req.id, Instant::now()),
                    Err(err) => AccountUpdate::SaveFailed(req.id, err),
                };
                update_tx.send(update).unwrap();
                ctx.request_repaint();
            }
        });
    }
}
