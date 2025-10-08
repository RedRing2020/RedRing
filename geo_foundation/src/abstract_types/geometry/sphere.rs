//! 球（Sphere）の抽象トレイト定義
//!
//! 3次元空間における球面および球体の共通インターフェースを定義する。
//! 具体的な実装はgeo_primitivesクレートで行う。

use crate::Scalar;
use std::any::Any;

/// 3次元球面・球体の共通トレイト
///
/// このトレイトは3次元空間における球面と球体の共通操作を定義する。
/// 球面は表面のみ、球体は内部を含む立体として扱う。
pub trait Sphere: Any + Send + Sync {
    /// スカラー型（f32またはf64）
    type Scalar: Scalar;

    /// 点型（球の中心座標）
    type Point;

    /// ベクトル型（変換操作用）
    type Vector;

    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 半径を取得
    fn radius(&self) -> Self::Scalar;

    /// 表面積を計算
    /// 公式: 4πr²
    fn surface_area(&self) -> Self::Scalar;

    /// 体積を計算
    /// 公式: (4/3)πr³
    fn volume(&self) -> Self::Scalar;

    /// 点が球面上にあるかチェック
    fn point_on_surface(&self, point: &Self::Point) -> bool;

    /// 点が球体内部にあるかチェック（境界含む）
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が球体内部にあるかチェック（境界含まない）
    fn contains_point_strict(&self, point: &Self::Point) -> bool;

    /// 球面上の点を球面座標（θ, φ）から取得
    /// θ: 方位角（0 ≤ θ < 2π）
    /// φ: 仰角（0 ≤ φ ≤ π）
    fn point_at_spherical(&self, theta: Self::Scalar, phi: Self::Scalar) -> Self::Point;

    /// 中心からの距離を計算
    fn distance_from_center(&self, point: &Self::Point) -> Self::Scalar;

    /// 球面までの最短距離を計算
    /// 負の値は球体内部を表す
    fn distance_to_surface(&self, point: &Self::Point) -> Self::Scalar;

    /// 球を平行移動
    fn translated(&self, offset: &Self::Vector) -> Self;

    /// 球を拡大縮小
    fn scaled(&self, factor: Self::Scalar) -> Self;

    /// 球の境界ボックスを取得
    fn bounding_box(&self) -> (Self::Point, Self::Point);

    /// 他の球との交差判定
    fn intersects_sphere(&self, other: &Self) -> bool;

    /// 球の種類識別
    fn sphere_kind(&self) -> SphereKind;
}

/// 球の種類を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SphereKind {
    /// 標準的な球
    Standard,
    /// 単位球（半径1、原点中心）
    Unit,
    /// 退化した球（半径0）
    Degenerate,
}

/// 球面座標系のユーティリティトレイト
pub trait SphericalCoordinates {
    /// スカラー型
    type Scalar: Scalar;

    /// 点型
    type Point;

    /// 直交座標から球面座標への変換
    /// 戻り値: (r, θ, φ) - 半径、方位角、仰角
    fn cartesian_to_spherical(
        &self,
        point: &Self::Point,
    ) -> (Self::Scalar, Self::Scalar, Self::Scalar);

    /// 球面座標から直交座標への変換
    fn spherical_to_cartesian(
        &self,
        r: Self::Scalar,
        theta: Self::Scalar,
        phi: Self::Scalar,
    ) -> Self::Point;
}

/// 球面上のパラメトリック曲線トレイト
pub trait SphericalCurve {
    /// スカラー型
    type Scalar: Scalar;

    /// 点型
    type Point;

    /// パラメータtでの点を取得
    fn point_at_parameter(&self, t: Self::Scalar) -> Self::Point;

    /// パラメータtでの接線ベクトルを取得
    fn tangent_at_parameter(&self, t: Self::Scalar) -> Self::Point;

    /// 曲線の長さを計算
    fn arc_length(&self) -> Self::Scalar;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_kind_enum() {
        let standard = SphereKind::Standard;
        let unit = SphereKind::Unit;
        let degenerate = SphereKind::Degenerate;

        assert_ne!(standard, unit);
        assert_ne!(unit, degenerate);
        assert_ne!(degenerate, standard);
    }

    #[test]
    fn test_sphere_trait_bounds() {
        // コンパイル時にトレイト境界をテスト
        fn check_sphere_bounds<T: Sphere>() {}
        fn check_spherical_coords<T: SphericalCoordinates>() {}
        fn check_spherical_curve<T: SphericalCurve>() {}

        // これらの関数が正常にコンパイルされることを確認
    }
}
