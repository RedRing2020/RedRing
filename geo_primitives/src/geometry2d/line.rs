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
}

impl Line2D {
    /// 開始点と終了点から線分を作成
    pub fn new(start: Point2D, end: Point2D) -> Self {
        let dx = end.x().value() - start.x().value();
        let dy = end.y().value() - start.y().value();
        let direction_vector = Vector2D::new(dx, dy);
        let base = if let Some(direction) = Direction2D::from_vector(&direction_vector) {
            InfiniteLine2D::new(start.clone(), direction)
        } else {
            InfiniteLine2D::new(start.clone(), Direction2D::unit_x())
        };
        Self { base, start, end }
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
            self.end.x().value() - self.start.x().value(),
            self.end.y().value() - self.start.y().value(),
        )
    }

    /// 線分の方向を取得
    pub fn direction(&self) -> Option<Direction2D> {
        Direction2D::from_vector(&self.direction_vector())
    }

    /// 線分の長さを取得
    pub fn length(&self) -> Scalar { Scalar::new(self.length_f64()) }
    pub fn length_f64(&self) -> f64 {
        let dx = self.end.x().value() - self.start.x().value();
        let dy = self.end.y().value() - self.start.y().value();
        (dx * dx + dy * dy).sqrt()
    }

    /// InfiniteLineとしての表現を取得
    pub fn to_infinite(&self) -> &InfiniteLine2D {
        &self.base
    }

    /// 線分の中点を取得
    pub fn midpoint(&self) -> Point2D {
        let mx = (self.start.x().value() + self.end.x().value()) * 0.5;
        let my = (self.start.y().value() + self.end.y().value()) * 0.5;
        Point2D::new(Scalar::new(mx), Scalar::new(my))
    }

    /// f64パラメータでの点を評価（コア実装）
    pub fn evaluate_f64(&self, t: f64) -> Option<Point2D> {
        if !(0.0..=1.0).contains(&t) { return None; }
        let sx = self.start.x().value();
        let sy = self.start.y().value();
        let ex = self.end.x().value();
        let ey = self.end.y().value();
        let x = sx + (ex - sx) * t;
        let y = sy + (ey - sy) * t;
        Some(Point2D::new(Scalar::new(x), Scalar::new(y)))
    }

    /// 旧 API: Scalar パラメータでの点を評価（後方互換用）
    #[deprecated(note = "Use evaluate_f64(t: f64) instead")]
    pub fn evaluate(&self, t: Scalar) -> Option<Point2D> { self.evaluate_f64(t.value()) }

    /// 点が線分上にあるか判定
    pub fn contains_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
        // まず無限直線上にあるかチェック
        let to_point = Vector2D::new(
            point.x().value() - self.start.x().value(),
            point.y().value() - self.start.y().value(),
        );

        let direction_vec = self.direction_vector();

        // 方向ベクトルがゼロベクトルの場合（同じ点）
        let len_sq = direction_vec.x()*direction_vec.x() + direction_vec.y()*direction_vec.y();
        if len_sq < tolerance.linear * tolerance.linear {
            return point.tolerant_eq(&self.start, tolerance);
        }

        // 外積で直線からの距離をチェック
        let cross = to_point.x() * direction_vec.y() - to_point.y() * direction_vec.x();
        if cross.abs() > tolerance.linear {
            return false;
        }

        // 線分の範囲内かチェック
    let dot = to_point.x() * direction_vec.x() + to_point.y() * direction_vec.y();
    let projection_param = dot / len_sq;

        projection_param >= -tolerance.linear && projection_param <= 1.0 + tolerance.linear
    }

    /// f64での点が線分上にあるか判定
    pub fn contains_point_f64(&self, point: &Point2D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.contains_point(point, &tolerance)
    }

    /// 点から線分までの距離
    /// f64での点から線分までの距離（コア実装）
    pub fn distance_to_point_f64(&self, point: &Point2D) -> f64 {
        let sx = self.start.x().value();
        let sy = self.start.y().value();
        let ex = self.end.x().value();
        let ey = self.end.y().value();
        let px = point.x().value();
        let py = point.y().value();

        let dx = ex - sx;
        let dy = ey - sy;
        let seg_len_sq = dx * dx + dy * dy;
        if seg_len_sq < 1e-10 { // 退化
            let dxp = px - sx;
            let dyp = py - sy;
            return (dxp * dxp + dyp * dyp).sqrt();
        }
        let t = ((px - sx) * dx + (py - sy) * dy) / seg_len_sq;
        if t <= 0.0 {
            let dxp = px - sx; let dyp = py - sy; (dxp * dxp + dyp * dyp).sqrt()
        } else if t >= 1.0 {
            let dxp = px - ex; let dyp = py - ey; (dxp * dxp + dyp * dyp).sqrt()
        } else {
            let cx = sx + dx * t;
            let cy = sy + dy * t;
            let dxp = px - cx; let dyp = py - cy; (dxp * dxp + dyp * dyp).sqrt()
        }
    }

    /// 旧 API: Scalar 戻り距離（後方互換）
    #[deprecated(note = "Use distance_to_point_f64(&Point2D) -> f64 instead")]
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar { Scalar::new(self.distance_to_point_f64(point)) }

    /// 線分上での点のパラメータを取得
    /// f64 での線分パラメータ取得（存在し線分上であれば [0,1]）
    pub fn parameter_of_point_f64(&self, point: &Point2D, tolerance: &ToleranceContext) -> Option<f64> {
        if !self.contains_point(point, tolerance) { return None; }
        let sx = self.start.x().value();
        let sy = self.start.y().value();
        let ex = self.end.x().value();
        let ey = self.end.y().value();
        let dx = ex - sx; let dy = ey - sy;
        let len_sq = dx * dx + dy * dy;
        if len_sq < tolerance.linear * tolerance.linear { return Some(0.0); }
        let px = point.x().value();
        let py = point.y().value();
        let param = ((px - sx) * dx + (py - sy) * dy) / len_sq;
        Some(param.clamp(0.0, 1.0))
    }

    /// 旧 API: Scalar 版（後方互換）
    #[deprecated(note = "Use parameter_of_point_f64(&Point2D, &ToleranceContext) -> Option<f64> instead")]
    pub fn parameter_of_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> Option<Scalar> {
        self.parameter_of_point_f64(point, tolerance).map(Scalar::new)
    }

    /// 線分を移動
    /// f64での線分を移動（コア実装）
    pub fn translate_f64(&self, dx: f64, dy: f64) -> Line2D {
        let sx = self.start.x().value() + dx;
        let sy = self.start.y().value() + dy;
        let ex = self.end.x().value() + dx;
        let ey = self.end.y().value() + dy;
        Line2D::new(
            Point2D::new(Scalar::new(sx), Scalar::new(sy)),
            Point2D::new(Scalar::new(ex), Scalar::new(ey)),
        )
    }

    /// 旧 API: Scalar 版（後方互換）
    #[deprecated(note = "Use translate_f64(dx, dy) instead")]
    pub fn translate(&self, dx: Scalar, dy: Scalar) -> Line2D { self.translate_f64(dx.value(), dy.value()) }

    /// 線分を回転（原点中心）
    /// f64での線分を回転（原点中心）
    pub fn rotate_f64(&self, angle: f64) -> Line2D {
        let (sin_angle, cos_angle) = angle.sin_cos();
        let (sx, sy) = (self.start.x().value(), self.start.y().value());
        let (ex, ey) = (self.end.x().value(), self.end.y().value());
        let rsx = sx * cos_angle - sy * sin_angle;
        let rsy = sx * sin_angle + sy * cos_angle;
        let rex = ex * cos_angle - ey * sin_angle;
        let rey = ex * sin_angle + ey * cos_angle;
        Line2D::new(
            Point2D::new(Scalar::new(rsx), Scalar::new(rsy)),
            Point2D::new(Scalar::new(rex), Scalar::new(rey)),
        )
    }

    /// 旧 API: Scalar 版（後方互換）
    #[deprecated(note = "Use rotate_f64(angle) instead")]
    pub fn rotate(&self, angle: Scalar) -> Line2D { self.rotate_f64(angle.value()) }

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
        let point = line.evaluate_f64(0.0).unwrap();
        assert_eq!(point.x().value(), 0.0);
        assert_eq!(point.y().value(), 0.0);

        // t = 1.0で終了点
        let point = line.evaluate_f64(1.0).unwrap();
        assert_eq!(point.x().value(), 2.0);
        assert_eq!(point.y().value(), 4.0);

        // t = 0.5で中点
        let point = line.evaluate_f64(0.5).unwrap();
        assert_eq!(point.x().value(), 1.0);
        assert_eq!(point.y().value(), 2.0);

        // 範囲外
        assert!(line.evaluate_f64(-0.1).is_none());
        assert!(line.evaluate_f64(1.1).is_none());
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
    assert!(line.distance_to_point_f64(&point) < 1e-10);

        // 線分に垂直な点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(1.0));
    assert!((line.distance_to_point_f64(&point) - 1.0).abs() < 1e-10);

        // 開始点の延長上
        let point = Point2D::new(Scalar::new(-1.0), Scalar::new(0.0));
    assert!((line.distance_to_point_f64(&point) - 1.0).abs() < 1e-10);

        // 終了点の延長上
        let point = Point2D::new(Scalar::new(3.0), Scalar::new(0.0));
        assert!((line.distance_to_point_f64(&point) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_of_point() {
        let line = Line2D::from_f64(0.0, 0.0, 2.0, 4.0);
        let tolerance = ToleranceContext::standard();

        // 開始点
        let point = Point2D::new(Scalar::new(0.0), Scalar::new(0.0));
    let param = line.parameter_of_point_f64(&point, &tolerance).unwrap();
    assert!((param - 0.0).abs() < 1e-10);

        // 終了点
        let point = Point2D::new(Scalar::new(2.0), Scalar::new(4.0));
    let param = line.parameter_of_point_f64(&point, &tolerance).unwrap();
    assert!((param - 1.0).abs() < 1e-10);

        // 中点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(2.0));
    let param = line.parameter_of_point_f64(&point, &tolerance).unwrap();
    assert!((param - 0.5).abs() < 1e-10);

        // 線分上にない点
        let point = Point2D::new(Scalar::new(1.0), Scalar::new(3.0));
        assert!(line.parameter_of_point_f64(&point, &tolerance).is_none());
    }

    #[test]
    fn test_translate() {
        let line = Line2D::from_f64(0.0, 0.0, 1.0, 1.0);
    let translated = line.translate_f64(2.0, 3.0);

        assert_eq!(translated.start().x().value(), 2.0);
        assert_eq!(translated.start().y().value(), 3.0);
        assert_eq!(translated.end().x().value(), 3.0);
        assert_eq!(translated.end().y().value(), 4.0);
    }

    #[test]
    fn test_rotate() {
        let line = Line2D::from_f64(1.0, 0.0, 2.0, 0.0);
    let rotated = line.rotate_f64(std::f64::consts::PI / 2.0); // 90度回転

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

    assert_eq!(direction.x(), 3.0);
    assert_eq!(direction.y(), 4.0);
    }

    #[test]
    fn test_direction() {
        let line = Line2D::from_f64(0.0, 0.0, 3.0, 4.0);
        let direction = line.direction().unwrap();

        // 正規化された方向ベクトル
    assert!((direction.to_vector().x() - 0.6).abs() < 1e-10);
    assert!((direction.to_vector().y() - 0.8).abs() < 1e-10);

        // 退化した線分では方向が取得できない
        let degenerate_line = Line2D::from_f64(1.0, 1.0, 1.0, 1.0);
        assert!(degenerate_line.direction().is_none());
    }
}
