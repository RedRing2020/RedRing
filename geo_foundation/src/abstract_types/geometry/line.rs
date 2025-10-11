//! Line (線分) トレイト定義
//!
//! 2D/3D空間における有限の線分の最小限のインターフェースを提供
//! LineはInfiniteLineを基盤として、開始点と終了点を追加した構造
//!
//! # 設計方針: 最小責務原則
//!
//! ## 責務の明確化
//!
//! ```text
//! Line Trait = 基本属性のみ
//! ├── 端点アクセス (start_point, end_point)
//! ├── パラメータ範囲 (start_parameter, end_parameter)
//! ├── 基本幾何計算 (midpoint, direction, vector)
//! ├── 線分特有の判定 (contains_point, length)
//! └── バウンディングボックス (bounding_box)
//!
//! 除外される責務:
//! ├── 交差判定 (intersection_*) → geo_algorithms
//! ├── 変換操作 (transform_*) → geo_algorithms
//! ├── 分割・合成 (subdivide, union) → geo_algorithms
//! └── 距離計算 (distance_*) → geo_algorithms
//! ```
//!
//! ## Line2D vs Line3D の設計
//!
//! ```rust,ignore
//! // 2D線分 - 平面幾何用
//! trait Line2D<T: Scalar> {
//!     type Point2D;      // Point2D<T>
//!     type Vector2D;     // Vector2D<T>
//!     type Direction2D;  // Direction2D<T>
//!     type InfiniteLine2D; // InfiniteLine2D<T>
//!     type BoundingBox2D;  // BoundingBox2D<T>
//! }
//!
//! // 3D線分 - 空間幾何用
//! trait Line3D<T: Scalar> {
//!     type Point3D;      // Point3D<T>
//!     type Vector3D;     // Vector3D<T>
//!     type Direction3D;  // Direction3D<T>
//!     type InfiniteLine3D; // InfiniteLine3D<T>
//!     type BoundingBox3D;  // BoundingBox3D<T>
//! }
//! ```
//!
//! ## バウンディングボックスの統合
//!
//! ```text
//! Line trait に bounding_box() を含める理由:
//! ├── 空間インデックス作成の基盤（R-tree等）
//! ├── レンダリング最適化（視錐台カリング）
//! ├── 衝突判定の第一段階チェック
//! └── 基本的な幾何属性として扱う
//! ```

use analysis::Scalar;

/// 2D空間における有限線分のトレイト
///
/// InfiniteLine2Dを基盤として、開始・終了パラメータで線分を定義
pub trait Line2D<T: Scalar> {
    /// 2D点の型
    type Point2D;

    /// 2Dベクトルの型
    type Vector2D;

    /// 2D方向ベクトルの型（正規化済み）
    type Direction2D;

    /// 2D無限直線の型
    type InfiniteLine2D;

    /// 2Dバウンディングボックスの型
    type BoundingBox2D;

    /// 基盤となる無限直線を取得
    fn infinite_line(&self) -> &Self::InfiniteLine2D;

    /// 線分の開始パラメータを取得
    fn start_parameter(&self) -> T;

    /// 線分の終了パラメータを取得
    fn end_parameter(&self) -> T;

    /// 線分の開始点を取得
    fn start_point(&self) -> Self::Point2D;

    /// 線分の終了点を取得
    fn end_point(&self) -> Self::Point2D;

    /// 線分の中点を取得
    fn midpoint(&self) -> Self::Point2D;

    /// 線分の方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction2D;

    /// 線分のベクトル（終了点 - 開始点）を取得
    fn vector(&self) -> Self::Vector2D;

    /// 指定された点が線分上にあるかを判定（許容誤差内）
    fn contains_point(&self, point: &Self::Point2D, tolerance: T) -> bool;

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 線分のバウンディングボックスを取得
    ///
    /// 空間インデックス作成やレンダリング最適化で使用
    fn bounding_box(&self) -> Self::BoundingBox2D;

    /// パラメータ値から点を取得（線分範囲内にクランプ）
    fn point_at_parameter(&self, t: T) -> Self::Point2D;

    /// 正規化されたパラメータ（0.0から1.0）から点を取得
    fn point_at_normalized_parameter(&self, t: T) -> Self::Point2D;

