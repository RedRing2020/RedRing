use geo_core::{Point2D, Vector2D, Scalar, ToleranceContext, TolerantEq};
use crate::geometry2d::Direction2D;
use crate::geometry2d::infinite_line::InfiniteLine2D;

/// 2D線分
///
/// 開始点と終了点を持つ有限線分
/// InfiniteLineをベースとして持つ
#[derive(Debug, Clone)]
pub struct Line2D {
    base: InfiniteLine2D,
    start: Point2D,
    end: Point2D,
    length: Scalar,
}

impl Line2D {
    /// 開始点と終了点から線分を作成
    pub fn new(start: Point2D, end: Point2D) -> Self {
        let direction_vector = Vector2D::new(
            end.x().clone() - start.x().clone(),
            end.y().clone() - start.y().clone(),
        );
        let length = Scalar::new((direction_vector.x().value() * direction_vector.x().value() +
                                 direction_vector.y().value() * direction_vector.y().value()).sqrt());

        // InfiniteLineを作成（方向が取得できない場合はゼロベクトル方向を使用）
        let base = if let Some(direction) = Direction2D::from_vector(&direction_vector) {
            InfiniteLine2D::new(start.clone(), direction)
        } else {
            // 退化した線分の場合、X軸方向を使用
            InfiniteLine2D::new(start.clone(), Direction2D::unit_x())
        };

        Self { base, start, end, length }
    }

    /// f64座標から線分を作成
    pub fn from_f64(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self {
        let start = Point2D::new(Scalar::new(start_x), Scalar::new(start_y));
        let end = Point2D::new(Scalar::new(end_x), Scalar::new(end_y));
        Self::new(start, end)
    }

    /// 開始点を取得
    pub fn start(&self) -> &Point2D {
        &self.start
    }

    /// 終了点を取得
    pub fn end(&self) -> &Point2D {
        &self.end
    }

    /// 線分の方向ベクトルを取得
    pub fn direction_vector(&self) -> Vector2D {
        Vector2D::new(
            self.end.x().clone() - self.start.x().clone(),
            self.end.y().clone() - self.start.y().clone(),
        )
    }

    /// 線分の方向を取得
    pub fn direction(&self) -> Option<Direction2D> {
        Direction2D::from_vector(&self.direction_vector())
    }

    /// 線分の長さを取得
    pub fn length(&self) -> Scalar {
        self.length.clone()
    }

    /// f64での線分の長さを取得
    pub fn length_f64(&self) -> f64 {
        self.length.value()
    }

    /// InfiniteLineとしての表現を取得
    pub fn to_infinite(&self) -> &InfiniteLine2D {
        &self.base
    }

    /// 線分の中点を取得
    pub fn midpoint(&self) -> Point2D {
        Point2D::new(
            (self.start.x().clone() + self.end.x().clone()) / Scalar::new(2.0),
            (self.start.y().clone() + self.end.y().clone()) / Scalar::new(2.0),
        )
    }

    /// パラメータt (0 <= t <= 1) での点を評価
    pub fn evaluate(&self, t: Scalar) -> Option<Point2D> {
        if t.value() < 0.0 || t.value() > 1.0 {
            None
        } else {
            let direction = self.direction_vector();
            Some(Point2D::new(
                self.start.x().clone() + direction.x() * t.clone(),
                self.start.y().clone() + direction.y() * t,
            ))
        }
    }

    /// f64パラメータでの点を評価
    pub fn evaluate_f64(&self, t: f64) -> Option<Point2D> {
        self.evaluate(Scalar::new(t))
    }

    /// 点が線分上にあるか判定
    pub fn contains_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
        // まず無限直線上にあるかチェック
        let to_point = Vector2D::new(
            point.x().clone() - self.start.x().clone(),
            point.y().clone() - self.start.y().clone(),
        );

        let direction_vec = self.direction_vector();

        // 方向ベクトルがゼロベクトルの場合（同じ点）
        let dir_magnitude = (direction_vec.x().value() * direction_vec.x().value() +
                            direction_vec.y().value() * direction_vec.y().value()).sqrt();
        if dir_magnitude < tolerance.linear {
            return point.tolerant_eq(&self.start, tolerance);
        }

        // 外積で直線からの距離をチェック
        let cross = to_point.x() * direction_vec.y() - to_point.y() * direction_vec.x();
        if !cross.tolerant_eq(&Scalar::new(0.0), tolerance) {
            return false;
        }

        // 線分の範囲内かチェック
        let dot = to_point.x() * direction_vec.x() + to_point.y() * direction_vec.y();
        let projection_param = dot.value() / (dir_magnitude * dir_magnitude);

        projection_param >= -tolerance.linear && projection_param <= 1.0 + tolerance.linear
    }

