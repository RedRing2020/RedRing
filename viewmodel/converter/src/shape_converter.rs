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
use geo_foundation::{
    Arc3DProperties, Circle3DProperties, ConicalSurface3DProperties,
    CylindricalSurface3DProperties, EllipseArc3DProperties, EllipsoidalSurface3DProperties,
    InfiniteLine3DProperties, Plane3DProperties, Ray3DProperties, SphericalSurface3DProperties,
    PrimitiveKind, Triangle3DProperties,
};
use geo_primitives::{
    Arc3D, Circle3D, ConicalSurface3D, CylindricalSurface3D, Ellipse3D, EllipseArc3D,
    EllipsoidalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D,
    SphericalSurface3D, TorusSurface3D, Triangle3D, Vector3D,
};
use std::f64::consts::PI;

/// テッセレーション品質パラメータ
///
/// 現在は固定値を使用していますが、Phase 3では適応的品質計算を導入予定：
/// - カメラからの距離に応じたLOD切り替え
/// - 画面上のピクセルサイズに基づくセグメント数調整
/// - 形状の曲率に応じた密度調整
///
/// 詳細は dev/architecture/SHAPE_VISUALIZATION_DESIGN.md の
/// 「テッセレーション品質パラメータ改善提案」を参照。
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

    /// 平面グリッドの表示範囲（半径）
    pub plane_grid_extent: f64,

    /// 無限直線の表示範囲（両方向の長さ）
    pub infinite_line_extent: f64,

    /// 光線の表示範囲（一方向の長さ）
    pub ray_extent: f64,

    /// LOD有効化フラグ（Phase 3で使用予定）
    pub enable_lod: bool,

    /// LOD距離閾値（Phase 3で使用予定）
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
            plane_grid_extent: 10.0,
            infinite_line_extent: 100.0,
            ray_extent: 100.0,
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

