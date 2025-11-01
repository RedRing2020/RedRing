//! ConicalSurface3D の Foundation パターン実装
//!
//! ExtensionFoundation トレイトの実装により、
//! 他の幾何プリミティブとの統一インターフェースを提供

use crate::{BBox3D, ConicalSurface3D};
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for ConicalSurface3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::ConicalSurface
    }

    fn bounding_box(&self) -> Self::BBox {
        // 実用的な範囲で境界ボックスを計算
        // デフォルトで軸方向に ±100 単位の範囲を使用
        let default_range = T::from_f64(100.0);
        self.bounding_box(-default_range, default_range)
    }

    fn measure(&self) -> Option<T> {
        // 円錐サーフェスの表面積は無限大（無制限範囲）
        // 境界が指定された場合のみ計算可能
        None
    }
}

impl<T: Scalar> ConicalSurface3D<T> {
    /// 指定された範囲での表面積を計算
    ///
    /// # Arguments
    /// * `min_v` - 軸方向の最小範囲
    /// * `max_v` - 軸方向の最大範囲
    /// * `min_u` - 円周方向の最小角度（デフォルト: 0）
    /// * `max_u` - 円周方向の最大角度（デフォルト: 2π）
    ///
    /// # Returns
    /// 指定された範囲での表面積
    pub fn surface_area(&self, min_v: T, max_v: T, min_u: Option<T>, max_u: Option<T>) -> T {
        let u_min = min_u.unwrap_or(T::ZERO);
        let u_max = max_u.unwrap_or(T::PI * (T::ONE + T::ONE));
        let u_range = u_max - u_min;

        // 円錐サーフェスの表面積公式
        // A = (u_range / 2π) * π * (r1 + r2) * s
        // ここで s は母線の長さ
        let r1 = self.radius_at_v(min_v);
        let r2 = self.radius_at_v(max_v);
        let height = max_v - min_v;

        // 母線の長さ
        let slant_height = height / self.semi_angle().cos();

        // 表面積
        let pi = T::PI;
        let two = T::ONE + T::ONE;
        (u_range / (two * pi)) * pi * (r1 + r2) * slant_height
    }

    /// 最小境界ボックスを計算（Foundation パターン用）
    ///
    /// # Returns
    /// 実用的な範囲での最小境界ボックス
    pub fn minimal_bounding_box(&self) -> BBox3D<T> {
        // 円錐は無限に延びるため、実用的な範囲を設定
        // 基準半径の10倍程度の範囲を使用
        let practical_range = self.radius() * T::from_f64(10.0);
        self.bounding_box(-practical_range, practical_range)
    }
}
