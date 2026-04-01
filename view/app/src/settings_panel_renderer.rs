use crate::settings_panel_ui::SettingsPanelUiState;
use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State;
use fontdb::{Database, Family, Query};
use tracing::{info, warn};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::{event::WindowEvent, window::Window};

struct PendingSettingsPanelFrame {
    paint_jobs: Vec<ClippedPrimitive>,
    textures_delta: TexturesDelta,
    pixels_per_point: f32,
}

pub(crate) struct SettingsPanelRenderer {
    egui_context: Context,
    egui_state: State,
    egui_renderer: Renderer,
    pending_frame: Option<PendingSettingsPanelFrame>,
}

impl SettingsPanelRenderer {
    pub(crate) fn new(window: &Window, device: &Device, format: TextureFormat) -> Self {
        let egui_context = Context::default();
        install_cjk_system_font(&egui_context);

        let egui_state = State::new(
            egui_context.clone(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = Renderer::new(device, format, RendererOptions::default());

        Self {
            egui_context,
            egui_state,
            egui_renderer,
            pending_frame: None,
        }
    }

    pub(crate) fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_state.on_window_event(window, event).consumed
    }

    /// eguiが現在マウスポインタを独占使用中（スライダードラッグ等）
    pub(crate) fn is_using_pointer(&self) -> bool {
        self.egui_context.egui_is_using_pointer()
    }

    pub(crate) fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.egui_state.on_mouse_motion(delta);
    }

    pub(crate) fn prepare(&mut self, window: &Window, panel: &mut SettingsPanelUiState) {
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_context.run_ui(raw_input, |ctx| panel.show(ctx));
        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = self
            .egui_context
            .tessellate(full_output.shapes, pixels_per_point);

        self.egui_state
            .handle_platform_output(window, full_output.platform_output);
        self.pending_frame = Some(PendingSettingsPanelFrame {
            paint_jobs,
            textures_delta: full_output.textures_delta,
            pixels_per_point,
        });
    }

    pub(crate) fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        if viewport_width == 0 || viewport_height == 0 {
            self.pending_frame = None;
            return;
        }

        let Some(frame) = self.pending_frame.take() else {
            return;
        };

        for (texture_id, image_delta) in &frame.textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, *texture_id, image_delta);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [viewport_width, viewport_height],
            pixels_per_point: frame.pixels_per_point,
        };

        let user_command_buffers = self.egui_renderer.update_buffers(
            device,
            queue,
            encoder,
            &frame.paint_jobs,
            &screen_descriptor,
        );
        if !user_command_buffers.is_empty() {
            queue.submit(user_command_buffers);
        }

        if !frame.paint_jobs.is_empty() {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Settings Panel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let mut render_pass = render_pass.forget_lifetime();
            self.egui_renderer
                .render(&mut render_pass, &frame.paint_jobs, &screen_descriptor);
        }

        for texture_id in &frame.textures_delta.free {
            self.egui_renderer.free_texture(texture_id);
        }
    }
}

fn install_cjk_system_font(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(font_bytes) = load_system_cjk_font_bytes() {
        let font_name = "system_cjk".to_owned();
        fonts.font_data.insert(
            font_name.clone(),
            egui::FontData::from_owned(font_bytes).into(),
        );

        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, font_name.clone());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push(font_name);
        }

        info!("egui: CJK system font detected and applied");
    } else {
        warn!("egui: CJK system font not found; non-ASCII text may render as square boxes");
    }

    ctx.set_fonts(fonts);
}

fn load_system_cjk_font_bytes() -> Option<Vec<u8>> {
    let mut db = Database::new();
    db.load_system_fonts();

    let family_candidates = [
        "Yu Gothic",
        "Meiryo",
        "MS Gothic",
        "Microsoft YaHei",
        "Hiragino Sans",
        "PingFang SC",
        "Noto Sans CJK JP",
        "Noto Sans CJK SC",
        "Noto Sans JP",
    ];

    for family_name in family_candidates {
        let query = Query {
            families: &[Family::Name(family_name)],
            ..Query::default()
        };

        if let Some(face_id) = db.query(&query) {
            if let Some(bytes) = db.with_face_data(face_id, |data, _index| data.to_vec()) {
                return Some(bytes);
            }
        }
    }

    None
}
