//! shape_converter - 幾何形状からGPU頂点データへの変換
//!
//! MVVMアーキテクチャにおけるViewModel層の責務として、
//! geo_primitives の各種形状を GPU レンダリング用の頂点データに変換します。
//!
//! ## アーキテクチャ設計
//!
//! ```text
//! Model層 (geo_primitives)
//!   LineSegment3D, Circle3D, Arc3D, etc.
//!          ↓
//! ViewModel層 (このモジュール)
//!   shape_to_vertices() - 形状種別に応じた変換
//!          ↓
//! View層 (render/stage)
//!   MeshVertex (GPU形式)
//! ```
//!
//! ## Foundation Patternとの統合
//!
//! `ExtensionFoundation` トレイトの `primitive_kind()` を活用して、
//! 型消去された形状オブジェクトを適切に変換します。

use crate::mesh_converter::VertexData;
use geo_foundation::{Circle3DProperties, PrimitiveKind};
use geo_primitives::{Circle3D, LineSegment3D, Point3D, Vector3D};
use std::f64::consts::PI;

/// テッセレーション品質パラメータ
#[derive(Debug, Clone)]
pub struct TessellationQuality {
    /// 線形形状の最小セグメント数
    pub min_segments: usize,

    /// 線形形状の最大セグメント数
    pub max_segments: usize,

    /// 円形状のデフォルトセグメント数
    pub circle_segments: usize,

    /// 球面のU方向分割数
    pub sphere_u_divisions: usize,

    /// 球面のV方向分割数
    pub sphere_v_divisions: usize,

    /// 平面グリッドの分割数
    pub plane_grid_size: usize,

    /// LOD有効化フラグ
    pub enable_lod: bool,

    /// LOD距離閾値
    pub lod_distance_threshold: f32,
}

impl Default for TessellationQuality {
    fn default() -> Self {
        Self {
            min_segments: 8,
            max_segments: 64,
            circle_segments: 32,
            sphere_u_divisions: 32,
            sphere_v_divisions: 16,
            plane_grid_size: 10,
            enable_lod: false,
            lod_distance_threshold: 100.0,
        }
    }
}

/// 形状変換エラー
#[derive(Debug, thiserror::Error)]
pub enum ShapeConversionError {
    #[error("Unsupported primitive kind: {0:?}")]
    UnsupportedPrimitive(PrimitiveKind),

    #[error("Invalid tessellation parameters")]
    InvalidTessellation,

    #[error("Shape has degenerate geometry")]
    DegenerateGeometry,
}

/// LineSegment3D を GPU用頂点データに変換
///
/// 線分は始点と終点の2頂点で表現されます。
/// 法線はゼロベクトルとして設定されます（線には法線が定義されないため）。
pub fn line_segment_to_vertices(segment: &LineSegment3D<f64>) -> Vec<VertexData> {
    // 始点と終点を取得
    let start = segment.start();
    let end = segment.end();

    // 法線はゼロベクトル（線には法線が定義されない）
    let normal = [0.0f32, 0.0, 0.0];

    vec![
        VertexData::new(
            [start.x() as f32, start.y() as f32, start.z() as f32],
            normal,
        ),
        VertexData::new([end.x() as f32, end.y() as f32, end.z() as f32], normal),
    ]
}

/// Circle3D を GPU用頂点データに変換（ワイヤーフレーム）
///
/// 円を多角形近似して頂点列を生成します。
/// セグメント数は品質パラメータで指定されます。
pub fn circle_to_vertices(
    circle: &Circle3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let segments = quality.circle_segments;
    let mut vertices = Vec::with_capacity(segments + 1); // +1 for closing the loop

    // 円の中心と半径を取得（タプルからPoint3Dに変換）
    let center_tuple = circle.center();
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = circle.radius();

    // 円の法線方向を取得（平面の法線）
    let normal_vec = circle.normal();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // 円の平面上で2つの直交する基底ベクトルを計算
    // 法線ベクトルに直交する任意のベクトルを見つける
    let arbitrary = if normal_vec.x().abs() > 0.9 {
        Vector3D::new(0.0, 1.0, 0.0)
    } else {
        Vector3D::new(1.0, 0.0, 0.0)
    };

    let u = normal_vec.cross(&arbitrary).normalize();
    let v = normal_vec.cross(&u).normalize();

    // 円周上の点を生成
    for i in 0..=segments {
        let angle = 2.0 * PI * (i as f64) / (segments as f64);
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 円周上の点 = center + radius * (cos(θ) * u + sin(θ) * v)
        let point_x = center.x() + radius * (cos_a * u.x() + sin_a * v.x());
        let point_y = center.y() + radius * (cos_a * u.y() + sin_a * v.y());
        let point_z = center.z() + radius * (cos_a * u.z() + sin_a * v.z());

        vertices.push(VertexData::new(
            [point_x as f32, point_y as f32, point_z as f32],
            normal,
        ));
    }

    vertices
}

