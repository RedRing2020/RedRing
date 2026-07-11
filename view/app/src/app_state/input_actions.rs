//! AppState のキーボード入力ハンドリングを扱うモジュール。

use super::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CamDemoStartTrigger {
    BallEndMill,
    FlatEndMill,
}

fn resolve_cam_demo_start_trigger_with_modifiers(
    key: &winit::keyboard::Key,
    physical_key: &winit::keyboard::PhysicalKey,
    shift_pressed: bool,
) -> Option<CamDemoStartTrigger> {
    if !shift_pressed {
        return None;
    }

    match (physical_key, key) {
        (
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyB),
            winit::keyboard::Key::Character(ch),
        ) if ch.as_str() == "B" => Some(CamDemoStartTrigger::BallEndMill),
        (
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyF),
            winit::keyboard::Key::Character(ch),
        ) if ch.as_str() == "F" => Some(CamDemoStartTrigger::FlatEndMill),
        _ => None,
    }
}

impl AppState {
    /// キーボード入力を処理
    pub fn handle_keyboard_input(
        &mut self,
        key: &winit::keyboard::Key,
        physical_key: &winit::keyboard::PhysicalKey,
        pressed: bool,
    ) {
        self.mouse_input.update_key(key, pressed);

        if !pressed {
            return;
        }

        if let Some(trigger) = resolve_cam_demo_start_trigger_with_modifiers(
            key,
            physical_key,
            self.mouse_input.is_shift_pressed(),
        ) {
            match trigger {
                CamDemoStartTrigger::BallEndMill => self.load_sample_toolpath_ball_end_mill(),
                CamDemoStartTrigger::FlatEndMill => self.load_sample_toolpath_flat_end_mill(),
            }
            return;
        }

        if matches!(
            key,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space)
        ) {
            self.toggle_auto_snapshot_playback();
            return;
        }

