//! 2次元円弧（Arc2D）の実装
//!
//! 基本実装に集中 - 複雑な変換や他の幾何要素との組み合わせは回避

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Angle, Scalar};

/// 2次元円弧
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
    start_direction: Vector2D<T>,
    start_angle: Angle<T>,
    end_angle: Angle<T>,
}

impl<T: Scalar> Arc2D<T> {
    /// 新しい円弧を作成
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径（正の値）
    /// * `start_direction` - 開始方向ベクトル（正規化される）
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn new(
        center: Point2D<T>,
        radius: T,
        start_direction: Vector2D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        let normalized_dir = start_direction.try_normalize()?;

        Some(Self {
            center,
            radius,
            start_direction: normalized_dir,
            start_angle,
            end_angle,
        })
    }

    /// XY平面上の円弧を作成（開始角度はX軸正方向から）
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn xy_arc(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Self::new(center, radius, Vector2D::unit_x(), start_angle, end_angle)
    }

    /// 3点を通る円弧を作成
    ///
    /// # 引数
    /// * `start` - 開始点
    /// * `middle` - 中間点
    /// * `end` - 終了点
    pub fn from_three_points(
        start: Point2D<T>,
        middle: Point2D<T>,
        end: Point2D<T>,
    ) -> Option<Self> {
        // 3点から円の中心と半径を計算
        let v1 = start.vector_to(&middle);
        let v2 = start.vector_to(&end);

        // 3点が一直線上にある場合は円弧を作成できない
        let cross = v1.cross(&v2);
        if cross.abs() <= DefaultTolerances::distance::<T>() {
            return None;
        }

        // 外心の計算
        let d = (start.x() * (middle.y() - end.y())
            + middle.x() * (end.y() - start.y())
            + end.x() * (start.y() - middle.y()))
            * (T::ONE + T::ONE);

        if d.abs() <= DefaultTolerances::distance::<T>() {
            return None;
        }

        let ux = ((start.x() * start.x() + start.y() * start.y()) * (middle.y() - end.y())
            + (middle.x() * middle.x() + middle.y() * middle.y()) * (end.y() - start.y())
            + (end.x() * end.x() + end.y() * end.y()) * (start.y() - middle.y()))
            / d;

        let uy = ((start.x() * start.x() + start.y() * start.y()) * (end.x() - middle.x())
            + (middle.x() * middle.x() + middle.y() * middle.y()) * (start.x() - end.x())
            + (end.x() * end.x() + end.y() * end.y()) * (middle.x() - start.x()))
            / d;

        let center = Point2D::new(ux, uy);
        let radius = start.vector_to(&center).length();

        // 各点に対応する角度を計算
        let start_dir = start.vector_to(&center).try_normalize()?;
        let end_dir = end.vector_to(&center).try_normalize()?;

        let start_angle = start_dir.y().atan2(start_dir.x());
        let end_angle = end_dir.y().atan2(end_dir.x());

        Self::new(
            center,
            radius,
            start_dir,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 開始方向ベクトルを取得
    pub fn start_direction(&self) -> Vector2D<T> {
        self.start_direction
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    /// 角度範囲を取得
    pub fn angle_span(&self) -> Angle<T> {
        self.angle_span_radians()
    }

    /// 角度範囲をラジアン値として取得（内部使用）
    fn angle_span_radians(&self) -> Angle<T> {
        let mut span = self.end_angle - self.start_angle;
        if span.to_radians() < T::ZERO {
            span += Angle::from_radians(T::TAU);
        }
        span
    }

    /// 円弧長を計算
    pub fn arc_length(&self) -> T {
        self.radius * self.angle_span_radians().to_radians()
    }

    /// 完全円かどうかを判定
    pub fn is_full_circle(&self) -> bool {
        let span = self.angle_span();
        let two_pi = Angle::from_radians(T::TAU);
        span.is_equivalent_default(&two_pi)
    }

    /// 退化した円弧かどうかを判定（非常に小さい半径または角度範囲）
    pub fn is_degenerate(&self) -> bool {
        self.radius <= DefaultTolerances::distance::<T>()
            || self.angle_span_radians().to_radians() <= Angle::<T>::tolerance()
    }

    /// パラメータ t (0.0 ～ 1.0) における点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = self.start_angle.to_radians() + t * self.angle_span_radians().to_radians();
        self.point_at_angle(angle)
    }

    /// 指定角度における点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let x = self.radius * angle.cos();
        let y = self.radius * angle.sin();
        let point_on_circle = Vector2D::new(x, y);
        self.center + point_on_circle
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 中点を取得
    pub fn mid_point(&self) -> Point2D<T> {
        let mid_angle = self.start_angle.to_radians()
            + self.angle_span_radians().to_radians() / (T::ONE + T::ONE);
        self.point_at_angle(mid_angle)
    }

    /// 指定角度が円弧の範囲内にあるかを判定
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        let normalized_angle = self.normalize_angle(angle);
        let normalized_start = self.normalize_angle(self.start_angle);
        let normalized_end = self.normalize_angle(self.end_angle);

        if normalized_start <= normalized_end {
            normalized_angle >= normalized_start && normalized_angle <= normalized_end
        } else {
            // 0度をまたぐ場合
            normalized_angle >= normalized_start || normalized_angle <= normalized_end
        }
    }

    /// 角度を [0, 2π) 範囲に正規化
    pub fn normalize_angle(&self, angle: Angle<T>) -> Angle<T> {
        let two_pi = Angle::from_radians(T::TAU);
        let mut normalized = angle;

        // 負の角度を正に変換
        while normalized.to_radians() < T::ZERO {
            normalized += two_pi;
        }

        // 2π以上の角度を削減
        while normalized >= two_pi {
            normalized -= two_pi;
        }

        normalized
    }

    /// Circle2D に変換（完全円の場合のみ）
    pub fn to_circle(&self) -> Option<Circle2D<T>> {
        if self.is_full_circle() {
            Circle2D::new(self.center, self.radius)
        } else {
            None
        }
    }
}
