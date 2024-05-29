use crate::tab::markdown_editor::ast::Ast;
use crate::tab::markdown_editor::style::{InlineNode, MarkdownNode, Url};
use egui::{Image, Ui};
use lb_rs::Uuid;
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Transform, TreeParsing as _};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Default)]
pub struct ImageCache<'i> {
    pub map: HashMap<Url, Arc<Mutex<ImageState<'i>>>>,
}

#[derive(Clone, Debug, Default)]
pub enum ImageState<'i> {
    #[default]
    Loading,
    Loaded(Image<'i>),
    Failed(String),
}

pub fn calc<'i>(
    ast: &Ast, prior_cache: &ImageCache, client: &reqwest::blocking::Client, core: &lb_rs::Core,
    ui: &Ui,
) -> ImageCache<'i> {
    let mut result = ImageCache::default();

    let mut prior_cache = prior_cache.clone();
    for node in &ast.nodes {
        if let MarkdownNode::Inline(InlineNode::Image(_, url, title)) = &node.node_type {
            let (url, title) = (url.clone(), title.clone());

            if result.map.contains_key(&url) {
                // the second removal of the same image from the prior cache is always a cache miss and causes performance issues
                // we need to remove cache hits from the prior cache to avoid freeing them from the texture manager
                continue;
            }

            if let Some(cached) = prior_cache.map.remove(&url) {
                // re-use image from previous cache (even it if failed to load)
                result.map.insert(url, cached);
            } else {
                let url = url.clone();
                let image_state: Arc<Mutex<ImageState>> = Default::default();
                let client = client.clone();
                let core = core.clone();
                let ctx = ui.ctx().clone();

                result.map.insert(url.clone(), image_state.clone());

                // fetch image
                thread::spawn(move || {
                    let texture_result = (|| -> Result<Image, String> {
                        // use core for lb:// urls
                        // todo: also handle relative paths
                        let maybe_lb_id = match url.strip_prefix("lb://") {
                            Some(id) => Some(Uuid::parse_str(id).map_err(|e| e.to_string())?),
                            None => None,
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
                                let tree = usvg::Tree::from_data(&image_bytes, &Default::default())
                                    .map_err(|e| e.to_string())?;
                                let tree = resvg::Tree::from_usvg(&tree);
                                if let Some(content_area) = tree.content_area {
                                    // dimensions & transform chosen so that all svg content appears in the result
                                    let mut pix_map = Pixmap::new(
                                        content_area.width() as _,
                                        content_area.height() as _,
                                    )
                                    .ok_or("failed to create pixmap")
                                    .map_err(|e| e.to_string())?;
                                    let transform = Transform::identity()
                                        .post_translate(-content_area.left(), -content_area.top());
                                    tree.render(transform, &mut pix_map.as_mut());
                                    pix_map.encode_png().map_err(|e| e.to_string())?
                                } else {
                                    // empty svg
                                    Pixmap::new(100, 100)
                                        .unwrap()
                                        .encode_png()
                                        .map_err(|e| e.to_string())?
                                }
                            } else {
                                // leave non-drawings alone
                                image_bytes
                            }
                        } else {
                            // leave non-lockbook images alone
                            image_bytes
                        };

                        Ok(Image::from_bytes(format!("bytes://{}", url), image_bytes))
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

    result
}

fn download_image(
    client: &reqwest::blocking::Client, url: &str,
) -> Result<Vec<u8>, reqwest::Error> {
    let response = client.get(url).send()?.bytes()?.to_vec();
    Ok(response)
}

impl<'i> ImageCache<'i> {
    pub fn any_loading(&self) -> bool {
        self.map
            .values()
            .any(|state| matches!(state.lock().unwrap().deref(), Loading))
    }
}