/// Circle3D をワイヤーフレーム用LineList頂点データに変換
///
/// 円周を線分のリストとして表現します（LineListトポロジ用）。
/// 各セグメントは2頂点のペアとして生成されます。
pub fn circle_to_wireframe_line_segments(
    circle: &Circle3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let segments = quality.circle_segments;
    let mut vertices = Vec::with_capacity(segments * 2); // 各線分に2頂点

    // 円の中心と半径を取得
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

    // 円周を線分のペアとして生成
    for i in 0..segments {
        let angle1 = 2.0 * PI * (i as f64) / (segments as f64);
        let angle2 = 2.0 * PI * ((i + 1) as f64) / (segments as f64);

        // 始点
        let point1_x = center.x() + radius * (angle1.cos() * u.x() + angle1.sin() * v.x());
        let point1_y = center.y() + radius * (angle1.cos() * u.y() + angle1.sin() * v.y());
        let point1_z = center.z() + radius * (angle1.cos() * u.z() + angle1.sin() * v.z());

        // 終点
        let point2_x = center.x() + radius * (angle2.cos() * u.x() + angle2.sin() * v.x());
        let point2_y = center.y() + radius * (angle2.cos() * u.y() + angle2.sin() * v.y());
        let point2_z = center.z() + radius * (angle2.cos() * u.z() + angle2.sin() * v.z());

        // 線分として2頂点を追加
        vertices.push(VertexData::new(
            [point1_x as f32, point1_y as f32, point1_z as f32],
            normal,
        ));
        vertices.push(VertexData::new(
            [point2_x as f32, point2_y as f32, point2_z as f32],
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

/// Triangle3D を GPU用頂点データに変換（ワイヤーフレーム）
///
/// 三角形の3辺をワイヤーフレームとして表示します。
/// 各辺は2頂点の線分として表現されます。
pub fn triangle_to_wireframe_vertices(triangle: &Triangle3D<f64>) -> Vec<VertexData> {
    // 3つの頂点を取得（タプルからPoint3Dに変換）
    let va_tuple = triangle.vertex_a();
    let vb_tuple = triangle.vertex_b();
    let vc_tuple = triangle.vertex_c();

    let va = Point3D::new(va_tuple.0, va_tuple.1, va_tuple.2);
    let vb = Point3D::new(vb_tuple.0, vb_tuple.1, vb_tuple.2);
    let vc = Point3D::new(vc_tuple.0, vc_tuple.1, vc_tuple.2);

    // 法線を計算
    let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
    let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());
    let normal_vec = edge1.cross(&edge2).normalize();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // 3辺をワイヤーフレームとして生成（各辺2頂点）
    vec![
        // 辺 A-B
        VertexData::new([va.x() as f32, va.y() as f32, va.z() as f32], normal),
        VertexData::new([vb.x() as f32, vb.y() as f32, vb.z() as f32], normal),
        // 辺 B-C
        VertexData::new([vb.x() as f32, vb.y() as f32, vb.z() as f32], normal),
        VertexData::new([vc.x() as f32, vc.y() as f32, vc.z() as f32], normal),
        // 辺 C-A
        VertexData::new([vc.x() as f32, vc.y() as f32, vc.z() as f32], normal),
        VertexData::new([va.x() as f32, va.y() as f32, va.z() as f32], normal),
    ]
}

/// Triangle3D を GPU用頂点データに変換（ソリッド）
///
/// 三角形を塗りつぶして表示します。
/// CCW（反時計回り）順序で頂点を配置し、法線を計算します。
pub fn triangle_to_solid_vertices(triangle: &Triangle3D<f64>) -> Vec<VertexData> {
    // 3つの頂点を取得（タプルからPoint3Dに変換）
    let va_tuple = triangle.vertex_a();
    let vb_tuple = triangle.vertex_b();
    let vc_tuple = triangle.vertex_c();

    let va = Point3D::new(va_tuple.0, va_tuple.1, va_tuple.2);
    let vb = Point3D::new(vb_tuple.0, vb_tuple.1, vb_tuple.2);
    let vc = Point3D::new(vc_tuple.0, vc_tuple.1, vc_tuple.2);

    // 法線を計算（CCW順序を前提）
    let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
    let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());
    let normal_vec = edge1.cross(&edge2).normalize();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // 3頂点を配置
    vec![
        VertexData::new([va.x() as f32, va.y() as f32, va.z() as f32], normal),
        VertexData::new([vb.x() as f32, vb.y() as f32, vb.z() as f32], normal),
        VertexData::new([vc.x() as f32, vc.y() as f32, vc.z() as f32], normal),
    ]
}

/// Triangle3D をワイヤーフレーム用LineList頂点データに変換
///
/// 三角形の3辺を線分のリストとして表現します（LineListトポロジ用）。
/// 各辺は2頂点のペアとして生成されます。
pub fn triangle_to_wireframe_line_segments(triangle: &Triangle3D<f64>) -> Vec<VertexData> {
    // 3つの頂点を取得
    let va_tuple = triangle.vertex_a();
    let vb_tuple = triangle.vertex_b();
    let vc_tuple = triangle.vertex_c();

    let va = Point3D::new(va_tuple.0, va_tuple.1, va_tuple.2);
    let vb = Point3D::new(vb_tuple.0, vb_tuple.1, vb_tuple.2);
    let vc = Point3D::new(vc_tuple.0, vc_tuple.1, vc_tuple.2);

    // 法線を計算（CCW順序を前提）
    let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
    let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());
    let normal_vec = edge1.cross(&edge2).normalize();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    let va_f32 = [va.x() as f32, va.y() as f32, va.z() as f32];
    let vb_f32 = [vb.x() as f32, vb.y() as f32, vb.z() as f32];
    let vc_f32 = [vc.x() as f32, vc.y() as f32, vc.z() as f32];

    // 3辺を線分として生成（各辺に2頂点）
    vec![
        // 辺 A-B
        VertexData::new(va_f32, normal),
        VertexData::new(vb_f32, normal),
        // 辺 B-C
        VertexData::new(vb_f32, normal),
        VertexData::new(vc_f32, normal),
        // 辺 C-A
        VertexData::new(vc_f32, normal),
        VertexData::new(va_f32, normal),
    ]
}

/// Arc3D を GPU用頂点データに変換（ワイヤーフレーム）
///
/// 円弧をセグメント分割して頂点列を生成します。
/// 開始角度から終了角度までの範囲のみを表示します。
pub fn arc_to_vertices(arc: &Arc3D<f64>, quality: &TessellationQuality) -> Vec<VertexData> {
    // 円弧の角度範囲を取得（Arc3DPropertiesトレイトを使用してf64を取得）
    let start_angle: f64 = Arc3DProperties::start_angle(arc);
    let end_angle: f64 = Arc3DProperties::end_angle(arc);

    // 角度範囲を計算（CCW方向）
    let mut angle_range: f64 = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += 2.0 * PI;
    }

    // 角度範囲に基づいてセグメント数を決定
    let segments = ((angle_range / (2.0 * PI)) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;

    let mut vertices = Vec::with_capacity(segments + 1);

    // 円弧の中心と半径を取得（Arc3DPropertiesトレイトを使用）
    let center: (f64, f64, f64) = Arc3DProperties::center(arc);
    let radius: f64 = Arc3DProperties::radius(arc);

    // 円弧の法線方向と開始方向を取得（Arc3D内部メソッドを使用）
    let normal_dir = arc.normal();
    let start_dir = arc.start_direction();

    // Direction3Dから Vector3Dへ変換
    let normal_vec = normal_dir.as_vector();
    let start_dir_vec = start_dir.as_vector();

    let normal_f32 = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // V方向 = normal × start_dir
    let v = normal_vec.cross(&start_dir_vec).normalize();

    // 円弧上の点を生成
    for i in 0..=segments {
        let t = (i as f64) / (segments as f64);
        let angle = start_angle + t * angle_range;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 円弧上の点 = center + radius * (cos(θ) * start_dir + sin(θ) * v)
        let point_x = center.0 + radius * (cos_a * start_dir_vec.x() + sin_a * v.x());
        let point_y = center.1 + radius * (cos_a * start_dir_vec.y() + sin_a * v.y());
        let point_z = center.2 + radius * (cos_a * start_dir_vec.z() + sin_a * v.z());

        vertices.push(VertexData::new(
            [point_x as f32, point_y as f32, point_z as f32],
            normal_f32,
        ));
    }

    vertices
}

/// Arc3D をワイヤーフレーム用LineList頂点データに変換
///
/// 円弧を線分のリストとして表現します（LineListトポロジ用）。
/// 各セグメントは2頂点のペアとして生成されます。
pub fn arc_to_wireframe_line_segments(
    arc: &Arc3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    // 円弧の角度範囲を取得
    let start_angle: f64 = Arc3DProperties::start_angle(arc);
    let end_angle: f64 = Arc3DProperties::end_angle(arc);

    // 角度範囲を計算（CCW方向）
    let mut angle_range: f64 = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += 2.0 * PI;
    }

    // 角度範囲に基づいてセグメント数を決定
    let segments = ((angle_range / (2.0 * PI)) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;

    let mut vertices = Vec::with_capacity(segments * 2); // 各線分に2頂点

    // 円弧の中心と半径を取得
    let center: (f64, f64, f64) = Arc3DProperties::center(arc);
    let radius: f64 = Arc3DProperties::radius(arc);

    // 円弧の法線方向と開始方向を取得
    let normal_dir = arc.normal();
    let start_dir = arc.start_direction();

    // Direction3Dから Vector3Dへ変換
    let normal_vec = normal_dir.as_vector();
    let start_dir_vec = start_dir.as_vector();

    let normal_f32 = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // V方向 = normal × start_dir
    let v = normal_vec.cross(&start_dir_vec).normalize();

    // 円弧を線分のペアとして生成
    for i in 0..segments {
        let t1 = (i as f64) / (segments as f64);
        let t2 = ((i + 1) as f64) / (segments as f64);

        let angle1 = start_angle + t1 * angle_range;
        let angle2 = start_angle + t2 * angle_range;

        // 始点
        let cos_a1 = angle1.cos();
        let sin_a1 = angle1.sin();
        let point1_x = center.0 + radius * (cos_a1 * start_dir_vec.x() + sin_a1 * v.x());
        let point1_y = center.1 + radius * (cos_a1 * start_dir_vec.y() + sin_a1 * v.y());
        let point1_z = center.2 + radius * (cos_a1 * start_dir_vec.z() + sin_a1 * v.z());

        // 終点
        let cos_a2 = angle2.cos();
        let sin_a2 = angle2.sin();
        let point2_x = center.0 + radius * (cos_a2 * start_dir_vec.x() + sin_a2 * v.x());
        let point2_y = center.1 + radius * (cos_a2 * start_dir_vec.y() + sin_a2 * v.y());
        let point2_z = center.2 + radius * (cos_a2 * start_dir_vec.z() + sin_a2 * v.z());

        // 線分として2頂点を追加
        vertices.push(VertexData::new(
            [point1_x as f32, point1_y as f32, point1_z as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [point2_x as f32, point2_y as f32, point2_z as f32],
            normal_f32,
        ));
    }

    vertices
}

/// Plane3D を GPU用頂点データに変換（グリッド表示）
///
/// 平面を有限範囲のグリッド線として表示します。
/// グリッドは平面のU軸（第一軸）とV軸（第二軸）に沿って配置されます。
///
/// # Arguments
/// * `plane` - 平面（原点と座標軸を持つ）
/// * `quality` - テッセレーション品質パラメータ
///
/// # Returns
/// グリッド線の頂点データ（LineListトポロジ用）
pub fn plane_to_grid_vertices(
    plane: &Plane3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let grid_size = quality.plane_grid_size;
    let extent = quality.plane_grid_extent;
    let step = extent * 2.0 / grid_size as f64;

    let origin_tuple = Plane3DProperties::origin(plane);
    let u_axis_tuple = Plane3DProperties::u_axis(plane);
    let v_axis_tuple = Plane3DProperties::v_axis(plane);
    let normal_tuple = Plane3DProperties::normal(plane);

    let origin = Point3D::new(origin_tuple.0, origin_tuple.1, origin_tuple.2);
    let u_axis = Vector3D::new(u_axis_tuple.0, u_axis_tuple.1, u_axis_tuple.2);
    let v_axis = Vector3D::new(v_axis_tuple.0, v_axis_tuple.1, v_axis_tuple.2);
    let normal = Vector3D::new(normal_tuple.0, normal_tuple.1, normal_tuple.2);

    let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

    // グリッド線数は (grid_size + 1) × 2方向
    let mut vertices = Vec::with_capacity((grid_size + 1) * 4 * 2);

    // U方向の線（V軸に平行）を生成
    for i in 0..=grid_size {
        let u = -extent + step * i as f64;

        // 始点: (u, -extent)
        let start = Point3D::new(
            origin.x() + u * u_axis.x() - extent * v_axis.x(),
            origin.y() + u * u_axis.y() - extent * v_axis.y(),
            origin.z() + u * u_axis.z() - extent * v_axis.z(),
        );

        // 終点: (u, +extent)
        let end = Point3D::new(
            origin.x() + u * u_axis.x() + extent * v_axis.x(),
            origin.y() + u * u_axis.y() + extent * v_axis.y(),
            origin.z() + u * u_axis.z() + extent * v_axis.z(),
        );

        vertices.push(VertexData::new(
            [start.x() as f32, start.y() as f32, start.z() as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [end.x() as f32, end.y() as f32, end.z() as f32],
            normal_f32,
        ));
    }

    // V方向の線（U軸に平行）を生成
    for i in 0..=grid_size {
        let v = -extent + step * i as f64;

        // 始点: (-extent, v)
        let start = Point3D::new(
            origin.x() - extent * u_axis.x() + v * v_axis.x(),
            origin.y() - extent * u_axis.y() + v * v_axis.y(),
            origin.z() - extent * u_axis.z() + v * v_axis.z(),
        );

        // 終点: (+extent, v)
        let end = Point3D::new(
            origin.x() + extent * u_axis.x() + v * v_axis.x(),
            origin.y() + extent * u_axis.y() + v * v_axis.y(),
            origin.z() + extent * u_axis.z() + v * v_axis.z(),
        );

        vertices.push(VertexData::new(
            [start.x() as f32, start.y() as f32, start.z() as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [end.x() as f32, end.y() as f32, end.z() as f32],
            normal_f32,
        ));
    }

    vertices
}

/// Ellipse3D を GPU用頂点データに変換（ワイヤーフレーム）
///
/// 楕円を多角形近似して頂点列を生成します。
/// セグメント数は品質パラメータで指定されます。
pub fn ellipse_to_vertices(
    ellipse: &Ellipse3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let segments = quality.circle_segments;
    let mut vertices = Vec::with_capacity(segments + 1); // +1 for closing the loop

    let center = ellipse.center();
    let semi_major = ellipse.semi_major_axis();
    let semi_minor = ellipse.semi_minor_axis();
    let normal = ellipse.normal().as_vector();
    let major_dir = ellipse.major_axis_direction().as_vector();
    let minor_dir = ellipse.minor_axis_direction().as_vector();

    let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

    // 楕円周上の点を生成
    for i in 0..=segments {
        let angle = 2.0 * PI * (i as f64) / (segments as f64);
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 楕円周上の点 = center + a*cos(θ)*major_dir + b*sin(θ)*minor_dir
        let point = Point3D::new(
            center.x() + semi_major * cos_a * major_dir.x() + semi_minor * sin_a * minor_dir.x(),
            center.y() + semi_major * cos_a * major_dir.y() + semi_minor * sin_a * minor_dir.y(),
            center.z() + semi_major * cos_a * major_dir.z() + semi_minor * sin_a * minor_dir.z(),
        );

        vertices.push(VertexData::new(
            [point.x() as f32, point.y() as f32, point.z() as f32],
            normal_f32,
        ));
    }

    vertices
}

/// Ellipse3D をワイヤーフレーム用LineList頂点データに変換
///
/// 楕円周を線分のリストとして表現します（LineListトポロジ用）。
pub fn ellipse_to_wireframe_line_segments(
    ellipse: &Ellipse3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let segments = quality.circle_segments;
    let mut vertices = Vec::with_capacity(segments * 2);

    let center = ellipse.center();
    let semi_major = ellipse.semi_major_axis();
    let semi_minor = ellipse.semi_minor_axis();
    let normal = ellipse.normal().as_vector();
    let major_dir = ellipse.major_axis_direction().as_vector();
    let minor_dir = ellipse.minor_axis_direction().as_vector();

    let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

    // 楕円周を線分のペアとして生成
    for i in 0..segments {
        let angle1 = 2.0 * PI * (i as f64) / (segments as f64);
        let angle2 = 2.0 * PI * ((i + 1) as f64) / (segments as f64);

        // 始点
        let point1 = Point3D::new(
            center.x()
                + semi_major * angle1.cos() * major_dir.x()
                + semi_minor * angle1.sin() * minor_dir.x(),
            center.y()
                + semi_major * angle1.cos() * major_dir.y()
                + semi_minor * angle1.sin() * minor_dir.y(),
            center.z()
                + semi_major * angle1.cos() * major_dir.z()
                + semi_minor * angle1.sin() * minor_dir.z(),
        );

        // 終点
        let point2 = Point3D::new(
            center.x()
                + semi_major * angle2.cos() * major_dir.x()
                + semi_minor * angle2.sin() * minor_dir.x(),
            center.y()
                + semi_major * angle2.cos() * major_dir.y()
                + semi_minor * angle2.sin() * minor_dir.y(),
            center.z()
                + semi_major * angle2.cos() * major_dir.z()
                + semi_minor * angle2.sin() * minor_dir.z(),
        );

        vertices.push(VertexData::new(
            [point1.x() as f32, point1.y() as f32, point1.z() as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [point2.x() as f32, point2.y() as f32, point2.z() as f32],
            normal_f32,
        ));
    }

    vertices
}

/// EllipseArc3D を GPU用頂点データに変換（ワイヤーフレーム）
///
/// 楕円弧をセグメント分割して頂点列を生成します。
pub fn ellipse_arc_to_vertices(
    arc: &EllipseArc3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    // 楕円弧の角度範囲を取得
    let start_angle: f64 = EllipseArc3DProperties::start_angle(arc);
    let end_angle: f64 = EllipseArc3DProperties::end_angle(arc);

    // 角度範囲を計算（CCW方向）
    let mut angle_range: f64 = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += 2.0 * PI;
    }

    // 角度範囲に基づいてセグメント数を決定
    let segments = ((angle_range / (2.0 * PI)) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;

    let mut vertices = Vec::with_capacity(segments + 1);

    let ellipse = arc.ellipse();
    let center = ellipse.center();
    let semi_major = ellipse.semi_major_axis();
    let semi_minor = ellipse.semi_minor_axis();
    let normal = ellipse.normal().as_vector();
    let major_dir = ellipse.major_axis_direction().as_vector();
    let minor_dir = ellipse.minor_axis_direction().as_vector();

    let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

    // 楕円弧上の点を生成
    for i in 0..=segments {
        let t = (i as f64) / (segments as f64);
        let angle = start_angle + t * angle_range;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 楕円弧上の点
        let point = Point3D::new(
            center.x() + semi_major * cos_a * major_dir.x() + semi_minor * sin_a * minor_dir.x(),
            center.y() + semi_major * cos_a * major_dir.y() + semi_minor * sin_a * minor_dir.y(),
            center.z() + semi_major * cos_a * major_dir.z() + semi_minor * sin_a * minor_dir.z(),
        );

        vertices.push(VertexData::new(
            [point.x() as f32, point.y() as f32, point.z() as f32],
            normal_f32,
        ));
    }

    vertices
}

/// EllipseArc3D をワイヤーフレーム用LineList頂点データに変換
///
/// 楕円弧を線分のリストとして表現します（LineListトポロジ用）。
pub fn ellipse_arc_to_wireframe_line_segments(
    arc: &EllipseArc3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    // 角度範囲を取得
    let start_angle: f64 = EllipseArc3DProperties::start_angle(arc);
    let end_angle: f64 = EllipseArc3DProperties::end_angle(arc);

    let mut angle_range: f64 = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += 2.0 * PI;
    }

    let segments = ((angle_range / (2.0 * PI)) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;

    let mut vertices = Vec::with_capacity(segments * 2);

    let ellipse = arc.ellipse();
    let center = ellipse.center();
    let semi_major = ellipse.semi_major_axis();
    let semi_minor = ellipse.semi_minor_axis();
    let normal = ellipse.normal().as_vector();
    let major_dir = ellipse.major_axis_direction().as_vector();
    let minor_dir = ellipse.minor_axis_direction().as_vector();

    let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

    // 楕円弧を線分として生成
    for i in 0..segments {
        let t1 = (i as f64) / (segments as f64);
        let t2 = ((i + 1) as f64) / (segments as f64);

        let angle1 = start_angle + t1 * angle_range;
        let angle2 = start_angle + t2 * angle_range;

        // 始点
        let point1 = Point3D::new(
            center.x()
                + semi_major * angle1.cos() * major_dir.x()
                + semi_minor * angle1.sin() * minor_dir.x(),
            center.y()
                + semi_major * angle1.cos() * major_dir.y()
                + semi_minor * angle1.sin() * minor_dir.y(),
            center.z()
                + semi_major * angle1.cos() * major_dir.z()
                + semi_minor * angle1.sin() * minor_dir.z(),
        );

        // 終点
        let point2 = Point3D::new(
            center.x()
                + semi_major * angle2.cos() * major_dir.x()
                + semi_minor * angle2.sin() * minor_dir.x(),
            center.y()
                + semi_major * angle2.cos() * major_dir.y()
                + semi_minor * angle2.sin() * minor_dir.y(),
            center.z()
                + semi_major * angle2.cos() * major_dir.z()
                + semi_minor * angle2.sin() * minor_dir.z(),
        );

        vertices.push(VertexData::new(
            [point1.x() as f32, point1.y() as f32, point1.z() as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [point2.x() as f32, point2.y() as f32, point2.z() as f32],
            normal_f32,
        ));
    }

    vertices
}

/// Ray3D を GPU用頂点データに変換（有限長表示）
///
/// 光線を起点から一方向へ延びる有限長の線分として表示します。
///
/// # Arguments
/// * `ray` - 光線
/// * `quality` - テッセレーション品質パラメータ（ray_extentを使用）
///
/// # Returns
/// 2頂点の線分データ（LineListトポロジ用）
pub fn ray_to_vertices(ray: &Ray3D<f64>, quality: &TessellationQuality) -> Vec<VertexData> {
    let origin_point = Ray3DProperties::origin(ray);
    let direction_vec = Ray3DProperties::direction(ray);
    let extent = quality.ray_extent;

    let origin = Point3D::new(origin_point.x(), origin_point.y(), origin_point.z());
    let direction = Vector3D::new(direction_vec.x(), direction_vec.y(), direction_vec.z());

    // 終点 = 起点 + 方向 × 表示範囲
    let end = Point3D::new(
        origin.x() + direction.x() * extent,
        origin.y() + direction.y() * extent,
        origin.z() + direction.z() * extent,
    );

    // 法線はゼロベクトル（線には法線が定義されない）
    let normal = [0.0f32, 0.0, 0.0];

    vec![
        VertexData::new(
            [origin.x() as f32, origin.y() as f32, origin.z() as f32],
            normal,
        ),
        VertexData::new([end.x() as f32, end.y() as f32, end.z() as f32], normal),
    ]
}

/// InfiniteLine3D を GPU用頂点データに変換（有限長表示）
///
/// 無限直線を両方向へ延びる有限長の線分として表示します。
///
/// # Arguments
/// * `line` - 無限直線
/// * `quality` - テッセレーション品質パラメータ（infinite_line_extentを使用）
///
/// # Returns
/// 2頂点の線分データ（LineListトポロジ用）
pub fn infinite_line_to_vertices(
    line: &InfiniteLine3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let point_tuple = InfiniteLine3DProperties::point(line);
    let direction_tuple = InfiniteLine3DProperties::direction(line);
    let extent = quality.infinite_line_extent;

    let point = Point3D::new(point_tuple.0, point_tuple.1, point_tuple.2);
    let direction = Vector3D::new(direction_tuple.0, direction_tuple.1, direction_tuple.2);

    // 始点 = 点 - 方向 × 表示範囲
    let start = Point3D::new(
        point.x() - direction.x() * extent,
        point.y() - direction.y() * extent,
        point.z() - direction.z() * extent,
    );

    // 終点 = 点 + 方向 × 表示範囲
    let end = Point3D::new(
        point.x() + direction.x() * extent,
        point.y() + direction.y() * extent,
        point.z() + direction.z() * extent,
    );

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

/// CylindricalSurface3D を GPU用頂点データに変換（ソリッド）
///
/// 円筒面をUVパラメトリック分割して三角形メッシュを生成します。
///
/// # Arguments
/// * `surface` - 円筒面
/// * `quality` - テッセレーション品質パラメータ
///
/// # Returns
/// 三角形メッシュの頂点データ（TriangleListトポロジ用）
pub fn cylindrical_surface_to_vertices(
    surface: &CylindricalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let u_divisions = quality.circle_segments; // 円周方向
    let v_divisions = quality.sphere_v_divisions; // 高さ方向
    let height = 10.0; // デフォルト高さ（将来的にパラメータ化）

    let center_tuple = CylindricalSurface3DProperties::center(surface);
    let axis_tuple = CylindricalSurface3DProperties::axis(surface);
    let ref_tuple = CylindricalSurface3DProperties::ref_direction(surface);
    let radius = CylindricalSurface3DProperties::radius(surface);

    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let axis = Vector3D::new(axis_tuple.0, axis_tuple.1, axis_tuple.2);
    let ref_dir = Vector3D::new(ref_tuple.0, ref_tuple.1, ref_tuple.2);

    // Y軸 = Z軸 × X軸
    let y_axis = axis.cross(&ref_dir).normalize();

    let mut vertices = Vec::new();

    // UV グリッドで円筒面を生成
    for i in 0..v_divisions {
        for j in 0..u_divisions {
            let v1 = (i as f64 / v_divisions as f64) * height - height / 2.0;
            let v2 = ((i + 1) as f64 / v_divisions as f64) * height - height / 2.0;
            let u1 = (j as f64 / u_divisions as f64) * 2.0 * PI;
            let u2 = ((j + 1) as f64 / u_divisions as f64) * 2.0 * PI;

            // 4頂点を計算
            let p1 = calculate_cylinder_point(&center, &ref_dir, &y_axis, &axis, radius, u1, v1);
            let p2 = calculate_cylinder_point(&center, &ref_dir, &y_axis, &axis, radius, u2, v1);
            let p3 = calculate_cylinder_point(&center, &ref_dir, &y_axis, &axis, radius, u2, v2);
            let p4 = calculate_cylinder_point(&center, &ref_dir, &y_axis, &axis, radius, u1, v2);

            // 法線（外向き）
            let n1 = calculate_cylinder_normal(&ref_dir, &y_axis, u1);
            let n2 = calculate_cylinder_normal(&ref_dir, &y_axis, u2);

            // 2つの三角形に分割
            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n2));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n2));

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n2));
            vertices.push(VertexData::new([p4.x() as f32, p4.y() as f32, p4.z() as f32], n1));
        }
    }

    vertices
}

#[inline]
fn calculate_cylinder_point(
    center: &Point3D<f64>,
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    radius: f64,
    u: f64,
    v: f64,
) -> Point3D<f64> {
    Point3D::new(
        center.x() + radius * (u.cos() * x_axis.x() + u.sin() * y_axis.x()) + v * z_axis.x(),
        center.y() + radius * (u.cos() * x_axis.y() + u.sin() * y_axis.y()) + v * z_axis.y(),
        center.z() + radius * (u.cos() * x_axis.z() + u.sin() * y_axis.z()) + v * z_axis.z(),
    )
}

#[inline]
fn calculate_cylinder_normal(
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    u: f64,
) -> [f32; 3] {
    let nx = u.cos() * x_axis.x() + u.sin() * y_axis.x();
    let ny = u.cos() * x_axis.y() + u.sin() * y_axis.y();
    let nz = u.cos() * x_axis.z() + u.sin() * y_axis.z();
    [nx as f32, ny as f32, nz as f32]
}

/// SphericalSurface3D を GPU用頂点データに変換（ソリッド）
///
/// 球面をUVパラメトリック分割して三角形メッシュを生成します。
pub fn spherical_surface_to_vertices(
    surface: &SphericalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let u_divisions = quality.sphere_u_divisions; // 経度方向
    let v_divisions = quality.sphere_v_divisions; // 緯度方向

    let center_tuple = SphericalSurface3DProperties::center(surface);
    let axis_tuple = SphericalSurface3DProperties::axis(surface);
    let ref_tuple = SphericalSurface3DProperties::ref_direction(surface);
    let radius = SphericalSurface3DProperties::radius(surface);

    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let z_axis = Vector3D::new(axis_tuple.0, axis_tuple.1, axis_tuple.2);
    let x_axis = Vector3D::new(ref_tuple.0, ref_tuple.1, ref_tuple.2);
    let y_axis = z_axis.cross(&x_axis).normalize();

    let mut vertices = Vec::new();

    // UV グリッドで球面を生成
    for i in 0..v_divisions {
        for j in 0..u_divisions {
            let v1 = (i as f64 / v_divisions as f64) * PI - PI / 2.0; // -π/2 to π/2
            let v2 = ((i + 1) as f64 / v_divisions as f64) * PI - PI / 2.0;
            let u1 = (j as f64 / u_divisions as f64) * 2.0 * PI; // 0 to 2π
            let u2 = ((j + 1) as f64 / u_divisions as f64) * 2.0 * PI;

            // 極点の退化処理
            if i == 0 {
                // 北極
                let p_pole = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u1, v2);
                let p2 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u1, v2);
                let p3 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u2, v2);

                let n_pole = calculate_sphere_normal(&x_axis, &y_axis, &z_axis, u1, v2);

                vertices.push(VertexData::new([p_pole.x() as f32, p_pole.y() as f32, p_pole.z() as f32], n_pole));
                vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n_pole));
                vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n_pole));
                continue;
            }

            if i == v_divisions - 1 {
                // 南極付近
                let p1 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u1, v1);
                let p2 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u2, v1);
                let p_pole = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u1, v2);

                let n1 = calculate_sphere_normal(&x_axis, &y_axis, &z_axis, u1, v1);

                vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
                vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n1));
                vertices.push(VertexData::new([p_pole.x() as f32, p_pole.y() as f32, p_pole.z() as f32], n1));
                continue;
            }

            // 通常の4頂点クワッド
            let p1 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u1, v1);
            let p2 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u2, v1);
            let p3 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u2, v2);
            let p4 = calculate_sphere_point(&center, &x_axis, &y_axis, &z_axis, radius, u1, v2);

            let n1 = calculate_sphere_normal(&x_axis, &y_axis, &z_axis, u1, v1);
            let n2 = calculate_sphere_normal(&x_axis, &y_axis, &z_axis, u2, v1);
            let n3 = calculate_sphere_normal(&x_axis, &y_axis, &z_axis, u2, v2);
            let n4 = calculate_sphere_normal(&x_axis, &y_axis, &z_axis, u1, v2);

            // 2つの三角形
            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n2));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n3));

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n3));
            vertices.push(VertexData::new([p4.x() as f32, p4.y() as f32, p4.z() as f32], n4));
        }
    }

    vertices
}

