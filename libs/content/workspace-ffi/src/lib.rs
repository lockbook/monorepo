use egui_wgpu_backend::wgpu::{self, SurfaceTexture};
use std::time::Instant;
use std::{iter, time::Duration};
use tracing::{info, info_span, instrument, span, Level};
use workspace_rs::workspace::Workspace;

/// cbindgen:ignore
pub mod android;
pub mod apple;
pub mod response;

pub use response::Response;

#[repr(C)]
pub struct WgpuWorkspace<'window> {
    pub start_time: Instant,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'window>,
    pub adapter: wgpu::Adapter,

    // remember size last frame to detect resize
    pub surface_width: u32,
    pub surface_height: u32,

    pub rpass: egui_wgpu_backend::RenderPass,
    pub screen: egui_wgpu_backend::ScreenDescriptor,

    pub context: egui::Context,
    pub raw_input: egui::RawInput,

    pub workspace: Workspace,
}

impl<'window> WgpuWorkspace<'window> {
    #[instrument(level = "info", name = "rust frame", skip_all)]
    pub fn frame(&mut self) -> Response {
        self.configure_surface();
        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                // This error occurs when the app is minimized on Windows.
                // Silently return here to prevent spamming the console with:
                // "The underlying surface has changed, and therefore the swap chain must be updated"
                eprintln!("wgpu::SurfaceError::Outdated");
                return Default::default();
            }
            Err(e) => {
                eprintln!("Dropped frame with error: {}", e);
                return Default::default();
            }
        };

        info!("output_frame");
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        info!("output view");

        // can probably use run
        self.set_egui_screen();
        self.raw_input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.context.begin_frame(self.raw_input.take());
        info!("begin frame");

        let workspace_response = {
            let fill = if self.context.style().visuals.dark_mode {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            };
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(fill))
                .show(&self.context, |ui| {
                    self.workspace.show_debug_stroke(ui)
                    // self.workspace.show(ui)
                })
                .inner
        };

        let full_output = self.context.end_frame();

        let paint_jobs = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        info!("tess");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        self.rpass
            .add_textures(&self.device, &self.queue, &tdelta)
            .expect("add texture ok");
        info!("add textures");

        self.rpass
            .update_buffers(&self.device, &self.queue, &paint_jobs, &self.screen);

        info!("update buffers");
        // Record all render passes.
        self.rpass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &self.screen,
                Some(wgpu::Color::BLACK),
            )
            .unwrap();
        // Submit the commands.
        info!("run  pass");

        self.queue.submit(iter::once(encoder.finish()));

        info!("submit queue");

        // Redraw egui

        output_frame.present();

        self.rpass
            .remove_textures(tdelta)
            .expect("remove texture ok");

        info!("done");

        Response::new(
            &self.context,
            full_output.platform_output,
            full_output.viewport_output,
            workspace_response,
        )
    }

    pub fn set_egui_screen(&mut self) {
        use egui::{Pos2, Rect};

        self.raw_input.screen_rect = Some(Rect {
            min: Pos2::ZERO,
            max: Pos2::new(
                self.screen.physical_width as f32 / self.screen.scale_factor,
                self.screen.physical_height as f32 / self.screen.scale_factor,
            ),
        });
        self.context.set_pixels_per_point(self.screen.scale_factor);
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        // todo: is this really fine?
        // from here: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs#L65
        let capabilities = self.surface.get_capabilities(&self.adapter);
        capabilities.formats[0]
    }

    pub fn configure_surface(&mut self) {
        use egui_wgpu_backend::wgpu::CompositeAlphaMode;

        let resized = self.screen.physical_width != self.surface_width
            || self.screen.physical_height != self.surface_height;
        let visible = self.screen.physical_width * self.screen.physical_height != 0;
        if resized && visible {
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format(),
                width: self.screen.physical_width,
                height: self.screen.physical_height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            self.surface.configure(&self.device, &surface_config);
            self.surface_width = self.screen.physical_width;
            self.surface_height = self.screen.physical_height;
        }
    }
}
