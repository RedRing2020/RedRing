//! AppState のキーボード入力ハンドリングを扱うモジュール。

use super::AppState;

impl AppState {
    /// キーボード入力を処理
    pub fn handle_keyboard_input(&mut self, key: &winit::keyboard::Key, pressed: bool) {
        self.mouse_input.update_key(key, pressed);

        if !pressed {
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
                    tracing::info!("=== カメラ操作ヘルプ ===");
                    tracing::info!("r: カメラリセット");
                    tracing::info!("t: 標準CAD視点");
                    tracing::info!("f: 正面視点");
                    tracing::info!("e: 緊急脱出");
                    tracing::info!("w: ワイヤーフレーム切替");
                    tracing::info!("1: Draftステージ");
                    tracing::info!("2: Outlineステージ");
                    tracing::info!("3: Shadingステージ");
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
                    tracing::info!("p: カッターパスのみ表示（色分け線）");
                    tracing::info!(
                        "Shift+P: CAMシミュレーション可視化（ToolPath + ワーク + 除去）"
                    );
                    tracing::info!("k/j: Snapshotフレーム送り/巻き戻し");
                    tracing::info!("w: Octree表示モード切替（Wire/Solid）");
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
                    self.load_debug_toolpath_only();
                }
                "P" => {
                    self.load_debug_toolpath();
                }
                "k" => {
                    if self.debug_snapshot.series.is_some() {
                        self.cycle_debug_simulation_snapshot();
                    } else {
                        self.load_debug_simulation_snapshots();
                    }
                }
                "j" => {
                    if self.debug_snapshot.series.is_some() {
                        self.rewind_debug_simulation_snapshot();
                    } else {
                        self.load_debug_simulation_snapshots();
                    }
                }
                "T" => {
                    self.load_debug_triangle();
                }
                _ => {}
            }
        }
    }
}