#[inline]
fn calculate_sphere_point(
    center: &Point3D<f64>,
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    radius: f64,
    u: f64,
    v: f64,
) -> Point3D<f64> {
    let cos_v = v.cos();
    Point3D::new(
        center.x() + radius * (cos_v * u.cos() * x_axis.x() + cos_v * u.sin() * y_axis.x() + v.sin() * z_axis.x()),
        center.y() + radius * (cos_v * u.cos() * x_axis.y() + cos_v * u.sin() * y_axis.y() + v.sin() * z_axis.y()),
        center.z() + radius * (cos_v * u.cos() * x_axis.z() + cos_v * u.sin() * y_axis.z() + v.sin() * z_axis.z()),
    )
}

#[inline]
fn calculate_sphere_normal(
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    u: f64,
    v: f64,
) -> [f32; 3] {
    let cos_v = v.cos();
    let nx = cos_v * u.cos() * x_axis.x() + cos_v * u.sin() * y_axis.x() + v.sin() * z_axis.x();
    let ny = cos_v * u.cos() * x_axis.y() + cos_v * u.sin() * y_axis.y() + v.sin() * z_axis.y();
    let nz = cos_v * u.cos() * x_axis.z() + cos_v * u.sin() * y_axis.z() + v.sin() * z_axis.z();
    [nx as f32, ny as f32, nz as f32]
}

