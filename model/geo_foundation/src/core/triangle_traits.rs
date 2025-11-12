//! Triangle 抽象トレイト定義
//!
//! Foundation統一システムに基づくTriangle抽象化

use analysis::Scalar;

use crate::core::point_traits::Point3D;

/// Triangle3D作成のための抽象トレイト
/// NURBSなどの高レベル実装がTriangleを作成するためのインターフェース
pub trait Triangle3DConstructor<T: Scalar>: Sized {
    /// Point3Dトレイトを実装する型のパラメータ
    type Point: Point3D<T>;

    /// 新しいTriangle3Dインスタンスを作成
    /// 三つの点が同一線上にない場合にSomeを返し、同一線上の場合はNoneを返す
    fn new(p1: Self::Point, p2: Self::Point, p3: Self::Point) -> Option<Self>;
}

/// Triangle3D の基本操作抽象トレイト
pub trait Triangle3D<T: Scalar> {
    /// Point3Dトレイトを実装する型のパラメータ
    type Point: Point3D<T>;

    /// 三角形の面積を計算
    fn area(&self) -> T;

    /// 第一頂点を取得
    fn vertex1(&self) -> &Self::Point;

    /// 第二頂点を取得
    fn vertex2(&self) -> &Self::Point;

    /// 第三頂点を取得
    fn vertex3(&self) -> &Self::Point;
}
