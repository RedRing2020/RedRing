use crate::help_ui::{HelpUiState, NoticeBarUiState};
use crate::machining_conditions_ui::MachiningConditionsUiState;
use crate::menu_bar_ui::MenuBarUiState;
use crate::settings_panel_ui::SettingsPanelUiState;
use egui::{ClippedPrimitive, Context, Order, Rect, TexturesDelta, ViewportId};
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

/// ウィンドウの中身の下と、範囲の下端との間（ウィンドウの枠と、下の余白）
const WINDOW_BODY_BOTTOM_GAP: f32 = 24.0;
/// ウィンドウの中身の高さの下限（範囲がとても低いときも、少しは見せる）
const WINDOW_BODY_MIN_HEIGHT: f32 = 120.0;

/// ウィンドウの中身を置ける高さ（中身の上端から、範囲 `area`（メニューバーと知らせの行の間）の下端まで）
pub(crate) fn window_body_max_height(ui: &egui::Ui, area: Rect) -> f32 {
    (area.bottom() - ui.cursor().top() - WINDOW_BODY_BOTTOM_GAP).max(WINDOW_BODY_MIN_HEIGHT)
}

pub(crate) struct SettingsPanelRenderer {
    egui_context: Context,
    egui_state: State,
    egui_renderer: Renderer,
    pending_frame: Option<PendingSettingsPanelFrame>,
    /// メニューバーなど上端の UI の高さ（egui の座標）と、その物理ピクセルへの倍率（最後に描いた画面の値）
    menu_bar_bottom: f32,
    /// 画面の下の知らせの行の上端（論理座標。最後に描いた画面の値）
    notice_bar_top: f32,
    pixels_per_point: f32,
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
            menu_bar_bottom: 0.0,
            notice_bar_top: f32::INFINITY,
            pixels_per_point: 1.0,
        }
    }

    pub(crate) fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_state.on_window_event(window, event).consumed
    }

    /// ポインタが egui の上にあるか、egui が使っている（スライダーのドラッグ等。egui の外で始めたドラッグは含めない）
    ///
    /// egui の上は、メニューバーと、背景より前の層（ウィンドウ・メニューの一覧）。`run_ui` は画面全体を背景の Ui と
    /// するため、egui の `egui_wants_pointer_input` は 3D の表示の上でも真になり、使えない。
    pub(crate) fn wants_pointer_input(&self) -> bool {
        let ctx = &self.egui_context;
        if ctx.egui_is_using_pointer() {
            return true;
        }
        let (position, any_down) = ctx.input(|i| (i.pointer.interact_pos(), i.pointer.any_down()));
        let Some(position) = position.filter(|_| !any_down) else {
            return false;
        };
        position.y < self.menu_bar_bottom
            || position.y > self.notice_bar_top
            || ctx
                .layer_id_at(position)
                .is_some_and(|layer| layer.order != Order::Background)
    }

    /// eguiが現在キーボード入力を受け取っている（入力欄の編集中）
    pub(crate) fn is_using_keyboard(&self) -> bool {
        self.egui_context.egui_wants_keyboard_input()
    }

    pub(crate) fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.egui_state.on_mouse_motion(delta);
    }

    /// メニューバーなど上端の UI の高さ（物理ピクセル。最後に描いた画面の値）
    pub(crate) fn top_inset(&self) -> f32 {
        self.menu_bar_bottom * self.pixels_per_point
    }

    /// メニューバー・知らせの行と、設定パネル・加工条件・ヘルプのウィンドウを、同じ egui の文脈で描く（ウィンドウは
    /// メニューバーと知らせの行の間に置く）
    pub(crate) fn prepare(
        &mut self,
        window: &Window,
        menu: &mut MenuBarUiState,
        notice: &NoticeBarUiState<'_>,
        panel: &mut SettingsPanelUiState,
        conditions: &mut MachiningConditionsUiState,
        help: &mut HelpUiState,
    ) {
        let raw_input = self.egui_state.take_egui_input(window);
        let mut area = Rect::NOTHING;
        let full_output = self.egui_context.run_ui(raw_input, |ui| {
            menu.show(ui);
            notice.show(ui);
            area = ui.available_rect_before_wrap();
            panel.show(ui, area);
            conditions.show(ui, area);
            help.show(ui, area);
        });
        let pixels_per_point = full_output.pixels_per_point;
        self.menu_bar_bottom = area.top();
        self.notice_bar_top = area.bottom();
        self.pixels_per_point = pixels_per_point;
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

        // 同じテクスチャの更新は、並んだ順に当てる
        for (texture_id, image_deltas) in &frame.textures_delta.set {
            for image_delta in image_deltas {
                self.egui_renderer
                    .update_texture(device, queue, *texture_id, image_delta);
            }
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
