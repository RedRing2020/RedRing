//! Ray（レイ/半無限直線）トレイト定義
//!
//! 2D/3D空間における半無限直線（レイ）の抽象的なインターフェースを提供
//! 起点から特定の方向に無限に延びる直線を表現し、レイキャスティングや
//! 衝突検出などのCAD/CAM操作に使用される。
//!
//! # 使用例
//!
//! ```ignore
//! use geo_foundation::abstract_types::geometry::Ray3D;
//!
//! // マウスクリックからレイを生成
//! // let ray = Ray3D::from_screen_point(camera, screen_x, screen_y);
//! // let intersection = ray.intersect_mesh(&mesh);
//! ```

use crate::abstract_types::Scalar;

/// 2Dレイの基本操作を定義するトレイト
pub trait Ray2D<T: Scalar> {
    /// 点の型（通常は Point2D）
    type Point;
    /// ベクトルの型（通常は Vector2D）
    type Vector;
    /// 方向の型（通常は Direction2D）
    type Direction;
    /// エラー型
    type Error;

    /// レイの起点を取得
    fn origin(&self) -> Self::Point;

    /// レイの方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction;

    /// 指定されたパラメータ t での点を取得
    /// t >= 0 の範囲でのみ有効（半無限直線のため）
    /// point = origin + t * direction (t >= 0)
    fn point_at_parameter(&self, t: T) -> Option<Self::Point>;

    /// 指定された点がレイ上にあるかを判定（許容誤差内）
    /// 点がレイの起点より後方にある場合はfalseを返す
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 指定された点からレイへの最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 指定された点からレイ上の最近点を取得
    /// 最近点がレイの起点より後方にある場合は起点を返す
    fn closest_point(&self, point: &Self::Point) -> Self::Point;

    /// 指定された点のレイ上でのパラメータ値を取得
    /// 負の値の場合は起点より後方にあることを示す
    fn parameter_at_point(&self, point: &Self::Point) -> T;

    /// 他のレイとの交点を計算
    fn intersect_ray(&self, other: &Self) -> Option<Self::Point>;

    /// 無限直線との交点を計算
    /// 具体的な無限直線型との交点計算は実装側で行う
    fn intersect_infinite_line(&self, line_origin: &Self::Point, line_direction: &Self::Direction) -> Option<Self::Point>;

    /// 線分との交点を計算
    fn intersect_line_segment(&self, start: &Self::Point, end: &Self::Point) -> Option<Self::Point>;

    /// 円との交点を計算（最も近い交点）
    fn intersect_circle(&self, center: &Self::Point, radius: T) -> Option<Self::Point>;

    /// 他のレイと平行かどうかを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他のレイと同一かどうかを判定
    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool;

    /// レイを指定した距離で切り詰める
    fn truncate(&self, max_distance: T) -> Result<(Self::Point, Self::Point), Self::Error>;

    /// レイを指定した点まで延長（点がレイ上にある場合）
    fn extend_to_point(&self, point: &Self::Point) -> Option<T>;
}

/// 3Dレイの基本操作を定義するトレイト
pub trait Ray3D<T: Scalar> {
    /// 点の型（通常は Point3D）
    type Point;
    /// ベクトルの型（通常は Vector3D）
    type Vector;
    /// 方向の型（通常は Direction3D）
    type Direction;
    /// エラー型
    type Error;

    /// レイの起点を取得
    fn origin(&self) -> Self::Point;

    /// レイの方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction;

    /// 指定されたパラメータ t での点を取得
    /// t >= 0 の範囲でのみ有効（半無限直線のため）
    /// point = origin + t * direction (t >= 0)
    fn point_at_parameter(&self, t: T) -> Option<Self::Point>;

    /// 指定された点がレイ上にあるかを判定（許容誤差内）
    /// 点がレイの起点より後方にある場合はfalseを返す
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 指定された点からレイへの最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 指定された点からレイ上の最近点を取得
    /// 最近点がレイの起点より後方にある場合は起点を返す
    fn closest_point(&self, point: &Self::Point) -> Self::Point;

    /// 指定された点のレイ上でのパラメータ値を取得
    /// 負の値の場合は起点より後方にあることを示す
    fn parameter_at_point(&self, point: &Self::Point) -> T;

    /// 他のレイとの交点を計算（3Dでは交差しない場合が多い）
    fn intersect_ray(&self, other: &Self) -> Option<Self::Point>;

    /// 無限直線との交点を計算
    /// 具体的な無限直線型との交点計算は実装側で行う
    fn intersect_infinite_line(&self, line_origin: &Self::Point, line_direction: &Self::Direction) -> Option<Self::Point>;

    /// 指定した平面との交点を計算
    fn intersect_plane(&self, plane_point: &Self::Point, plane_normal: &Self::Vector) -> Option<Self::Point>;

    /// 球との交点を計算（最も近い交点）
    fn intersect_sphere(&self, center: &Self::Point, radius: T) -> Option<Self::Point>;

