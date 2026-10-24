//! Entity Core Traits
//!
//! エンティティの識別子・表示属性・形状の値を取り出す trait定義。

use crate::{Angle, Direction3D, Point3D, Scalar, TriangleMesh3DProperties};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrokePattern {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    DashDotDot,
    Hidden,
}

pub trait EntityIdentity {
    type Id: Copy + Eq + std::hash::Hash;

    fn entity_id(&self) -> Self::Id;
}

pub trait EntityDisplayProperties {
    fn entity_visible(&self) -> bool;
    fn entity_color(&self) -> [f32; 4];
}

pub trait StrokeDisplayProperties {
    fn entity_stroke_pattern(&self) -> StrokePattern;
    fn entity_stroke_width(&self) -> f32;
}

pub trait PlanarStroke2DProperties<T: Scalar> {
    /// 2D の線を配置する平面の原点
    fn definition_plane_origin(&self) -> Point3D<T>;
    /// 2D の線を配置する平面の法線
    fn definition_plane_normal(&self) -> Direction3D<T>;
    /// 2D の線を配置する平面の U 軸
    fn definition_plane_u_axis(&self) -> Direction3D<T>;
}

pub trait LineEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn line_start(&self) -> Point3D<T>;
    fn line_end(&self) -> Point3D<T>;
}

pub trait CircleEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn circle_center(&self) -> Point3D<T>;
    fn circle_radius(&self) -> T;
    /// 円が存在する平面の法線
    fn circle_axis(&self) -> Direction3D<T>;
}

pub trait ArcEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn arc_center(&self) -> Point3D<T>;
    fn arc_radius(&self) -> T;
    fn arc_start_angle(&self) -> Angle<T>;
    /// 開始角から反時計回りに終了角までを円弧の範囲とする
    fn arc_end_angle(&self) -> Angle<T>;
    /// 円弧平面の法線
    fn arc_normal(&self) -> Direction3D<T>;
    /// 角度 0 の方向
    fn arc_start_direction(&self) -> Direction3D<T>;
}

/// 面を持つエンティティ（シェル・面単体）
pub trait FaceSetEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    /// 面の ID（全体で一意）
    type FaceId: Copy + Eq + std::hash::Hash;
    /// 表示用の三角形メッシュ
    type DisplayMesh: TriangleMesh3DProperties<T>;
    /// 表示用の三角形を作れない理由
    type DisplayMeshError;

    /// 面の ID（面の並びの順）
    fn face_ids(&self) -> Vec<Self::FaceId>;

    /// 面 `face_id` の表示用の三角形（頂点法線つき。曲面は許容弦誤差 `chord_tolerance` で離散化する）
    fn face_display_mesh(
        &self,
        face_id: Self::FaceId,
        chord_tolerance: T,
    ) -> Result<Self::DisplayMesh, Self::DisplayMeshError>;
}
