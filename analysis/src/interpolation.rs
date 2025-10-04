/// 補間・近似機能
///
/// 線形補間、スプライン補間、ベジエ曲線補間を提供する

use geo_core::{Point2D, Vector2D, ToleranceContext};

/// 線形補間器
pub struct LinearInterpolator {
    tolerance: ToleranceContext,
}

impl LinearInterpolator {
    pub fn new(tolerance: ToleranceContext) -> Self {
        Self { tolerance }
    }

    /// 2点間の線形補間
    pub fn interpolate(&self, p0: &Point2D, p1: &Point2D, t: f64) -> Point2D {
    let x = p0.x() * (1.0 - t) + p1.x() * t;
    let y = p0.y() * (1.0 - t) + p1.y() * t;
    Point2D::from_f64(x, y)
    }

    /// 点列の区分線形補間
    pub fn interpolate_polyline(&self, points: &[Point2D], t: f64) -> Option<Point2D> {
        if points.len() < 2 {
            return None;
        }

        if t <= 0.0 {
            return Some(points[0]);
        }
        if t >= 1.0 {
            return Some(*points.last().unwrap());
        }

        // セグメント長さの累積
        let mut lengths = vec![0.0];
        let mut total_length = 0.0;

        for i in 1..points.len() {
            let dist = points[i-1].distance_to(&points[i]).value();
            total_length += dist;
            lengths.push(total_length);
        }

        let target_length = t * total_length;

        // 対象セグメントを見つける
        for i in 1..lengths.len() {
            if target_length <= lengths[i] {
                let segment_start = lengths[i-1];
                let segment_length = lengths[i] - segment_start;

                if segment_length < self.tolerance.linear {
                    return Some(points[i-1]);
                }

                let local_t = (target_length - segment_start) / segment_length;
                return Some(self.interpolate(&points[i-1], &points[i], local_t));
            }
        }

        Some(*points.last().unwrap())
    }
}

/// 3次ベジエ曲線
pub struct CubicBezier {
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    p3: Point2D,
}

impl CubicBezier {
    pub fn new(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D) -> Self {
        Self { p0, p1, p2, p3 }
    }

    /// ベジエ曲線上の点を評価
    pub fn evaluate(&self, t: f64) -> Point2D {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

    let x = uuu * self.p0.x() +
        3.0 * uu * t * self.p1.x() +
        3.0 * u * tt * self.p2.x() +
        ttt * self.p3.x();

    let y = uuu * self.p0.y() +
        3.0 * uu * t * self.p1.y() +
        3.0 * u * tt * self.p2.y() +
        ttt * self.p3.y();

        Point2D::from_f64(x, y)
    }

    /// 1次導関数（接線ベクトル）
    pub fn derivative(&self, t: f64) -> Vector2D {
        let u = 1.0 - t;
        let uu = u * u;
        let tt = t * t;

    let dx = -3.0 * uu * self.p0.x() +
         3.0 * (uu - 2.0 * u * t) * self.p1.x() +
         3.0 * (2.0 * u * t - tt) * self.p2.x() +
         3.0 * tt * self.p3.x();

    let dy = -3.0 * uu * self.p0.y() +
         3.0 * (uu - 2.0 * u * t) * self.p1.y() +
         3.0 * (2.0 * u * t - tt) * self.p2.y() +
         3.0 * tt * self.p3.y();

        Vector2D::from_f64(dx, dy)
    }

    /// 曲線を指定した分割数でサンプリング
    pub fn sample(&self, divisions: usize) -> Vec<Point2D> {
        let mut points = Vec::with_capacity(divisions + 1);

        for i in 0..=divisions {
            let t = i as f64 / divisions as f64;
            points.push(self.evaluate(t));
        }

        points
    }
}

/// カットマル・ロム・スプライン
pub struct CatmullRomSpline {
    points: Vec<Point2D>,
    _tension: f64, // 張力パラメータ（現在未使用。将来の減衰/カーディナルトランジション実装用に保持）
}

impl CatmullRomSpline {
    pub fn new(points: Vec<Point2D>, tension: f64) -> Self {
        Self { points, _tension: tension }
    }

    /// スプライン曲線上の点を評価
    pub fn evaluate(&self, t: f64) -> Option<Point2D> {
        if self.points.len() < 4 {
            return None;
        }

        let segment_count = self.points.len() - 3;
        let segment_t = t * segment_count as f64;
        let segment_index = segment_t.floor() as usize;
        let local_t = segment_t - segment_index as f64;

        if segment_index >= segment_count {
            return Some(self.points[self.points.len() - 2]);
        }

        let p0 = &self.points[segment_index];
        let p1 = &self.points[segment_index + 1];
        let p2 = &self.points[segment_index + 2];
        let p3 = &self.points[segment_index + 3];

        Some(self.catmull_rom_interpolate(p0, p1, p2, p3, local_t))
    }

    fn catmull_rom_interpolate(&self, p0: &Point2D, p1: &Point2D, p2: &Point2D, p3: &Point2D, t: f64) -> Point2D {
        let t2 = t * t;
        let t3 = t2 * t;

    let x = 0.5 * ((2.0 * p1.x()) +
               (-p0.x() + p2.x()) * t +
               (2.0 * p0.x() - 5.0 * p1.x() + 4.0 * p2.x() - p3.x()) * t2 +
               (-p0.x() + 3.0 * p1.x() - 3.0 * p2.x() + p3.x()) * t3);

    let y = 0.5 * ((2.0 * p1.y()) +
               (-p0.y() + p2.y()) * t +
               (2.0 * p0.y() - 5.0 * p1.y() + 4.0 * p2.y() - p3.y()) * t2 +
               (-p0.y() + 3.0 * p1.y() - 3.0 * p2.y() + p3.y()) * t3);

        Point2D::from_f64(x, y)
    }

    /// スプライン曲線を均等にサンプリング
    pub fn sample(&self, divisions: usize) -> Vec<Point2D> {
        let mut result = Vec::with_capacity(divisions + 1);

        for i in 0..=divisions {
            let t = i as f64 / divisions as f64;
            if let Some(point) = self.evaluate(t) {
                result.push(point);
            }
        }

        result
    }
}