/// ConicalSurface3D を GPU用頂点データに変換（ソリッド）
///
/// 円錐面をUVパラメトリック分割して三角形メッシュを生成します。
pub fn conical_surface_to_vertices(
    surface: &ConicalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let u_divisions = quality.circle_segments; // 円周方向
    let v_divisions = quality.sphere_v_divisions; // 高さ方向

    let apex_tuple = <ConicalSurface3D<f64> as ConicalSurface3DProperties<f64>>::apex(surface);
    let base_center_tuple = <ConicalSurface3D<f64> as ConicalSurface3DProperties<f64>>::base_center(surface);
    let radius = <ConicalSurface3D<f64> as ConicalSurface3DProperties<f64>>::radius(surface);
    let height = <ConicalSurface3D<f64> as ConicalSurface3DProperties<f64>>::height(surface);
    let axis_tuple = <ConicalSurface3D<f64> as ConicalSurface3DProperties<f64>>::axis(surface);
    let ref_tuple = <ConicalSurface3D<f64> as ConicalSurface3DProperties<f64>>::ref_direction(surface);

    let apex = Point3D::new(apex_tuple.0, apex_tuple.1, apex_tuple.2);
    let base_center = Point3D::new(base_center_tuple.0, base_center_tuple.1, base_center_tuple.2);
    let z_axis = Vector3D::new(axis_tuple.0, axis_tuple.1, axis_tuple.2);
    let x_axis = Vector3D::new(ref_tuple.0, ref_tuple.1, ref_tuple.2);
    let y_axis = z_axis.cross(&x_axis).normalize();

    // 頂点から底面へ向かう軸ベクトル
    let axis_dir = Vector3D::new(
        base_center.x() - apex.x(),
        base_center.y() - apex.y(),
        base_center.z() - apex.z(),
    ).normalize();

    // 半角の計算: tan(semi_angle) = radius / height
    let semi_angle = (radius / height).atan();

    let mut vertices = Vec::new();

    for i in 0..v_divisions {
        for j in 0..u_divisions {
            let v1 = i as f64 / v_divisions as f64; // 0 to 1 (apex to base)
            let v2 = (i + 1) as f64 / v_divisions as f64;
            let u1 = (j as f64 / u_divisions as f64) * 2.0 * PI;
            let u2 = ((j + 1) as f64 / u_divisions as f64) * 2.0 * PI;

            // v = 0が頂点、v = 1が底面
            let h1 = v1 * height;
            let h2 = v2 * height;
            let r1 = v1 * radius; // v=0で0, v=1でradius
            let r2 = v2 * radius;

            let p1 = calculate_cone_point_from_apex(&apex, &x_axis, &y_axis, &axis_dir, r1, u1, h1);
            let p2 = calculate_cone_point_from_apex(&apex, &x_axis, &y_axis, &axis_dir, r1, u2, h1);
            let p3 = calculate_cone_point_from_apex(&apex, &x_axis, &y_axis, &axis_dir, r2, u2, h2);
            let p4 = calculate_cone_point_from_apex(&apex, &x_axis, &y_axis, &axis_dir, r2, u1, h2);

            let n1 = calculate_cone_normal(&x_axis, &y_axis, &axis_dir, u1, semi_angle);
            let n2 = calculate_cone_normal(&x_axis, &y_axis, &axis_dir, u2, semi_angle);

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n2));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n2));

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n2));
            vertices.push(VertexData::new([p4.x() as f32, p4.y() as f32, p4.z() as f32], n1));
        }
    }

    vertices
}

