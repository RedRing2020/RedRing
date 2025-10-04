use geo_core::{Scalar, ToleranceContext, Vector2D};
use crate::geometry2d::{Point2D, Direction2D};

/// 2D円：中心点、半径、回転方向を持つ円
#[derive(Debug, Clone)]
pub struct Circle2D {
    center: Point2D,
    radius: Scalar,
    direction: Direction2D, // 回転方向（通常は正方向 = 反時計回り）
}

impl Circle2D {
    /// 中心点、半径、回転方向から円を作成
    pub fn new(center: Point2D, radius: Scalar, direction: Direction2D) -> Option<Self> {
        if radius.value() <= 0.0 {
            None
        } else {
            Some(Self { center, radius, direction })
        }
    }

    /// f64値から円を作成（反時計回り方向）
    pub fn from_f64(center: Point2D, radius: f64) -> Option<Self> {
        if radius <= 0.0 {
            None
        } else {
            Some(Self {
                center,
                radius: Scalar::from_f64(radius),
                direction: Direction2D::unit_x(), // デフォルトは反時計回り
            })
        }
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point2D {
        &self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> Scalar {
        self.radius
    }

    /// 半径をf64で取得
    pub fn radius_f64(&self) -> f64 {
        self.radius.value()
    }

    /// 回転方向を取得
    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    /// 回転方向を反転
    pub fn reverse_direction(&mut self) {
        // 方向ベクトルを-1倍して反転
        if let Some(reversed) = Direction2D::from_f64(-self.direction.x(), -self.direction.y()) {
            self.direction = reversed;
        }
    }

    /// θ における円周上の点を計算
    pub fn evaluate(&self, theta: f64) -> Point2D {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let radius_val = self.radius.value();

        Point2D::new(
            self.center.x() + radius_val * cos_theta,
            self.center.y() + radius_val * sin_theta,
        )
    }

    /// θ における接線方向ベクトル
    pub fn tangent(&self, theta: f64) -> Vector2D {
        Vector2D::from_f64(
            -theta.sin(),
            theta.cos(),
        )
    }

    /// θ における法線方向ベクトル（中心から外向き）
    pub fn normal(&self, theta: f64) -> Vector2D {
        Vector2D::from_f64(
            theta.cos(),
            theta.sin(),
        )
    }

    /// 点が円周上にあるかどうか（誤差を考慮）
    pub fn contains_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance_sq = dx * dx + dy * dy;
        let radius_sq = self.radius.value() * self.radius.value();

        (distance_sq - radius_sq).abs() < tolerance.linear
    }

    /// 円を平行移動
    pub fn translate(&self, translation: &Vector2D) -> Self {
        Self {
            center: Point2D::new(
                self.center.x() + translation.x().value(),
                self.center.y() + translation.y().value(),
            ),
            radius: self.radius,
            direction: self.direction.clone(),
        }
    }

    /// 円を拡大・縮小
    pub fn scale(&self, factor: Scalar) -> Option<Self> {
        if factor.value() <= 0.0 {
            None
        } else {
            Some(Self {
                center: self.center.clone(),
                radius: self.radius * factor,
                direction: self.direction.clone(),
            })
        }
    }

    /// 中心点周りで回転
    pub fn rotate(&self, angle_rad: f64) -> Self {
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 中心点はそのまま、方向ベクトルのみ回転
        let old_dir_x = self.direction.x();
        let old_dir_y = self.direction.y();

        let new_direction = Direction2D::from_f64(
            old_dir_x * cos_angle - old_dir_y * sin_angle,
            old_dir_x * sin_angle + old_dir_y * cos_angle,
        ).unwrap_or_else(|| self.direction.clone());

        Self {
            center: self.center.clone(),
            radius: self.radius,
            direction: new_direction,
        }
    }
}
