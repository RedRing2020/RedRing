//! ジョブマネージャーのアイコン（赤い輪。ウィンドウとタスクトレイに使う）

/// アイコンの大きさ（ピクセル）
pub(crate) const ICON_SIZE: u32 = 32;

/// 赤い輪の RGBA（上の行から、`ICON_SIZE` × `ICON_SIZE`）
pub(crate) fn ring_rgba() -> Vec<u8> {
    let size = ICON_SIZE as f32;
    let center = size / 2.0;
    let outer = size / 2.0 - 1.0;
    let inner = outer * 0.55;
    let mut rgba = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            // 輪の縁は 1 ピクセルでぼかす
            let coverage =
                (outer - distance + 0.5).clamp(0.0, 1.0) * (distance - inner + 0.5).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[220, 30, 40, (coverage * 255.0).round() as u8]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_is_opaque_on_the_ring_and_clear_in_the_middle_and_corners() {
        let rgba = ring_rgba();
        assert_eq!(rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
        let alpha = |x: u32, y: u32| rgba[((y * ICON_SIZE + x) * 4 + 3) as usize];
        assert_eq!(alpha(ICON_SIZE / 2, ICON_SIZE / 2), 0);
        assert_eq!(alpha(0, 0), 0);
        assert_eq!(alpha(ICON_SIZE / 2, 3), 255);
    }
}
