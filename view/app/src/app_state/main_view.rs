//! AppState の通常の表示（製品形状・切削アニメーション）と、デバッグ表示の切り替えを扱うモジュール。
//!
//! デバッグ表示に切り替えるときは、通常の表示のステージとカメラを取っておき、戻すときにそのまま戻す。デバッグ表示の
//! 間は、切削アニメーションのコマを動かさない（届いた素材の面は、取っておいたステージに当てる）。

use super::debug_snapshot_state::SnapshotPlaybackMode;
use super::AppState;
use crate::app::APP_TITLE;
use stage::{MeshStage, RenderStage};
use viewmodel_graphics::Camera;

/// 取っておいた通常の表示
pub(super) struct MainView {
    camera: Camera,
    /// 通常の表示のステージ（最初のデバッグ表示のステージを描いたときに取っておく）
    stage: Option<Box<dyn RenderStage>>,
    /// 今のデバッグ表示のステージに、ツールパスの線を重ねたか
    toolpath_overlay: bool,
}

impl std::fmt::Debug for MainView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainView").finish_non_exhaustive()
    }
}

impl AppState {
    /// デバッグ表示か（通常の表示を取っておいているか）
    pub(crate) fn debug_view_shown(&self) -> bool {
        self.main_view.is_some()
    }

    /// デバッグ表示に入る。通常の表示からなら、カメラを取っておき、自動再生を一時停止する（デバッグ表示の間に呼んでも、
    /// 最初に取っておいたものを保つ）
    ///
    /// デバッグ表示がカメラを動かす前に呼ぶ。ステージは `show_debug_stage` で描くときに取っておく。
    pub(super) fn enter_debug_view(&mut self) {
        if self.main_view.is_some() {
            return;
        }
        if self.debug_snapshot.playback_mode == SnapshotPlaybackMode::AutoPlaying {
            self.pause_auto_snapshot_playback();
        }
        self.main_view = Some(MainView {
            camera: self.camera.clone(),
            stage: None,
            toolpath_overlay: false,
        });
        self.window.set_title(APP_TITLE);
    }

    /// デバッグ表示のステージ `stage` を描く。通常の表示のステージは取っておき、前のデバッグ表示のステージは捨てる
    ///
    /// `toolpath_overlay` は、`stage` にツールパスの線を重ねたか（線の表示の切り替えと設定の反映を当てる）。
    pub(super) fn show_debug_stage(&mut self, stage: Box<MeshStage>, toolpath_overlay: bool) {
        self.enter_debug_view();
        let previous = self.renderer.replace_stage(stage);
        if let Some(view) = &mut self.main_view {
            if view.stage.is_none() {
                view.stage = Some(previous);
            }
            view.toolpath_overlay = toolpath_overlay;
        }
        self.update_camera_uniforms();
    }

    /// カメラを、取っておいた通常の表示のカメラにする（デバッグ表示でなければ何もしない）
    pub(super) fn use_main_view_camera(&mut self) {
        if let Some(view) = &self.main_view {
            self.camera = view.camera.clone();
        }
    }

    /// ツールパスの線を重ねたデバッグ表示のメッシュステージ（重ねていなければ `None`）
    pub(super) fn debug_toolpath_overlay_stage(&mut self) -> Option<&mut MeshStage> {
        if !self.main_view.as_ref()?.toolpath_overlay {
            return None;
        }
        self.renderer
            .get_stage_mut()
            .as_any_mut()
            .downcast_mut::<MeshStage>()
    }

    /// 取っておいた通常の表示（ステージとカメラ）に戻す（デバッグ表示でなければ何もしない）
    pub(super) fn restore_main_view(&mut self) {
        let Some(MainView { camera, stage, .. }) = self.main_view.take() else {
            return;
        };
        if let Some(mut stage) = stage {
            // デバッグ表示の間に変わった画面の大きさを伝える
            stage.on_surface_resized(
                &self.graphic.device,
                (self.graphic.config.width, self.graphic.config.height),
            );
            self.renderer.set_stage(stage);
        }
        self.camera = camera;
        self.update_camera_uniforms();
        if self.notice == crate::help_ui::Notice::DebugViewShown {
            self.set_notice(crate::help_ui::Notice::Startup);
        }
        self.refresh_snapshot_window_title();
    }

    /// 新しい通常の表示を描くときに、取っておいた通常の表示を捨てる
    pub(super) fn discard_main_view(&mut self) {
        self.main_view = None;
    }

    /// 通常の表示のメッシュステージ（デバッグ表示の間は取っておいたもの。メッシュステージでなければ `None`）
    pub(super) fn main_mesh_stage(&mut self) -> Option<&mut MeshStage> {
        match &mut self.main_view {
            Some(MainView {
                stage: Some(stage), ..
            }) => stage.as_any_mut().downcast_mut::<MeshStage>(),
            _ => self
                .renderer
                .get_stage_mut()
                .as_any_mut()
                .downcast_mut::<MeshStage>(),
        }
    }

    /// デバッグ表示の間は、切削アニメーションのコマの操作を受けず、ステータスバーで知らせる（受けないとき `true`）
    pub(super) fn reject_while_debug_view(&mut self) -> bool {
        if !self.debug_view_shown() {
            return false;
        }
        self.set_notice(crate::help_ui::Notice::DebugViewShown);
        true
    }
}
