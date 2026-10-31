//! AppState のキーボード入力ハンドリングを扱うモジュール。

use super::AppState;

impl AppState {
    /// キーボード入力を処理
    pub fn handle_keyboard_input(&mut self, key: &winit::keyboard::Key, pressed: bool) {
        self.mouse_input.update_key(key, pressed);

        if !pressed {
            return;
        }

        // 設定パネル・加工条件のウィンドウの入力欄の編集中は、打った文字をキー操作にしない
        if self.is_ui_window_open() && self.renderer.is_settings_using_keyboard() {
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
                "M" => {
                    self.toggle_machining_conditions();
                }
                "R" => {
                    self.run_cutting_simulation_jobs();
                }
                "L" => {
                    self.import_cutting_simulation();
                }
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
                    tracing::info!("=== 切削シミュレーション ===");
                    tracing::info!(
                        "【取込】 STL ファイルをウィンドウにドロップ: 製品形状として取り込み"
                    );
                    tracing::info!(
                        "【投入】 Shift+M: 加工条件のウィンドウの表示切替（「投入」で表示中の製品形状を投入）"
                    );
                    tracing::info!(
                        "【実行】 Shift+R: キューにあるジョブを実行（終わるまで画面は止まる）"
                    );
                    tracing::info!(
                        "【取込】 Shift+L: 最も新しい切削シミュレーションを取り込んで再生"
                    );
                    tracing::info!("【設定】 Shift+S: 設定パネル表示切替");
                    tracing::info!("【移動】 k: コマ送りの移動量だけ先のフレームへ（移動量は設定パネルの表示タブ）");
                    tracing::info!("【移動】 j: コマ送りの移動量だけ前のフレームへ");
                    tracing::info!(
                        "【移動】 . / , : 工具を次 / 前の区間の端点へ（素材は工具の手前の時点。タイトルに表示）"
                    );
                    tracing::info!("【移動】 左ドラッグ（左上進捗バー）: 任意フレームへスクラブ");
                    tracing::info!("【再生】 Space: 自動再生/一時停止");
                    tracing::info!("【再生】 Shift+J: 自動再生停止（先頭へ）");
                    tracing::info!(
                        "【再生】 + / - : 再生速度を 2 倍・半分（現在倍率はタイトル表示）"
                    );
                    tracing::info!("【表示】 w: ワイヤー表示 / ソリッド表示を切り替え");
                    tracing::info!("【表示】 v: ツールパスの線の表示 / 非表示を切り替え");
                    tracing::info!("【表示】 Shift+V: 工具の線の表示 / 非表示を切り替え");
                    tracing::info!("=== その他 ===");
                    tracing::info!("デバッグ用の表示: 設定パネル（Shift+S）のデバッグタブ");
                    tracing::info!("Esc: 終了");
                    tracing::info!(
                        "マウス操作: Ctrl+左ドラッグ=回転, Ctrl+中ドラッグ=パン, Ctrl+右ドラッグ=ズーム"
                    );
                }
                "w" => {
                    self.toggle_wireframe();
                }
                "v" => {
                    self.toggle_toolpath_lines();
                }
                "V" => {
                    self.toggle_tool_lines();
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
                "." => {
                    self.step_tool_to_segment_endpoint(true);
                }
                "," => {
                    self.step_tool_to_segment_endpoint(false);
                }
                "J" => {
                    self.stop_auto_snapshot_playback();
                }
                "+" | "=" => {
                    self.scale_snapshot_playback_speed(2.0);
                }
                "-" | "_" => {
                    self.scale_snapshot_playback_speed(0.5);
                }
                _ => {}
            }
        }
    }
}