    /// f64での点が線分上にあるか判定
    pub fn contains_point_f64(&self, point: &Point2D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.contains_point(point, &tolerance)
    }

    /// 点から線分までの距離
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar {
        let to_point = Vector2D::new(
            point.x().clone() - self.start.x().clone(),
            point.y().clone() - self.start.y().clone(),
        );

        let direction_vec = self.direction_vector();
        let dir_magnitude_sq = direction_vec.x().value() * direction_vec.x().value() +
                              direction_vec.y().value() * direction_vec.y().value();

        // 退化した線分（点）の場合
        if dir_magnitude_sq < 1e-10 {
            let dx = point.x().clone() - self.start.x().clone();
            let dy = point.y().clone() - self.start.y().clone();
            return Scalar::new((dx.value() * dx.value() + dy.value() * dy.value()).sqrt());
        }

        // 投影パラメータを計算
        let dot = to_point.x() * direction_vec.x() + to_point.y() * direction_vec.y();
        let projection_param = dot.value() / dir_magnitude_sq;

        if projection_param <= 0.0 {
            // 開始点が最近点
            let dx = point.x().clone() - self.start.x().clone();
            let dy = point.y().clone() - self.start.y().clone();
            Scalar::new((dx.value() * dx.value() + dy.value() * dy.value()).sqrt())
        } else if projection_param >= 1.0 {
            // 終了点が最近点
            let dx = point.x().clone() - self.end.x().clone();
            let dy = point.y().clone() - self.end.y().clone();
            Scalar::new((dx.value() * dx.value() + dy.value() * dy.value()).sqrt())
        } else {
            // 線分上の投影点が最近点
            let closest_on_line = Point2D::new(
                self.start.x().clone() + direction_vec.x() * Scalar::new(projection_param),
                self.start.y().clone() + direction_vec.y() * Scalar::new(projection_param),
            );

            let dx = point.x().clone() - closest_on_line.x().clone();
            let dy = point.y().clone() - closest_on_line.y().clone();
            Scalar::new((dx.value() * dx.value() + dy.value() * dy.value()).sqrt())
        }
    }

    /// f64での点から線分までの距離
    pub fn distance_to_point_f64(&self, point: &Point2D) -> f64 {
        self.distance_to_point(point).value()
    }

    /// 線分上での点のパラメータを取得
    pub fn parameter_of_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> Option<Scalar> {
        if !self.contains_point(point, tolerance) {
            return None;
        }

        let to_point = Vector2D::new(
            point.x().clone() - self.start.x().clone(),
            point.y().clone() - self.start.y().clone(),
        );

        let direction_vec = self.direction_vector();
        let dir_magnitude_sq = direction_vec.x().value() * direction_vec.x().value() +
                              direction_vec.y().value() * direction_vec.y().value();

        if dir_magnitude_sq < tolerance.linear * tolerance.linear {
            // 退化した線分の場合
            return Some(Scalar::new(0.0));
        }

        let dot = to_point.x() * direction_vec.x() + to_point.y() * direction_vec.y();
        let param = dot.value() / dir_magnitude_sq;

        Some(Scalar::new(param.clamp(0.0, 1.0)))
    }

    /// 線分を移動
    pub fn translate(&self, dx: Scalar, dy: Scalar) -> Line2D {
        let translated_start = Point2D::new(
            self.start.x().clone() + dx.clone(),
            self.start.y().clone() + dy.clone(),
        );
        let translated_end = Point2D::new(
            self.end.x().clone() + dx,
            self.end.y().clone() + dy,
        );
        Self::new(translated_start, translated_end)
    }

    /// f64での線分を移動
    pub fn translate_f64(&self, dx: f64, dy: f64) -> Line2D {
        self.translate(Scalar::new(dx), Scalar::new(dy))
    }

    /// 線分を回転（原点中心）
    pub fn rotate(&self, angle: Scalar) -> Line2D {
        let cos_angle = Scalar::new(angle.value().cos());
        let sin_angle = Scalar::new(angle.value().sin());

        let rotated_start = Point2D::new(
            self.start.x().clone() * cos_angle.clone() - self.start.y().clone() * sin_angle.clone(),
            self.start.x().clone() * sin_angle.clone() + self.start.y().clone() * cos_angle.clone(),
        );

        let rotated_end = Point2D::new(
            self.end.x().clone() * cos_angle.clone() - self.end.y().clone() * sin_angle.clone(),
            self.end.x().clone() * sin_angle + self.end.y().clone() * cos_angle,
        );

        Self::new(rotated_start, rotated_end)
    }

    /// f64での線分を回転（原点中心）
    pub fn rotate_f64(&self, angle: f64) -> Line2D {
        self.rotate(Scalar::new(angle))
    }