#[inline]
fn calculate_cone_point_from_apex(
    apex: &Point3D<f64>,
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    axis_dir: &Vector3D<f64>,
    radius: f64,
    u: f64,
    h: f64,
) -> Point3D<f64> {
    Point3D::new(
        apex.x() + radius * (u.cos() * x_axis.x() + u.sin() * y_axis.x()) + h * axis_dir.x(),
        apex.y() + radius * (u.cos() * x_axis.y() + u.sin() * y_axis.y()) + h * axis_dir.y(),
        apex.z() + radius * (u.cos() * x_axis.z() + u.sin() * y_axis.z()) + h * axis_dir.z(),
    )
}

#[inline]
fn calculate_cone_normal(
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    axis_dir: &Vector3D<f64>,
    u: f64,
    semi_angle: f64,
) -> [f32; 3] {
    let cos_angle = semi_angle.cos();
    let sin_angle = semi_angle.sin();

    let radial_x = u.cos() * x_axis.x() + u.sin() * y_axis.x();
    let radial_y = u.cos() * x_axis.y() + u.sin() * y_axis.y();
    let radial_z = u.cos() * x_axis.z() + u.sin() * y_axis.z();

    let nx = cos_angle * radial_x - sin_angle * axis_dir.x();
    let ny = cos_angle * radial_y - sin_angle * axis_dir.y();
    let nz = cos_angle * radial_z - sin_angle * axis_dir.z();

    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    [
        (nx / len) as f32,
        (ny / len) as f32,
        (nz / len) as f32,
    ]
}