    /// 線分を指定した数に等分割した点を取得
    fn subdivide_points(&self, num_divisions: usize) -> Vec<Self::Point2D>;

    /// 線分の開始点と終了点が一致しているか（点に退化しているか）
    fn is_degenerate(&self, tolerance: T) -> bool;
}

/// 3D空間における有限線分のトレイト
///
/// InfiniteLine3Dを基盤として、開始・終了パラメータで線分を定義
pub trait Line3D<T: Scalar> {
    /// 3D点の型
    type Point3D;

    /// 3Dベクトルの型
    type Vector3D;

    /// 3D方向ベクトルの型（正規化済み）
    type Direction3D;

    /// 3D無限直線の型
    type InfiniteLine3D;

    /// 3Dバウンディングボックスの型
    type BoundingBox3D;

    /// 基盤となる無限直線を取得
    fn infinite_line(&self) -> &Self::InfiniteLine3D;

    /// 線分の開始パラメータを取得
    fn start_parameter(&self) -> T;

    /// 線分の終了パラメータを取得
    fn end_parameter(&self) -> T;

    /// 線分の開始点を取得
    fn start_point(&self) -> Self::Point3D;

    /// 線分の終了点を取得
    fn end_point(&self) -> Self::Point3D;

    /// 線分の中点を取得
    fn midpoint(&self) -> Self::Point3D;

    /// 線分の方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction3D;

    /// 線分のベクトル（終了点 - 開始点）を取得
    fn vector(&self) -> Self::Vector3D;

    /// 指定された点が線分上にあるかを判定（許容誤差内）
    fn contains_point(&self, point: &Self::Point3D, tolerance: T) -> bool;

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 線分のバウンディングボックスを取得
    ///
    /// 空間インデックス作成やレンダリング最適化で使用
    fn bounding_box(&self) -> Self::BoundingBox3D;

    /// パラメータ値から点を取得（線分範囲内にクランプ）
    fn point_at_parameter(&self, t: T) -> Self::Point3D;

    /// 正規化されたパラメータ（0.0から1.0）から点を取得
    fn point_at_normalized_parameter(&self, t: T) -> Self::Point3D;

    /// 線分を指定した数に等分割した点を取得
    fn subdivide_points(&self, num_divisions: usize) -> Vec<Self::Point3D>;

    /// 線分の開始点と終了点が一致しているか（点に退化しているか）
    fn is_degenerate(&self, tolerance: T) -> bool;
}

/// Line2Dの追加機能（2D特有の機能）
pub trait Line2DExt<T: Scalar>: Line2D<T> {
    /// 線分の傾き（slope）を取得
    ///
    /// 垂直線の場合はNoneを返す
    fn slope(&self) -> Option<T>;

    /// Y切片を取得
    ///
    /// 垂直線の場合はNoneを返す
    fn y_intercept(&self) -> Option<T>;

    /// 線分がX軸に平行かを判定
    fn is_horizontal(&self, tolerance: T) -> bool;

    /// 線分がY軸に平行かを判定
    fn is_vertical(&self, tolerance: T) -> bool;
}

/// Line3Dの追加機能（3D特有の機能）
pub trait Line3DExt<T: Scalar>: Line3D<T> {
    /// 線分が特定の平面に平行かを判定
    fn is_parallel_to_plane(&self, plane_normal: &Self::Vector3D, tolerance: T) -> bool;

    /// 線分が特定の平面に垂直かを判定
    fn is_perpendicular_to_plane(&self, plane_normal: &Self::Vector3D, tolerance: T) -> bool;

    /// 線分の方向余弦を取得
    fn direction_cosines(&self) -> (T, T, T);
}

#[cfg(test)]
mod tests {
    use super::*;

    // トレイトの境界がコンパイル可能であることを確認するテスト
    #[allow(dead_code)]
    fn test_trait_bounds<T: Scalar, L>()
    where
        L: Line2D<T>,
        L::Point2D: Clone,
        L::Vector2D: Clone,
        L::Direction2D: Clone,
    {
        // トレイトの境界の検証
    }

    #[allow(dead_code)]
    fn test_3d_trait_bounds<T: Scalar, L>()
    where
        L: Line3D<T>,
        L::Point3D: Clone,
        L::Vector3D: Clone,
        L::Direction3D: Clone,
    {
        // 3Dトレイトの境界の検証
    }
}
