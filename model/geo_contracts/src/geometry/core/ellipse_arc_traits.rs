//! EllipseArc 形状の契約定義
//!
//! geo_primitives が実装すべき EllipseArc 系の公開 trait。
//! 角度は母楕円の local angle parameter で、角度範囲は開始角から反時計回りに終了角までとする。

use crate::{Angle, Direction3D, Point2D, Point3D, Scalar};

/// EllipseArc2D生成のためのConstructorトレイト
pub trait EllipseArc2DConstructor<T: Scalar> {
    /// 中心・長半軸・短半軸・回転角・開始角・終了角から楕円弧を作成する
    ///
    /// 母楕円を作成できない場合は `None`。
    fn new(
        center: Point2D<T>,
        semi_major: T,
        semi_minor: T,
        rotation: Angle<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self>
    where
        Self: Sized;
}

/// EllipseArc3D生成のためのConstructorトレイト
pub trait EllipseArc3DConstructor<T: Scalar> {
    /// 中心・長半軸・短半軸・平面の法線・長軸方向・開始角・終了角から楕円弧を作成する
    ///
    /// 母楕円を作成できない場合は `None`。
    #[allow(clippy::too_many_arguments)]
    fn new(
        center: Point3D<T>,
        semi_major: T,
        semi_minor: T,
        normal: Direction3D<T>,
        major_axis_direction: Direction3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self>
    where
        Self: Sized;
}

/// EllipseArc2D基本プロパティ取得トレイト
pub trait EllipseArc2DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> Point2D<T>;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角（X 軸から長軸までの角度）を取得
    fn rotation(&self) -> Angle<T>;

    /// 開始角度を取得
    fn start_angle(&self) -> Angle<T>;

    /// 終了角度を取得
    fn end_angle(&self) -> Angle<T>;

    /// 開始角から反時計回りに終了角までの掃引角
    fn angle_span(&self) -> Angle<T>;
}

/// EllipseArc3D基本プロパティ取得トレイト
pub trait EllipseArc3DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> Point3D<T>;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 楕円平面の法線を取得
    fn normal(&self) -> Direction3D<T>;

    /// 長軸方向を取得
    fn major_axis_direction(&self) -> Direction3D<T>;

    /// 短軸方向（法線 × 長軸方向）を取得
    fn minor_axis_direction(&self) -> Direction3D<T>;

    /// 開始角度を取得
    fn start_angle(&self) -> Angle<T>;

    /// 終了角度を取得
    fn end_angle(&self) -> Angle<T>;

    /// 開始角から反時計回りに終了角までの掃引角
    fn angle_span(&self) -> Angle<T>;
}

/// EllipseArc2D計量機能トレイト
pub trait EllipseArc2DDerived<T: Scalar> {
    /// 楕円弧の長さ
    fn length(&self) -> T;
}

pub trait EllipseArc2DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> Point2D<T>;

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> Point2D<T>;
}

pub trait EllipseArc2DEvaluation<T: Scalar> {
    /// EllipseArc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は EllipseArc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母楕円の local angle parameter（開始角 + 掃引角 × t）へ写像される。
    fn point_at_parameter(&self, t: T) -> Point2D<T>;

    /// 母楕円の native angle parameter を直接指定して ideal evaluation point を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Point2D<T>;
}

pub trait EllipseArc2DDistance<T: Scalar> {
    /// 点の角度が角度範囲内なら母楕円までの距離、範囲外なら始点・終点までの距離の小さい方
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

pub trait EllipseArc2DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が楕円弧上にあるかを判定する
    fn contains_point(&self, point: Point2D<T>) -> bool;
}

pub trait EllipseArc2DTrimRange<T: Scalar> {
    /// 角度が角度範囲に含まれるかを判定する（範囲の両端を含み、既定の角度トレランスで判定する）
    fn contains_angle(&self, angle: Angle<T>) -> bool;

    /// 点の角度が角度範囲に含まれるかを判定する
    ///
    /// 点が曲線上にあるかではなく、角度範囲のみを判定する。点の角度は、点に最も近い母楕円上の点のパラメータ角とする。
    fn contains_point_angle(&self, point: Point2D<T>) -> bool;
}

/// EllipseArc3D計量機能トレイト
pub trait EllipseArc3DDerived<T: Scalar> {
    /// 楕円弧の長さ
    fn length(&self) -> T;
}

pub trait EllipseArc3DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> Point3D<T>;

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> Point3D<T>;
}

pub trait EllipseArc3DEvaluation<T: Scalar> {
    /// EllipseArc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は EllipseArc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母楕円の local angle parameter（開始角 + 掃引角 × t）へ写像される。
    fn point_at_parameter(&self, t: T) -> Point3D<T>;

    /// 母楕円の native angle parameter を直接指定して ideal evaluation point を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T>;
}

pub trait EllipseArc3DDistance<T: Scalar> {
    /// 点の角度が角度範囲内なら母楕円までの距離（平面からの距離を含む）、範囲外なら始点・終点までの距離の小さい方
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait EllipseArc3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が楕円弧上にあるかを判定する
    fn contains_point(&self, point: Point3D<T>) -> bool;
}

pub trait EllipseArc3DTrimRange<T: Scalar> {
    /// 角度が角度範囲に含まれるかを判定する（範囲の両端を含み、既定の角度トレランスで判定する）
    fn contains_angle(&self, angle: Angle<T>) -> bool;

    /// 点の角度が角度範囲に含まれるかを判定する
    ///
    /// 点が曲線上にあるかではなく、角度範囲のみを判定する。点の角度は、点に最も近い母楕円上の点のパラメータ角とする（平面外の点は平面へ投影した点の最近点）。
    fn contains_point_angle(&self, point: Point3D<T>) -> bool;
}