    /// 三角形との交点を計算（Möller-Trumbore法）
    fn intersect_triangle(&self, v0: &Self::Point, v1: &Self::Point, v2: &Self::Point) -> Option<Self::Point>;

    /// 境界ボックスとの交点を計算
    fn intersect_bounding_box(&self, min_point: &Self::Point, max_point: &Self::Point) -> Option<(Self::Point, Self::Point)>;

    /// 他のレイと平行かどうかを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他のレイと同一かどうかを判定
    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool;

    /// レイを指定した平面に投影
    fn project_to_plane(&self, plane_normal: &Self::Vector, plane_point: &Self::Point) -> Option<Self>
    where
        Self: Sized;

    /// レイを指定した距離で切り詰める
    fn truncate(&self, max_distance: T) -> Result<(Self::Point, Self::Point), Self::Error>;

    /// レイを指定した点まで延長（点がレイ上にある場合）
    fn extend_to_point(&self, point: &Self::Point) -> Option<T>;
}

/// レイの構築操作に関するトレイト
pub trait RayBuilder<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 方向の型
    type Direction;
    /// レイの型
    type Ray;
    /// エラー型
    type Error;

    /// 点と方向ベクトルからレイを構築
    fn from_point_and_direction(
        origin: Self::Point,
        direction: Self::Direction,
    ) -> Result<Self::Ray, Self::Error>;

    /// 2点を通るレイを構築（第1点が起点、第2点への方向）
    fn from_two_points(
        origin: Self::Point,
        target: Self::Point,
    ) -> Result<Self::Ray, Self::Error>;

    /// スクリーン座標からレイを構築（3D用）
    fn from_screen_point(
        camera_position: Self::Point,
        camera_direction: Self::Direction,
        screen_x: T,
        screen_y: T,
        viewport_width: T,
        viewport_height: T,
        field_of_view: T,
    ) -> Result<Self::Ray, Self::Error>;
}

/// レイの変換操作に関するトレイト
pub trait RayTransform<T: Scalar> {
    /// 変換行列の型
    type Matrix;
    /// 変換されたレイの型
    type TransformedRay;

    /// レイを平行移動
    fn translate(&self, translation: Self::Matrix) -> Self::TransformedRay;

    /// レイを回転（2D: 中心点周り、3D: 軸周り）
    fn rotate(&self, rotation: Self::Matrix, center: Self::Matrix) -> Self::TransformedRay;

    /// レイを一般的な変換行列で変換
    fn transform(&self, matrix: &Self::Matrix) -> Self::TransformedRay;

    /// レイをスケール変換
    fn scale(&self, factor: T, center: Self::Matrix) -> Self::TransformedRay;

    /// レイを反転（方向ベクトルを逆向きに）
    fn reverse(&self) -> Self::TransformedRay;
}

/// レイの解析操作に関するトレイト
pub trait RayAnalysis<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;

    /// レイを指定した距離間隔でサンプリング
    fn sample_points(&self, max_distance: T, step_size: T) -> Vec<Self::Point>;

    /// レイ上の指定範囲での点群を生成
    fn generate_points(&self, start_distance: T, end_distance: T, num_points: usize) -> Vec<Self::Point>;

    /// レイと指定した境界内での交差範囲を取得
    fn clip_to_bounds(&self, min_point: Self::Point, max_point: Self::Point) -> Option<(T, T)>;

    /// レイの有効範囲を指定（最大距離の設定）
    fn with_max_distance(&self, max_distance: T) -> Self
    where
        Self: Sized + Clone;

    /// 複数の幾何学要素との最初の交点を取得
    /// 具体的な実装では特定の幾何学要素型を使用
    fn first_intersection<G>(&self, geometries: &[&G]) -> Option<(Self::Point, T)>
    where
        G: RayIntersectable<T, Point = Self::Point>;
}

/// レイと交差可能な幾何学要素のトレイト
pub trait RayIntersectable<T: Scalar> {
    /// 点の型
    type Point;

    /// レイとの交点を計算
    /// 具体的なレイ型を引数として受け取る
    fn intersect_ray<R>(&self, ray: &R) -> Option<Self::Point>
    where
        R: RayAnalysis<T, Point = Self::Point>;

    /// レイとの距離を計算
    /// 具体的なレイ型を引数として受け取る
    fn distance_to_ray<R>(&self, ray: &R) -> T
    where
        R: RayAnalysis<T, Point = Self::Point>;
}

// 必要に応じて他のトレイトをインポート
// use super::infinite_line::{InfiniteLine2D, InfiniteLine3D};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_trait_compilation() {
        // トレイト定義のコンパイルテスト
        // 実際の実装は具象型で行われる
    }

    #[test]
    fn test_ray_trait_bounds() {
        // トレイト境界のテスト
        #[allow(dead_code)]
        fn check_2d_ray<T: Scalar, R: Ray2D<T>>(_ray: &R) {
            // このテストはコンパイル時にトレイト境界を確認
        }
        
        #[allow(dead_code)]
        fn check_3d_ray<T: Scalar, R: Ray3D<T>>(_ray: &R) {
            // このテストはコンパイル時にトレイト境界を確認
        }
    }
}