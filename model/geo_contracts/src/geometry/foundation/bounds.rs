use analysis::abstract_types::Scalar;

use super::metadata::PrimitiveMetadata;

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
