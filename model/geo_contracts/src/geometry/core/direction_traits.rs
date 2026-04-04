//! Direction Core Traits - Direction形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use analysis::abstract_types::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

/// Direction2D生成のためのConstructorトレイト
pub trait Direction2DConstructor<T: Scalar> {
    /// ベクトルから方向を作成（自動正規化）
    fn from_vector(vector: Vector2<T>) -> Option<Self>
    where
        Self: Sized;

    /// X、Y成分から方向を作成
    fn new(x: T, y: T) -> Option<Self>
    where
        Self: Sized;

    /// X軸正方向の単位ベクトル
    fn positive_x() -> Self
    where
        Self: Sized;

    /// Y軸正方向の単位ベクトル
    fn positive_y() -> Self
    where
        Self: Sized;

    /// X軸負方向の単位ベクトル
    fn negative_x() -> Self
    where
        Self: Sized;

    /// Y軸負方向の単位ベクトル
    fn negative_y() -> Self
    where
        Self: Sized;

    /// タプルから作成
    fn from_tuple(components: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// Analysis Vector2から作成
    fn from_analysis_vector(vector: &Vector2<T>) -> Option<Self>
    where
        Self: Sized;
}

/// Direction3D生成のためのConstructorトレイト
pub trait Direction3DConstructor<T: Scalar> {
    /// ベクトルから方向を作成（自動正規化）
    fn from_vector(vector: Vector3<T>) -> Option<Self>
    where
        Self: Sized;

    /// X、Y、Z成分から方向を作成
    fn new(x: T, y: T, z: T) -> Option<Self>
    where
        Self: Sized;

    /// X軸正方向の単位ベクトル
    fn positive_x() -> Self
    where
        Self: Sized;

    /// Y軸正方向の単位ベクトル
    fn positive_y() -> Self
    where
        Self: Sized;

    /// Z軸正方向の単位ベクトル
    fn positive_z() -> Self
    where
        Self: Sized;

    /// X軸負方向の単位ベクトル
    fn negative_x() -> Self
    where
        Self: Sized;

    /// Y軸負方向の単位ベクトル
    fn negative_y() -> Self
    where
        Self: Sized;

    /// Z軸負方向の単位ベクトル
    fn negative_z() -> Self
    where
        Self: Sized;

    /// タプルから作成
    fn from_tuple(components: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// Analysis Vector3から作成
    fn from_analysis_vector(vector: &Vector3<T>) -> Option<Self>
    where
        Self: Sized;
}

/// Direction2D基本プロパティ取得トレイト
pub trait Direction2DProperties<T: Scalar> {
    /// X成分取得（-1.0 ≤ x ≤ 1.0）
    fn x(&self) -> T;

    /// Y成分取得（-1.0 ≤ y ≤ 1.0）
    fn y(&self) -> T;

    /// 成分を配列として取得
    fn components(&self) -> [T; 2];

    /// 成分をタプルとして取得
    fn to_tuple(&self) -> (T, T);

    /// Analysis Vector2へ変換
    fn to_analysis_vector(&self) -> Vector2<T>;

    /// 内部ベクトルを取得
    fn as_vector(&self) -> Vector2<T>;

    /// 長さ（常に1.0）
    fn length(&self) -> T;

    /// 正規化済みかどうか（常にtrue）
    fn is_normalized(&self) -> bool;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

/// Direction3D基本プロパティ取得トレイト
pub trait Direction3DProperties<T: Scalar> {
    /// X成分取得（-1.0 ≤ x ≤ 1.0）
    fn x(&self) -> T;

    /// Y成分取得（-1.0 ≤ y ≤ 1.0）
    fn y(&self) -> T;

    /// Z成分取得（-1.0 ≤ z ≤ 1.0）
    fn z(&self) -> T;

    /// 成分を配列として取得
    fn components(&self) -> [T; 3];

    /// 成分をタプルとして取得
    fn to_tuple(&self) -> (T, T, T);

    /// Analysis Vector3へ変換
    fn to_analysis_vector(&self) -> Vector3<T>;

    /// 内部ベクトルを取得
    fn as_vector(&self) -> Vector3<T>;

    /// 長さ（常に1.0）
    fn length(&self) -> T;

    /// 正規化済みかどうか（常にtrue）
    fn is_normalized(&self) -> bool;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}

/// Direction2D関係演算機能トレイト
pub trait Direction2DRelation<T: Scalar> {
    /// 他の方向との内積
    fn dot(&self, other: &Self) -> T;

    /// 他の方向との角度
    fn angle_to(&self, other: &Self) -> T;

    /// 他の方向と平行かどうか
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の方向と垂直かどうか
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の方向と同じ方向かどうか
    fn is_same_direction(&self, other: &Self) -> bool;

    /// 他の方向と反対方向かどうか
    fn is_opposite_direction(&self, other: &Self) -> bool;
}

/// Direction2D変換機能トレイト
pub trait Direction2DTransform<T: Scalar> {
    /// 反転（逆方向）
    fn reverse(&self) -> Self;

    /// 90度回転（反時計回り）
    fn rotate_90(&self) -> Self;

    /// 指定角度だけ回転
    fn rotate(&self, angle: T) -> Self;
}

/// Direction3D関係演算機能トレイト
pub trait Direction3DRelation<T: Scalar> {
    /// 他の方向との内積
    fn dot(&self, other: &Self) -> T;

    /// 他の方向との角度
    fn angle_to(&self, other: &Self) -> T;

    /// 他の方向との外積（結果も正規化される）
    fn cross(&self, other: &Self) -> Self;

    /// 他の方向と平行かどうか
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の方向と垂直かどうか
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の方向と同じ方向かどうか
    fn is_same_direction(&self, other: &Self) -> bool;

    /// 他の方向と反対方向かどうか
    fn is_opposite_direction(&self, other: &Self) -> bool;
}

/// Direction3D変換機能トレイト
pub trait Direction3DTransform<T: Scalar> {
    /// 反転（逆方向）
    fn reverse(&self) -> Self;

    /// 指定軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self;
}

/// Direction2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait Direction2DCore<T: Scalar>: Direction2DConstructor<T> + Direction2DProperties<T> {}

/// Direction3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait Direction3DCore<T: Scalar>: Direction3DConstructor<T> + Direction3DProperties<T> {}

impl<T: Scalar, Direction> Direction2DCore<T> for Direction where
    Direction: Direction2DConstructor<T> + Direction2DProperties<T>
{
}

impl<T: Scalar, Direction> Direction3DCore<T> for Direction where
    Direction: Direction3DConstructor<T> + Direction3DProperties<T>
{
}
