//! EllipseArc Core Traits - EllipseArc形状のCore機能統合
//!
//! Foundation Pattern Phase 1 + Phase 2 実装
//! Transform機能は共通のAnalysisTransformトレイトを使用

use analysis::abstract_types::Scalar;

/// EllipseArc2D生成のためのConstructorトレイト（Phase 1 + Phase 2）
pub trait EllipseArc2DConstructor<T: Scalar> {
    /// 中心、長軸、短軸、回転角、角度範囲から楕円弧を作成
    fn new(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

/// EllipseArc3D生成のためのConstructorトレイト（Phase 1 + Phase 2）
pub trait EllipseArc3DConstructor<T: Scalar> {
    /// 3D空間での楕円弧を作成（法線、長軸方向、角度範囲）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major: T,
        semi_minor: T,
        major_direction: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

/// EllipseArc2D基本プロパティ取得トレイト（Phase 1 + Phase 2）
pub trait EllipseArc2DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;
}

/// EllipseArc3D基本プロパティ取得トレイト（Phase 1 + Phase 2）
pub trait EllipseArc3DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    /// 法線ベクトルを取得
    fn normal(&self) -> (T, T, T);
}

/// EllipseArc2D計量・関係演算機能トレイト（Phase 1 + Phase 2）
pub trait EllipseArc2DDerived<T: Scalar> {
    /// 楕円弧の長さ
    fn length(&self) -> T;

    /// 楕円弧の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T), (T, T));
}

pub trait EllipseArc2DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> (T, T);

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> (T, T);
}

pub trait EllipseArc2DEvaluation<T: Scalar> {
    /// EllipseArc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は EllipseArc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母楕円の local angle parameter へ線形写像される。
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 母楕円の native angle parameter を直接指定して ideal evaluation point を取得
    fn point_at_angle(&self, angle: T) -> (T, T);
}

pub trait EllipseArc2DContainment<T: Scalar> {
    /// 点が楕円弧上にあるか判定（許容誤差付き）
    fn contains_point(&self, point: (T, T), tolerance: T) -> bool;
}

pub trait EllipseArc2DTrimRange<T: Scalar> {
    /// Primitive 局所角度系の角度がトリム区間内か判定
    fn contains_angle(&self, angle: T) -> bool;
}

/// EllipseArc3D計量・関係演算機能トレイト（Phase 1 + Phase 2）
pub trait EllipseArc3DDerived<T: Scalar> {
    /// 楕円弧の長さ
    fn length(&self) -> T;

    /// 楕円弧の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
}

pub trait EllipseArc3DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> (T, T, T);

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> (T, T, T);
}

pub trait EllipseArc3DEvaluation<T: Scalar> {
    /// EllipseArc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は EllipseArc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母楕円の local angle parameter へ線形写像される。
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 母楕円の native angle parameter を直接指定して ideal evaluation point を取得
    fn point_at_angle(&self, angle: T) -> (T, T, T);
}

pub trait EllipseArc3DContainment<T: Scalar> {
    /// 点が楕円弧上にあるか判定（許容誤差付き）
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool;
}

pub trait EllipseArc3DTrimRange<T: Scalar> {
    /// Primitive 局所角度系の角度がトリム区間内か判定
    fn contains_angle(&self, angle: T) -> bool;
}
