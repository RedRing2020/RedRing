//! AppState の通常の表示（製品形状・切削アニメーション）と、デバッグ表示の切り替えを扱うモジュール。
//!
//! デバッグ表示に切り替えるときは、通常の表示のステージとカメラを取っておき、戻すときにそのまま戻す。デバッグ表示の
//! 間は、切削アニメーションのコマを動かさない（届いた素材の面は、取っておいたステージに当てる）。
//! 通常の表示と見比べる表示（逆オフセット形状）は、ツールパスの線を重ね、カメラを通常の表示と共有する（その表示の間に
//! 動かしたカメラは、戻した後も使う）。

use super::AppState;
use super::debug_snapshot_state::SnapshotPlaybackMode;
use stage::{MeshStage, RenderStage};
use viewmodel_graphics::Camera;

/// 取っておいた通常の表示
pub(super) struct MainView {
    camera: Camera,
    /// 通常の表示のステージ（最初のデバッグ表示のステージを描いたときに取っておく）
    stage: Option<Box<dyn RenderStage>>,
    /// 今のデバッグ表示が、通常の表示と見比べる表示か（ツールパスの線を重ね、カメラを通常の表示と共有する）
    comparison: bool,
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

    /// デバッグ表示に入る。通常の表示からなら、カメラを取っておき、自動再生を一時停止する。デバッグ表示の間に呼んだ場合は、
    /// 取っておいたものを保つ。ただし、今が見比べる表示なら、そこで動かしたカメラを通常の表示のカメラとして取っておき、
    /// 見比べる表示ではなくする（続けて呼んでも、その後に動かしたカメラで上書きしない）
    ///
    /// デバッグ表示がカメラを動かす前に呼ぶ。ステージは `show_debug_stage` で描くときに取っておく。
    pub(super) fn enter_debug_view(&mut self) {
        if let Some(view) = &mut self.main_view {
            // 見比べる表示で動かしたカメラは、通常の表示のカメラとして取っておく
            if view.comparison {
                view.camera = self.camera.clone();
                view.comparison = false;
            }
            return;
        }
        if self.debug_snapshot.playback_mode == SnapshotPlaybackMode::AutoPlaying {
            self.pause_auto_snapshot_playback();
        }
        self.main_view = Some(MainView {
            camera: self.camera.clone(),
            stage: None,
            comparison: false,
        });
        self.window_title.clear();
        self.refresh_window_title();
    }

    /// デバッグ表示のステージ `stage` を描く。通常の表示のステージは取っておき、前のデバッグ表示のステージは捨てる
    ///
    /// `comparison` は、`stage` が通常の表示と見比べる表示か（ツールパスの線の表示の切り替えと設定の反映を当て、カメラを
    /// 通常の表示と共有する）。
    pub(super) fn show_debug_stage(&mut self, stage: Box<MeshStage>, comparison: bool) {
        self.enter_debug_view();
        let previous = self.renderer.replace_stage(stage);
        if let Some(view) = &mut self.main_view {
            if view.stage.is_none() {
                view.stage = Some(previous);
            }
            view.comparison = comparison;
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
        if !self.main_view.as_ref()?.comparison {
            return None;
        }
        self.renderer
            .get_stage_mut()
            .as_any_mut()
            .downcast_mut::<MeshStage>()
    }

    /// 取っておいた通常の表示（ステージとカメラ）に戻す（デバッグ表示でなければ何もしない）
    ///
    /// 見比べる表示から戻すときは、カメラを今のまま使う。
    pub(super) fn restore_main_view(&mut self) {
        let Some(MainView {
            camera,
            stage,
            comparison,
        }) = self.main_view.take()
        else {
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
        if !comparison {
            self.camera = camera;
        }
        self.update_camera_uniforms();
        // 画面の下の行を起動の案内に戻すだけで、新しいメッセージとして一覧には足さない
        if self.notice == crate::help_ui::Notice::DebugViewShown {
            self.notice = crate::help_ui::Notice::Startup;
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