/// TorusSurface3D を GPU用頂点データに変換（ソリッド）
///
/// トーラス面をUVパラメトリック分割して三角形メッシュを生成します。
pub fn torus_surface_to_vertices(
    surface: &TorusSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let u_divisions = quality.circle_segments; // 主円周方向
    let v_divisions = quality.circle_segments / 2; // 副円周方向

    // TorusSurface3Dには直接アクセスするメソッドがないため、内部フィールドアクセスを使用
    // 注: これは一時的な実装で、将来的にトレイトを追加する必要があります
    let origin = surface.origin_internal();
    let z_axis_dir = surface.z_axis_internal();
    let x_axis_dir = surface.x_axis_internal();
    let major_radius = surface.major_radius_internal();
    let minor_radius = surface.minor_radius_internal();

    let z_axis = Vector3D::new(z_axis_dir.x(), z_axis_dir.y(), z_axis_dir.z());
    let x_axis = Vector3D::new(x_axis_dir.x(), x_axis_dir.y(), x_axis_dir.z());
    let y_axis = z_axis.cross(&x_axis).normalize();

    let mut vertices = Vec::new();

    for i in 0..u_divisions {
        for j in 0..v_divisions {
            let u1 = (i as f64 / u_divisions as f64) * 2.0 * PI;
            let u2 = ((i + 1) as f64 / u_divisions as f64) * 2.0 * PI;
            let v1 = (j as f64 / v_divisions as f64) * 2.0 * PI;
            let v2 = ((j + 1) as f64 / v_divisions as f64) * 2.0 * PI;

            let p1 = calculate_torus_point(&origin, &x_axis, &y_axis, &z_axis, major_radius, minor_radius, u1, v1);
            let p2 = calculate_torus_point(&origin, &x_axis, &y_axis, &z_axis, major_radius, minor_radius, u2, v1);
            let p3 = calculate_torus_point(&origin, &x_axis, &y_axis, &z_axis, major_radius, minor_radius, u2, v2);
            let p4 = calculate_torus_point(&origin, &x_axis, &y_axis, &z_axis, major_radius, minor_radius, u1, v2);

            let n1 = calculate_torus_normal(&x_axis, &y_axis, &z_axis, major_radius, minor_radius, u1, v1);
            let n2 = calculate_torus_normal(&x_axis, &y_axis, &z_axis, major_radius, minor_radius, u2, v1);
            let n3 = calculate_torus_normal(&x_axis, &y_axis, &z_axis, major_radius, minor_radius, u2, v2);
            let n4 = calculate_torus_normal(&x_axis, &y_axis, &z_axis, major_radius, minor_radius, u1, v2);

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n2));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n3));

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n3));
            vertices.push(VertexData::new([p4.x() as f32, p4.y() as f32, p4.z() as f32], n4));
        }
    }

    vertices
}

