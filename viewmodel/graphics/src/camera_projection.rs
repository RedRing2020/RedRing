use crate::camera::ProjectionMode;
use analysis::linalg::matrix::Matrix4x4;
use std::f32::consts::PI;

pub(crate) fn projection_matrix(
    projection_mode: ProjectionMode,
    distance: f32,
    zoom: f32,
    orthographic_bounds: Option<(f32, f32, f32, f32)>,
    aspect: f32,
) -> [[f32; 4]; 4] {
    tracing::debug!(
        "projection_matrix呼び出し: mode={:?}, aspect={:.3}, bounds={:?}",
        projection_mode,
        aspect,
        orthographic_bounds
    );

    match projection_mode {
        ProjectionMode::Perspective => {
            let near = (distance * 0.01).max(0.001);
            let far = (distance * 100.0).min(1000.0);

            tracing::warn!("⚠️ 透視投影が使用されています！ CAM可視化では平行投影を使用すべきです");

            Matrix4x4::perspective_rh_01(45.0 * PI / 180.0, aspect, near, far).to_column_major()
        }
        ProjectionMode::Orthographic => {
            let (left, right, bottom, top) = if let Some((bl, br, bb, bt)) = orthographic_bounds {
                let logical_width = br - bl;
                let logical_height = bt - bb;
                let logical_aspect = logical_width / logical_height;

                if aspect > logical_aspect {
                    let actual_width = logical_height * aspect;
                    let expand = (actual_width - logical_width) * 0.5;
                    tracing::debug!(
                            "🔲 CAD表示モード（横長）: 論理範囲({}, {}, {}, {}) → 実表示範囲({:.1}, {:.1}, {:.1}, {:.1})",
                            bl, br, bb, bt,
                            bl - expand, br + expand, bb, bt
                        );
                    (bl - expand, br + expand, bb, bt)
                } else {
                    let actual_height = logical_width / aspect;
                    let expand = (actual_height - logical_height) * 0.5;
                    tracing::debug!(
                            "🔲 CAD表示モード（縦長）: 論理範囲({}, {}, {}, {}) → 実表示範囲({:.1}, {:.1}, {:.1}, {:.1})",
                            bl, br, bb, bt,
                            bl, br, bb - expand, bt + expand
                        );
                    (bl, br, bb - expand, bt + expand)
                }
            } else {
                let size = distance * zoom;
                let left = -size * aspect * 0.5;
                let right = size * aspect * 0.5;
                let bottom = -size * 0.5;
                let top = size * 0.5;
                tracing::debug!(
                    "平行投影: 距離ベース left={:.2}, right={:.2}, bottom={:.2}, top={:.2}",
                    left,
                    right,
                    bottom,
                    top
                );
                (left, right, bottom, top)
            };
            let near = -5000.0;
            let far = 5000.0;

            tracing::info!(
                "✓ 平行投影行列生成: bounds=({:.1}, {:.1}, {:.1}, {:.1}), near={}, far={}",
                left,
                right,
                bottom,
                top,
                near,
                far
            );

            orthographic_rh_01(left, right, bottom, top, near, far)
        }
    }
}

fn orthographic_rh_01(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let rl_inv = 1.0 / (right - left);
    let tb_inv = 1.0 / (top - bottom);
    let fn_inv = 1.0 / (far - near);

    [
        [2.0 * rl_inv, 0.0, 0.0, 0.0],
        [0.0, 2.0 * tb_inv, 0.0, 0.0],
        [0.0, 0.0, -fn_inv, 0.0],
        [
            -(right + left) * rl_inv,
            -(top + bottom) * tb_inv,
            -near * fn_inv,
            1.0,
        ],
    ]
}