        if let winit::keyboard::Key::Character(ch) = key {
            match ch.as_str() {
                "r" => {
                    self.camera.reset();
                    self.update_camera_uniforms();
                    tracing::info!("カメラをリセット（rキー）");
                }
                "t" => {
                    self.camera.reset_to_standard_cad_view();
                    self.update_camera_uniforms();
                    tracing::info!("標準CAD視点に設定（tキー）");
                }
                "f" => {
                    self.camera.reset_to_front_view();
                    self.update_camera_uniforms();
                    tracing::info!("正面視点に設定（fキー）");
                }
                "e" => {
                    self.camera.emergency_camera_escape();
                    self.update_camera_uniforms();
                    tracing::warn!("緊急カメラ脱出実行（eキー）");
                }
                "h" => {
                    tracing::info!("=== カメラ操作 ===");
                    tracing::info!("r: カメラリセット");
                    tracing::info!("t: 標準CAD視点");
                    tracing::info!("f: 正面視点");
                    tracing::info!("e: 緊急脱出");
                    tracing::info!("1: Draftステージ");
                    tracing::info!("2: Outlineステージ");
                    tracing::info!("3: Shadingステージ");
                    tracing::info!("=== 切削シミュレーション ===");
                    tracing::info!(
                        "【開始】 Shift+F: フラットエンドミルのシミュレーションデモ開始"
                    );
                    tracing::info!("【開始】 Shift+B: ボールエンドミルのシミュレーションデモ開始");
                    tracing::info!("【設定】 Shift+S: 設定パネル表示切替");
                    tracing::info!("【開始】 p: ToolPath のみ表示（シミュレーションなし）");
                    tracing::info!("【移動】 k: 次のスナップショットフレームへ");
                    tracing::info!("【移動】 j: 前のスナップショットフレームへ");
                    tracing::info!("【移動】 左ドラッグ（左上進捗バー）: 任意フレームへスクラブ");
                    tracing::info!("【再生】 Space: 自動再生/一時停止");
                    tracing::info!("【再生】 Shift+J: 自動再生停止（先頭へ）");
                    tracing::info!("【再生】 + / - : 再生速度変更（現在倍率はタイトル表示）");
                    tracing::info!("【表示】 w: ワイヤー表示 / ソリッド表示を切り替え");
                    tracing::info!("=== デバッグ形状表示 ===");
                    tracing::info!("s: クリップ空間正方形（単位行列テスト）");
                    tracing::info!("l: LineSegment3D表示");
                    tracing::info!("c: Circle3D表示");
                    tracing::info!("T: Triangle3D表示 (Shift+T)");
                    tracing::info!("a: Arc3D表示");
                    tracing::info!("n: NurbsCurve3D表示（GPU評価）");
                    tracing::info!("m: NurbsSurface3D表示（GPU評価）");
                    tracing::info!("o: Octree再分割表示/深さ送り（同一最終形状の粗→細）");
                    tracing::info!("Shift+O: Octree深さアニメーション再生（粗→細）");
                    tracing::info!("=== その他 ===");
                    tracing::info!(
                        "マウス操作: Ctrl+左ドラッグ=回転, Ctrl+中ドラッグ=パン, Ctrl+右ドラッグ=ズーム"
                    );
                }
                "w" => {
                    self.toggle_wireframe();
                }
                "q" => {
                    self.load_debug_clip_square();
                }
                "1" => {
                    self.set_stage_draft();
                    tracing::info!("ステージ切替: Draft");
                }
                "2" => {
                    self.set_stage_outline();
                    tracing::info!("ステージ切替: Outline");
                }
                "3" => {
                    self.set_stage_shading();
                    tracing::info!("ステージ切替: Shading");
                }
                "l" => {
                    self.load_debug_line();
                }
                "c" => {
                    self.load_debug_circle();
                }
                "a" => {
                    self.load_debug_arc();
                }
                "n" => {
                    self.load_debug_nurbs();
                }
                "m" => {
                    self.load_debug_nurbs_surface();
                }
                "o" => {
                    let mut handled = false;
                    {
                        let stage = self.renderer.get_stage_mut();
                        if let Some((current_depth, max_depth)) =
                            stage.cycle_octree_depth(&self.graphic.device)
                        {
                            tracing::info!("Octree深さ表示: {}/{}", current_depth, max_depth);
                            handled = true;
                        }
                    }

                    if !handled {
                        self.load_debug_octree();
                    }
                }
                "O" => {
                    let mut handled = false;
                    {
                        let stage = self.renderer.get_stage_mut();
                        if stage.start_octree_depth_animation() {
                            tracing::info!("Octree深さアニメーション再生開始");
                            handled = true;
                        }
                    }

                    if !handled {
                        self.load_debug_octree();
                        let stage = self.renderer.get_stage_mut();
                        if stage.start_octree_depth_animation() {
                            tracing::info!("Octree深さアニメーション再生開始");
                        }
                    }
                }
                "p" => {
                    self.load_sample_toolpath_only();
                }
                "S" => {
                    self.toggle_settings_panel();
                }
                "k" => {
                    self.cycle_debug_simulation_snapshot();
                }
                "j" => {
                    self.rewind_debug_simulation_snapshot();
                }
                "J" => {
                    self.stop_auto_snapshot_playback();
                }
                "+" | "=" => {
                    self.adjust_snapshot_playback_speed(0.1);
                }
                "-" | "_" => {
                    self.adjust_snapshot_playback_speed(-0.1);
                }
                "T" => {
                    self.load_debug_triangle();
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cam_demo::CamSimulationDemoScenario;
    use winit::keyboard::{Key, KeyCode, PhysicalKey};

    use super::super::snapshot_playback::{
        resolve_snapshot_load_decision, SnapshotLoadDecision, SnapshotLoadTrigger,
    };
    use super::{resolve_cam_demo_start_trigger_with_modifiers, CamDemoStartTrigger};

    #[test]
    fn resolve_cam_demo_start_trigger_accepts_only_shift_plus_uppercase_bf() {
        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("B".into()),
                &PhysicalKey::Code(KeyCode::KeyB),
                true,
            ),
            Some(CamDemoStartTrigger::BallEndMill)
        );
        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("F".into()),
                &PhysicalKey::Code(KeyCode::KeyF),
                true,
            ),
            Some(CamDemoStartTrigger::FlatEndMill)
        );

        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("B".into()),
                &PhysicalKey::Code(KeyCode::KeyB),
                false,
            ),
            None
        );
        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("b".into()),
                &PhysicalKey::Code(KeyCode::KeyB),
                true,
            ),
            None
        );
        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("F".into()),
                &PhysicalKey::Code(KeyCode::KeyB),
                true,
            ),
            None
        );
        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("k".into()),
                &PhysicalKey::Code(KeyCode::KeyK),
                true,
            ),
            None
        );
        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Named(winit::keyboard::NamedKey::Space),
                &PhysicalKey::Code(KeyCode::Space),
                true,
            ),
            None
        );
    }

    #[test]
    fn cam_demo_start_sequence_stays_not_ready_until_explicit_shift_start() {
        let ui_triggers = [
            SnapshotLoadTrigger::SnapshotScrub,
            SnapshotLoadTrigger::KeyK,
            SnapshotLoadTrigger::KeyJ,
            SnapshotLoadTrigger::Space,
            SnapshotLoadTrigger::Pause,
        ];

        let mut current_cam_demo_scenario = None;
        for trigger in ui_triggers {
            assert_eq!(
                resolve_snapshot_load_decision(trigger, current_cam_demo_scenario),
                SnapshotLoadDecision::NotReady {
                    trigger_label: trigger.label()
                }
            );
        }

        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("k".into()),
                &PhysicalKey::Code(KeyCode::KeyK),
                true,
            ),
            None
        );

        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("B".into()),
                &PhysicalKey::Code(KeyCode::KeyB),
                true,
            ),
            Some(CamDemoStartTrigger::BallEndMill)
        );
        current_cam_demo_scenario = Some(CamSimulationDemoScenario::Success);

        for trigger in ui_triggers {
            assert_eq!(
                resolve_snapshot_load_decision(trigger, current_cam_demo_scenario),
                SnapshotLoadDecision::Ready {
                    scenario: CamSimulationDemoScenario::Success,
                    trigger_label: trigger.label()
                }
            );
        }

        assert_eq!(
            resolve_cam_demo_start_trigger_with_modifiers(
                &Key::Character("F".into()),
                &PhysicalKey::Code(KeyCode::KeyF),
                true,
            ),
            Some(CamDemoStartTrigger::FlatEndMill)
        );
        current_cam_demo_scenario = Some(CamSimulationDemoScenario::SuccessFlatEndMill);

        for trigger in ui_triggers {
            assert_eq!(
                resolve_snapshot_load_decision(trigger, current_cam_demo_scenario),
                SnapshotLoadDecision::Ready {
                    scenario: CamSimulationDemoScenario::SuccessFlatEndMill,
                    trigger_label: trigger.label()
                }
            );
        }
    }
}
