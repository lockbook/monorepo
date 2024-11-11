use crate::tab::markdown_editor::{self, utils};
use comrak::nodes::{NodeLink, NodeValue};
use markdown_editor::Editor;

use crate::tab;
use egui::{ColorImage, TextureId, Ui};
use lb_rs::Uuid;
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Transform};
use std::collections::HashMap;
use std::ops::Deref as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default)]
pub struct Images {
    pub map: HashMap<String, Arc<Mutex<ImageState>>>,

    pub ast_seq: usize,

    pub seq: usize,
}

impl Editor {
    pub fn images(&mut self, ui: &mut Ui) {
        let Editor { ast, images, .. } = self;

        if utils::check_assign(&mut images.ast_seq, ast.seq) {
            images.seq += 1;
        } else {
            return;
        }

        let mut prior_cache = images.map.clone();
        images.map.clear();
        for node in &ast.nodes {
            if let NodeValue::Image(NodeLink { url, title }) = node {
                let (url, title) = (url.clone(), title.clone());

                if images.map.contains_key(&url) {
                    // the second removal of the same image from the prior cache is always a cache miss and causes performance issues
                    // we need to remove cache hits from the prior cache to avoid freeing them from the texture manager
                    continue;
                }

                if let Some(cached) = prior_cache.remove(&url) {
                    // re-use image from previous cache (even it if failed to load)
                    images.map.insert(url, cached);
                } else {
                    let url = url.clone();
                    let image_state: Arc<Mutex<ImageState>> = Default::default();
                    let client = self.client.clone();
                    let core = self.core.clone();
                    let file_id = self.file_id;
                    let ctx = ui.ctx().clone();

                    images.map.insert(url.clone(), image_state.clone());

                    // fetch image
                    thread::spawn(move || {
                        let texture_manager = ctx.tex_manager();

                        let texture_result = (|| -> Result<TextureId, String> {
                            // use core for lb:// urls and relative paths
                            let maybe_lb_id = match url.strip_prefix("lb://") {
                                Some(id) => Some(Uuid::parse_str(id).map_err(|e| e.to_string())?),
                                None => tab::core_get_by_relative_path(
                                    &core,
                                    file_id,
                                    &PathBuf::from(&url),
                                )
                                .map(|f| f.id)
                                .ok(),
                            };

                            let image_bytes = if let Some(id) = maybe_lb_id {
                                core.read_document(id).map_err(|e| e.to_string())?
                            } else {
                                download_image(&client, &url).map_err(|e| e.to_string())?
                            };

                            // convert lockbook drawings to images
                            let image_bytes = if let Some(id) = maybe_lb_id {
                                let file = core.get_file_by_id(id).map_err(|e| e.to_string())?;
                                if file.name.ends_with(".svg") {
                                    // todo: check errors
                                    let tree = usvg::Tree::from_data(
                                        &image_bytes,
                                        &Default::default(),
                                        &Default::default(),
                                    )
                                    .map_err(|e| e.to_string())?;

                                    let bounding_box = tree.root().abs_bounding_box();

                                    // dimensions & transform chosen so that all svg content appears in the result
                                    let mut pix_map = Pixmap::new(
                                        bounding_box.width() as _,
                                        bounding_box.height() as _,
                                    )
                                    .ok_or("failed to create pixmap")
                                    .map_err(|e| e.to_string())?;
                                    let transform = Transform::identity()
                                        .post_translate(-bounding_box.left(), -bounding_box.top());
                                    resvg::render(&tree, transform, &mut pix_map.as_mut());
                                    pix_map.encode_png().map_err(|e| e.to_string())?
                                } else {
                                    // leave non-drawings alone
                                    image_bytes
                                }
                            } else {
                                // leave non-lockbook images alone
                                image_bytes
                            };

                            let image =
                                image::load_from_memory(&image_bytes).map_err(|e| e.to_string())?;
                            let size_pixels = [image.width() as usize, image.height() as usize];

                            let egui_image = egui::ImageData::Color(
                                ColorImage::from_rgba_unmultiplied(size_pixels, &image.to_rgba8())
                                    .into(),
                            );
                            Ok(texture_manager
                                .write()
                                .alloc(title, egui_image, Default::default()))
                        })();

                        match texture_result {
                            Ok(texture_id) => {
                                *image_state.lock().unwrap() = ImageState::Loaded(texture_id);
                            }
                            Err(err) => {
                                *image_state.lock().unwrap() = ImageState::Failed(err);
                            }
                        }

                        // request a frame when the image is done loading
                        ctx.request_repaint();
                    });
                }
            }
        }

        let texture_manager = ui.ctx().tex_manager();
        for (_, eviction) in prior_cache.drain() {
            if let ImageState::Loaded(eviction) = eviction.lock().unwrap().deref() {
                texture_manager.write().free(*eviction);
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ImageState {
    #[default]
    Loading,
    Loaded(TextureId),
    Failed(String),
}

fn download_image(
    client: &reqwest::blocking::Client, url: &str,
) -> Result<Vec<u8>, reqwest::Error> {
    let response = client.get(url).send()?.bytes()?.to_vec();
    Ok(response)
}

impl Images {
    pub fn any_loading(&self) -> bool {
        self.map
            .values()
            .any(|state| &ImageState::Loading == state.lock().unwrap().deref())
    }
}
