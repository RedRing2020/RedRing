//! 幾何 extension の foundation trait群。
//!
//! このモジュールは、幾何プリミティブ向けの最小 extension 契約を定義します。
//! - `PrimitiveMetadata`: プリミティブ種別の metadata
//! - `ExtensionFoundation<T>`: metadata 向けの互換 surface
//! - `Bounded<T>`: 軸平行境界ボックス (AABB)

use crate::classification::PrimitiveKind;
use analysis::abstract_types::Scalar;

/// 幾何プリミティブ向けの metadata capability です。
///
/// この trait は、プリミティブ種別の識別を提供します。
pub trait PrimitiveMetadata {
    /// プリミティブ種別を返します。
    fn primitive_kind(&self) -> PrimitiveKind;
}

/// metadata 指向の foundation 境界向け互換 trait です。
pub trait ExtensionFoundation<T: Scalar = f64>: PrimitiveMetadata {}

impl<T: Scalar, TPrimitive: PrimitiveMetadata + ?Sized> ExtensionFoundation<T> for TPrimitive {}

/// 軸平行境界ボックス (AABB) を持つ幾何プリミティブ向け trait です。
///
/// # 実装対象
///
/// Circle、Triangle、NURBS のような空間的広がりを持つプリミティブが実装します。
/// Point、Vector、Direction のような基本要素では通常不要です。
///
/// # AABB 型
///
/// 具体的な AABB 型 (`geo_core::Aabb2D`, `geo_core::Aabb3D`) は実装側が決めます。
/// この trait はその契約だけを定義します。
pub trait Bounded<T: Scalar = f64>: PrimitiveMetadata {
    /// AABB 型です。例: `geo_core::Aabb2D`, `geo_core::Aabb3D`
    type Aabb;

    /// 軸平行境界ボックスを返します。
    fn aabb(&self) -> Option<Self::Aabb>;
}
