//! Overlay描画向けの座標変換ユーティリティ。

pub fn screen_to_ndc(x: f32, y: f32, viewport_width: u32, viewport_height: u32) -> [f32; 2] {
    let ndc_x = (x / viewport_width as f32) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / viewport_height as f32) * 2.0;
    [ndc_x, ndc_y]
}
