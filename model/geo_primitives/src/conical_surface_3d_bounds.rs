//! ConicalSurface3D の Bounds 実装
//!
//! `Bounded` trait と境界計算補助を提供する。

use crate::ConicalSurface3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for ConicalSurface3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 実用的な範囲で境界ボックスを計算
        // デフォルトで軸方向に ±100 単位の範囲を使用
        let default_range = T::from_f64(100.0);
        Some(self.bounding_box(-default_range, default_range))
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
        let r1 = self.radius_at_v_internal(min_v);
        let r2 = self.radius_at_v_internal(max_v);
        let height = max_v - min_v;

        // 母線の長さ
        let slant_height = height / self.semi_angle_internal().cos();

        // 表面積
        let pi = T::PI;
        let two = T::ONE + T::ONE;
        (u_range / (two * pi)) * pi * (r1 + r2) * slant_height
    }
}