/// Circle3D を GPU用頂点データに変換（ソリッド - 扇形分割）
///
/// 円を三角形の扇形として分割し、ソリッド表示用のメッシュを生成します。
/// 各三角形は中心点と円周上の2点で構成されます。
pub fn circle_to_solid_vertices(
    circle: &Circle3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let segments = quality.circle_segments;
    let mut vertices = Vec::with_capacity(segments * 3); // 各セグメントで3頂点（扇形）

    // 円の中心と半径を取得（タプルからPoint3Dに変換）
    let center_tuple = circle.center();
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = circle.radius();

    // 円の法線方向を取得
    let normal_vec = circle.normal();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // 円の平面上で2つの直交する基底ベクトルを計算
    let arbitrary = if normal_vec.x().abs() > 0.9 {
        Vector3D::new(0.0, 1.0, 0.0)
    } else {
        Vector3D::new(1.0, 0.0, 0.0)
    };

    let u = normal_vec.cross(&arbitrary).normalize();
    let v = normal_vec.cross(&u).normalize();

    // 中心点の頂点データ
    let center_vertex = VertexData::new(
        [center.x() as f32, center.y() as f32, center.z() as f32],
        normal,
    );

    // 扇形の三角形を生成
    for i in 0..segments {
        let angle1 = 2.0 * PI * (i as f64) / (segments as f64);
        let angle2 = 2.0 * PI * ((i + 1) as f64) / (segments as f64);

        // 円周上の2点を計算
        let point1 = Point3D::new(
            center.x() + radius * (angle1.cos() * u.x() + angle1.sin() * v.x()),
            center.y() + radius * (angle1.cos() * u.y() + angle1.sin() * v.y()),
            center.z() + radius * (angle1.cos() * u.z() + angle1.sin() * v.z()),
        );

        let point2 = Point3D::new(
            center.x() + radius * (angle2.cos() * u.x() + angle2.sin() * v.x()),
            center.y() + radius * (angle2.cos() * u.y() + angle2.sin() * v.y()),
            center.z() + radius * (angle2.cos() * u.z() + angle2.sin() * v.z()),
        );

        // 三角形の頂点を追加（CCW順序）
        vertices.push(center_vertex);
        vertices.push(VertexData::new(
            [point1.x() as f32, point1.y() as f32, point1.z() as f32],
            normal,
        ));
        vertices.push(VertexData::new(
            [point2.x() as f32, point2.y() as f32, point2.z() as f32],
            normal,
        ));
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_primitives::Direction3D;

    #[test]
    fn test_line_segment_conversion() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(1.0, 1.0, 1.0);
        let segment = LineSegment3D::new(start, end).unwrap();

        let vertices = line_segment_to_vertices(&segment);

        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_circle_conversion_wireframe() {
        // XY平面上の円（Z=0, 法線=(0,0,1)）
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let radius = 1.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        let quality = TessellationQuality::default();
        let vertices = circle_to_vertices(&circle, &quality);

        // セグメント数+1の頂点が生成される（最後の点は最初の点と同じで閉じる）
        assert_eq!(vertices.len(), quality.circle_segments + 1);

        // 最初と最後の点は同じはず（浮動小数点精度を考慮）
        let first_pos = vertices[0].position;
        let last_pos = vertices[vertices.len() - 1].position;
        assert!((first_pos[0] - last_pos[0]).abs() < 0.0001);
        assert!((first_pos[1] - last_pos[1]).abs() < 0.0001);
        assert!((first_pos[2] - last_pos[2]).abs() < 0.0001);

        // 全ての法線が(0, 0, 1)であることを確認
        for vertex in &vertices {
            assert_eq!(vertex.normal, [0.0, 0.0, 1.0]);
        }
    }

    #[test]
    fn test_circle_conversion_solid() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let radius = 1.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        let quality = TessellationQuality::default();
        let vertices = circle_to_solid_vertices(&circle, &quality);

        // セグメント数 * 3頂点（各扇形三角形）
        assert_eq!(vertices.len(), quality.circle_segments * 3);

        // 全ての法線が(0, 0, 1)であることを確認
        for vertex in &vertices {
            assert_eq!(vertex.normal, [0.0, 0.0, 1.0]);
        }
    }

    #[test]
    fn test_circle_radius_verification() {
        // 生成された頂点が実際に円周上にあることを確認
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let radius = 2.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        let quality = TessellationQuality {
            circle_segments: 16,
            ..Default::default()
        };
        let vertices = circle_to_vertices(&circle, &quality);

        // 各頂点が中心からradiusの距離にあることを確認
        for vertex in &vertices[..vertices.len() - 1] {
            // 最後の頂点は除く（閉じるための重複）
            let pos = vertex.position;
            let distance = ((pos[0] * pos[0]) + (pos[1] * pos[1]) + (pos[2] * pos[2])).sqrt();
            assert!((distance - radius as f32).abs() < 0.001);
        }
    }
}