#[inline]
fn calculate_torus_point(
    origin: &Point3D<f64>,
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    major_radius: f64,
    minor_radius: f64,
    u: f64,
    v: f64,
) -> Point3D<f64> {
    let r = major_radius + minor_radius * v.cos();
    Point3D::new(
        origin.x() + r * (u.cos() * x_axis.x() + u.sin() * y_axis.x()) + minor_radius * v.sin() * z_axis.x(),
        origin.y() + r * (u.cos() * x_axis.y() + u.sin() * y_axis.y()) + minor_radius * v.sin() * z_axis.y(),
        origin.z() + r * (u.cos() * x_axis.z() + u.sin() * y_axis.z()) + minor_radius * v.sin() * z_axis.z(),
    )
}

#[inline]
fn calculate_torus_normal(
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    _major_radius: f64,
    _minor_radius: f64,
    u: f64,
    v: f64,
) -> [f32; 3] {
    let nx = v.cos() * (u.cos() * x_axis.x() + u.sin() * y_axis.x()) + v.sin() * z_axis.x();
    let ny = v.cos() * (u.cos() * x_axis.y() + u.sin() * y_axis.y()) + v.sin() * z_axis.y();
    let nz = v.cos() * (u.cos() * x_axis.z() + u.sin() * y_axis.z()) + v.sin() * z_axis.z();
    [nx as f32, ny as f32, nz as f32]
}

/// EllipsoidalSurface3D を GPU用頂点データに変換（ソリッド）
///
/// 楕円体面をUVパラメトリック分割して三角形メッシュを生成します。
pub fn ellipsoidal_surface_to_vertices(
    surface: &EllipsoidalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let u_divisions = quality.sphere_u_divisions; // 経度方向
    let v_divisions = quality.sphere_v_divisions; // 緯度方向

    let center_tuple = <EllipsoidalSurface3D<f64> as EllipsoidalSurface3DProperties<f64>>::center(surface);
    let axis_tuple = <EllipsoidalSurface3D<f64> as EllipsoidalSurface3DProperties<f64>>::axis(surface);
    let ref_tuple = <EllipsoidalSurface3D<f64> as EllipsoidalSurface3DProperties<f64>>::ref_direction(surface);
    let a_radius = <EllipsoidalSurface3D<f64> as EllipsoidalSurface3DProperties<f64>>::semi_axis_a(surface);
    let b_radius = <EllipsoidalSurface3D<f64> as EllipsoidalSurface3DProperties<f64>>::semi_axis_b(surface);
    let c_radius = <EllipsoidalSurface3D<f64> as EllipsoidalSurface3DProperties<f64>>::semi_axis_c(surface);

    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let z_axis = Vector3D::new(axis_tuple.0, axis_tuple.1, axis_tuple.2);
    let x_axis = Vector3D::new(ref_tuple.0, ref_tuple.1, ref_tuple.2);
    let y_axis = z_axis.cross(&x_axis).normalize();

    let mut vertices = Vec::new();

    for i in 0..v_divisions {
        for j in 0..u_divisions {
            let v1 = (i as f64 / v_divisions as f64) * PI - PI / 2.0;
            let v2 = ((i + 1) as f64 / v_divisions as f64) * PI - PI / 2.0;
            let u1 = (j as f64 / u_divisions as f64) * 2.0 * PI;
            let u2 = ((j + 1) as f64 / u_divisions as f64) * 2.0 * PI;

            // 極点の退化処理（球面と同様）
            if i == 0 || i == v_divisions - 1 {
                let (p1, p2, p3) = if i == 0 {
                    let pole = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v2);
                    let p2 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v2);
                    let p3 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u2, v2);
                    (pole, p2, p3)
                } else {
                    let p1 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v1);
                    let p2 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u2, v1);
                    let pole = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v2);
                    (p1, p2, pole)
                };

                let n = calculate_ellipsoid_normal(&x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, if i == 0 { v2 } else { v1 });

                vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n));
                vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n));
                vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n));
                continue;
            }

            let p1 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v1);
            let p2 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u2, v1);
            let p3 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u2, v2);
            let p4 = calculate_ellipsoid_point(&center, &x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v2);

            let n1 = calculate_ellipsoid_normal(&x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v1);
            let n2 = calculate_ellipsoid_normal(&x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u2, v1);
            let n3 = calculate_ellipsoid_normal(&x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u2, v2);
            let n4 = calculate_ellipsoid_normal(&x_axis, &y_axis, &z_axis, a_radius, b_radius, c_radius, u1, v2);

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p2.x() as f32, p2.y() as f32, p2.z() as f32], n2));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n3));

            vertices.push(VertexData::new([p1.x() as f32, p1.y() as f32, p1.z() as f32], n1));
            vertices.push(VertexData::new([p3.x() as f32, p3.y() as f32, p3.z() as f32], n3));
            vertices.push(VertexData::new([p4.x() as f32, p4.y() as f32, p4.z() as f32], n4));
        }
    }

    vertices
}