    /// 2つの線分が等しいか判定
    pub fn equals(&self, other: &Line2D, tolerance: &ToleranceContext) -> bool {
        (self.start.tolerant_eq(&other.start, tolerance) && self.end.tolerant_eq(&other.end, tolerance)) ||
        (self.start.tolerant_eq(&other.end, tolerance) && self.end.tolerant_eq(&other.start, tolerance))
    }

    /// 線分の境界ボックスを取得
    pub fn bounding_box(&self) -> (Point2D, Point2D) {
        let min_x = if self.start.x().value() < self.end.x().value() {
            self.start.x().clone()
        } else {
            self.end.x().clone()
        };
        let min_y = if self.start.y().value() < self.end.y().value() {
            self.start.y().clone()
        } else {
            self.end.y().clone()
        };
        let max_x = if self.start.x().value() > self.end.x().value() {
            self.start.x().clone()
        } else {
            self.end.x().clone()
        };
        let max_y = if self.start.y().value() > self.end.y().value() {
            self.start.y().clone()
        } else {
            self.end.y().clone()
        };

        (Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_creation() {
        let start = Point2D::new(Scalar::new(0.0), Scalar::new(0.0));
        let end = Point2D::new(Scalar::new(1.0), Scalar::new(1.0));
        let line = Line2D::new(start.clone(), end.clone());

        assert_eq!(line.start().x().value(), 0.0);
        assert_eq!(line.start().y().value(), 0.0);
        assert_eq!(line.end().x().value(), 1.0);
        assert_eq!(line.end().y().value(), 1.0);
    }

    #[test]
    fn test_line_from_f64() {
        let line = Line2D::from_f64(0.0, 0.0, 3.0, 4.0);

        assert_eq!(line.start().x().value(), 0.0);
        assert_eq!(line.start().y().value(), 0.0);
        assert_eq!(line.end().x().value(), 3.0);
        assert_eq!(line.end().y().value(), 4.0);
    }

    #[test]
    fn test_length() {
        let line = Line2D::from_f64(0.0, 0.0, 3.0, 4.0);
        let length = line.length();

        assert!((length.value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_midpoint() {
        let line = Line2D::from_f64(0.0, 0.0, 2.0, 4.0);
        let midpoint = line.midpoint();

        assert_eq!(midpoint.x().value(), 1.0);
        assert_eq!(midpoint.y().value(), 2.0);
    }

    #[test]
    fn test_evaluate() {
        let line = Line2D::from_f64(0.0, 0.0, 2.0, 4.0);

        // t = 0.0で開始点
        let point = line.evaluate(Scalar::new(0.0)).unwrap();
        assert_eq!(point.x().value(), 0.0);
        assert_eq!(point.y().value(), 0.0);

        // t = 1.0で終了点
        let point = line.evaluate(Scalar::new(1.0)).unwrap();
        assert_eq!(point.x().value(), 2.0);
        assert_eq!(point.y().value(), 4.0);

        // t = 0.5で中点
        let point = line.evaluate(Scalar::new(0.5)).unwrap();
        assert_eq!(point.x().value(), 1.0);
        assert_eq!(point.y().value(), 2.0);

        // 範囲外
        assert!(line.evaluate(Scalar::new(-0.1)).is_none());
        assert!(line.evaluate(Scalar::new(1.1)).is_none());
    }

    #[test]
    fn test_contains_point() {
        let line = Line2D::from_f64(0.0, 0.0, 2.0, 4.0);
        let tolerance = ToleranceContext::standard();

        // 線分上の点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(2.0));
        assert!(line.contains_point(&point, &tolerance));

        // 開始点
        let point = Point2D::new(Scalar::new(0.0), Scalar::new(0.0));
        assert!(line.contains_point(&point, &tolerance));

        // 終了点
        let point = Point2D::new(Scalar::new(2.0), Scalar::new(4.0));
        assert!(line.contains_point(&point, &tolerance));

        // 線分上にない点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(3.0));
        assert!(!line.contains_point(&point, &tolerance));

        // 直線の延長上にある点
        let point = Point2D::new(Scalar::new(-1.0), Scalar::new(-2.0));
        assert!(!line.contains_point(&point, &tolerance));
    }

    #[test]
    fn test_distance_to_point() {
        let line = Line2D::from_f64(0.0, 0.0, 2.0, 0.0); // 水平線

        // 線分上の点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(0.0));
        assert!(line.distance_to_point(&point).value() < 1e-10);

        // 線分に垂直な点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(1.0));
        assert!((line.distance_to_point(&point).value() - 1.0).abs() < 1e-10);

        // 開始点の延長上
        let point = Point2D::new(Scalar::new(-1.0), Scalar::new(0.0));
        assert!((line.distance_to_point(&point).value() - 1.0).abs() < 1e-10);

        // 終了点の延長上
        let point = Point2D::new(Scalar::new(3.0), Scalar::new(0.0));
        assert!((line.distance_to_point(&point).value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_of_point() {
        let line = Line2D::from_f64(0.0, 0.0, 2.0, 4.0);
        let tolerance = ToleranceContext::standard();

        // 開始点
        let point = Point2D::new(Scalar::new(0.0), Scalar::new(0.0));
        let param = line.parameter_of_point(&point, &tolerance).unwrap();
        assert!((param.value() - 0.0).abs() < 1e-10);

        // 終了点
        let point = Point2D::new(Scalar::new(2.0), Scalar::new(4.0));
        let param = line.parameter_of_point(&point, &tolerance).unwrap();
        assert!((param.value() - 1.0).abs() < 1e-10);

        // 中点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(2.0));
        let param = line.parameter_of_point(&point, &tolerance).unwrap();
        assert!((param.value() - 0.5).abs() < 1e-10);

        // 線分上にない点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(3.0));
        assert!(line.parameter_of_point(&point, &tolerance).is_none());
    }

    #[test]
    fn test_translate() {
        let line = Line2D::from_f64(0.0, 0.0, 1.0, 1.0);
        let translated = line.translate(Scalar::new(2.0), Scalar::new(3.0));

        assert_eq!(translated.start().x().value(), 2.0);
        assert_eq!(translated.start().y().value(), 3.0);
        assert_eq!(translated.end().x().value(), 3.0);
        assert_eq!(translated.end().y().value(), 4.0);
    }

    #[test]
    fn test_rotate() {
        let line = Line2D::from_f64(1.0, 0.0, 2.0, 0.0);
        let rotated = line.rotate(Scalar::new(std::f64::consts::PI / 2.0)); // 90度回転

        assert!((rotated.start().x().value() - 0.0).abs() < 1e-10);
        assert!((rotated.start().y().value() - 1.0).abs() < 1e-10);
        assert!((rotated.end().x().value() - 0.0).abs() < 1e-10);
        assert!((rotated.end().y().value() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_equals() {
        let line1 = Line2D::from_f64(0.0, 0.0, 1.0, 1.0);
        let line2 = Line2D::from_f64(0.0, 0.0, 1.0, 1.0);
        let line3 = Line2D::from_f64(1.0, 1.0, 0.0, 0.0); // 反対方向
        let line4 = Line2D::from_f64(0.0, 0.0, 2.0, 2.0);

        let tolerance = ToleranceContext::standard();

        assert!(line1.equals(&line2, &tolerance));
        assert!(line1.equals(&line3, &tolerance)); // 方向は関係なし
        assert!(!line1.equals(&line4, &tolerance));
    }

    #[test]
    fn test_bounding_box() {
        let line = Line2D::from_f64(1.0, 3.0, 2.0, 1.0);
        let (min_point, max_point) = line.bounding_box();

        assert_eq!(min_point.x().value(), 1.0);
        assert_eq!(min_point.y().value(), 1.0);
        assert_eq!(max_point.x().value(), 2.0);
        assert_eq!(max_point.y().value(), 3.0);
    }

    #[test]
    fn test_degenerate_line() {
        // 退化した線分（同じ点）
        let line = Line2D::from_f64(1.0, 2.0, 1.0, 2.0);
        let tolerance = ToleranceContext::standard();

        assert_eq!(line.length().value(), 0.0);

        let point = Point2D::new(Scalar::new(1.0), Scalar::new(2.0));
        assert!(line.contains_point(&point, &tolerance));

        let other_point = Point2D::new(Scalar::new(2.0), Scalar::new(3.0));
        assert!(!line.contains_point(&other_point, &tolerance));
    }

    #[test]
    fn test_direction_vector() {
        let line = Line2D::from_f64(1.0, 2.0, 4.0, 6.0);
        let direction = line.direction_vector();

        assert_eq!(direction.x().value(), 3.0);
        assert_eq!(direction.y().value(), 4.0);
    }

    #[test]
    fn test_direction() {
        let line = Line2D::from_f64(0.0, 0.0, 3.0, 4.0);
        let direction = line.direction().unwrap();

        // 正規化された方向ベクトル
        assert!((direction.to_vector().x().value() - 0.6).abs() < 1e-10);
        assert!((direction.to_vector().y().value() - 0.8).abs() < 1e-10);

        // 退化した線分では方向が取得できない
        let degenerate_line = Line2D::from_f64(1.0, 1.0, 1.0, 1.0);
        assert!(degenerate_line.direction().is_none());
    }
}
