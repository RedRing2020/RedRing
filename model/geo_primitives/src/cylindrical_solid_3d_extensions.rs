//! CylindricalSolid3D の拡張機能
//!
//! メッシュ生成、変換、サンプリングなどの高度な操作
//! ソリッド特有の体積的操作と表面的操作を含む

use crate::{CylindricalSolid3D, Point3D};
use geo_contracts::Scalar;

/// CylindricalSolid3D の拡張実装
impl<T: Scalar> CylindricalSolid3D<T> {}

/// 質量特性を表す構造体
#[derive(Debug, Clone, PartialEq)]
pub struct MassProperties<T: Scalar> {
    /// 質量
    pub mass: T,
    /// 質量中心
    pub center_of_mass: Point3D<T>,
    /// ローカル座標系での慣性モーメント [Ixx, Iyy, Izz]
    pub inertia_local: [T; 3],
}