#[inline]
fn calculate_ellipsoid_point(
    center: &Point3D<f64>,
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    a: f64,
    b: f64,
    c: f64,
    u: f64,
    v: f64,
) -> Point3D<f64> {
    let cos_v = v.cos();
    Point3D::new(
        center.x() + a * cos_v * u.cos() * x_axis.x() + b * cos_v * u.sin() * y_axis.x() + c * v.sin() * z_axis.x(),
        center.y() + a * cos_v * u.cos() * x_axis.y() + b * cos_v * u.sin() * y_axis.y() + c * v.sin() * z_axis.y(),
        center.z() + a * cos_v * u.cos() * x_axis.z() + b * cos_v * u.sin() * y_axis.z() + c * v.sin() * z_axis.z(),
    )
}

#[inline]
fn calculate_ellipsoid_normal(
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    a: f64,
    b: f64,
    c: f64,
    u: f64,
    v: f64,
) -> [f32; 3] {
    let cos_v = v.cos();
    // 楕円体の法線 = (x/a², y/b², z/c²) を正規化
    let nx = (cos_v * u.cos() / a) * x_axis.x() + (cos_v * u.sin() / b) * y_axis.x() + (v.sin() / c) * z_axis.x();
    let ny = (cos_v * u.cos() / a) * x_axis.y() + (cos_v * u.sin() / b) * y_axis.y() + (v.sin() / c) * z_axis.y();
    let nz = (cos_v * u.cos() / a) * x_axis.z() + (cos_v * u.sin() / b) * y_axis.z() + (v.sin() / c) * z_axis.z();

    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    [
        (nx / len) as f32,
        (ny / len) as f32,
        (nz / len) as f32,
    ]
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

    #[test]
    fn test_triangle_wireframe_conversion() {
        // XY平面上の三角形
        let va = Point3D::new(0.0, 0.0, 0.0);
        let vb = Point3D::new(1.0, 0.0, 0.0);
        let vc = Point3D::new(0.5, 1.0, 0.0);
        let triangle = Triangle3D::new(va, vb, vc).unwrap();

        let vertices = triangle_to_wireframe_vertices(&triangle);

        // 3辺 × 2頂点 = 6頂点
        assert_eq!(vertices.len(), 6);

        // 辺A-Bの検証
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 0.0, 0.0]);

        // 辺B-Cの検証
        assert_eq!(vertices[2].position, [1.0, 0.0, 0.0]);
        assert_eq!(vertices[3].position, [0.5, 1.0, 0.0]);

        // 辺C-Aの検証
        assert_eq!(vertices[4].position, [0.5, 1.0, 0.0]);
        assert_eq!(vertices[5].position, [0.0, 0.0, 0.0]);

        // 全ての法線がZ方向（0, 0, 1）であることを確認
        for vertex in &vertices {
            assert!((vertex.normal[0] - 0.0).abs() < 0.001);
            assert!((vertex.normal[1] - 0.0).abs() < 0.001);
            assert!((vertex.normal[2] - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_triangle_solid_conversion() {
        // XY平面上の三角形
        let va = Point3D::new(0.0, 0.0, 0.0);
        let vb = Point3D::new(1.0, 0.0, 0.0);
        let vc = Point3D::new(0.0, 1.0, 0.0);
        let triangle = Triangle3D::new(va, vb, vc).unwrap();

        let vertices = triangle_to_solid_vertices(&triangle);

        // 3頂点
        assert_eq!(vertices.len(), 3);

        // 頂点位置の検証
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(vertices[2].position, [0.0, 1.0, 0.0]);

        // 全ての法線がZ方向（0, 0, 1）であることを確認
        for vertex in &vertices {
            assert!((vertex.normal[0] - 0.0).abs() < 0.001);
            assert!((vertex.normal[1] - 0.0).abs() < 0.001);
            assert!((vertex.normal[2] - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_arc_conversion() {
        use geo_primitives::Angle;

        // XY平面上の90度の円弧（0度から90度）
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 2.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(std::f64::consts::FRAC_PI_2); // 90度

        let arc = Arc3D::xy_arc(center, radius, start_angle, end_angle).unwrap();

        let quality = TessellationQuality {
            circle_segments: 16,
            min_segments: 4,
            ..Default::default()
        };

        let vertices = arc_to_vertices(&arc, &quality);

        // セグメント数は角度範囲に応じて調整される
        assert!(vertices.len() >= quality.min_segments);

        // 最初の点は(2, 0, 0)付近、最後の点は(0, 2, 0)付近であることを確認
        let first_pos = vertices[0].position;
        assert!((first_pos[0] - 2.0).abs() < 0.1);
        assert!((first_pos[1] - 0.0).abs() < 0.1);

        let last_pos = vertices[vertices.len() - 1].position;
        assert!((last_pos[0] - 0.0).abs() < 0.1);
        assert!((last_pos[1] - 2.0).abs() < 0.1);

        // 全ての法線がZ方向（0, 0, 1）であることを確認
        for vertex in &vertices {
            assert!((vertex.normal[0] - 0.0).abs() < 0.001);
            assert!((vertex.normal[1] - 0.0).abs() < 0.001);
            assert!((vertex.normal[2] - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_arc_radius_verification() {
        use geo_primitives::Angle;

        // 円弧上の全ての点が中心からradiusの距離にあることを確認
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 3.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(std::f64::consts::PI); // 180度

        let arc = Arc3D::xy_arc(center, radius, start_angle, end_angle).unwrap();

        let quality = TessellationQuality {
            circle_segments: 32,
            ..Default::default()
        };

        let vertices = arc_to_vertices(&arc, &quality);

        // 各頂点が中心からradiusの距離にあることを確認
        for vertex in &vertices {
            let pos = vertex.position;
            let distance = ((pos[0] * pos[0]) + (pos[1] * pos[1]) + (pos[2] * pos[2])).sqrt();
            assert!((distance - radius as f32).abs() < 0.01);
        }
    }
}
