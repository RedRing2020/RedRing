//! 2D Ellipse implementation
//!
//! 2次元楕円の基本実装

use crate::geometry2d::{bbox::BBoxF64, Circle, Point2DF64, Vector2D};
use geo_foundation::abstract_types::Angle;
use geo_foundation::constants::precision::GEOMETRIC_TOLERANCE;

/// 楕円関連のエラー
#[derive(Debug, Clone, PartialEq)]
pub enum EllipseError {
    /// 軸の長さが無効（負または0）
    InvalidAxisLength,
    /// 軸の順序が無効（短軸が長軸より長い）
    InvalidAxisOrder,
}

impl std::fmt::Display for EllipseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EllipseError::InvalidAxisLength => write!(f, "Invalid axis length: must be positive"),
            EllipseError::InvalidAxisOrder => write!(
                f,
                "Invalid axis order: major radius must be >= minor radius"
            ),
        }
    }
}

impl std::error::Error for EllipseError {}

/// 2D平面上の楕円を表現する構造体
#[derive(Debug, Clone)]
pub struct Ellipse {
    center: crate::geometry2d::Point2DF64,
    major_radius: f64,
    minor_radius: f64,
    rotation: Angle<f64>, // 回転角度
}

impl Ellipse {
    /// 新しい楕円を作成
    ///
    /// # Arguments
    /// * `center` - 楕円の中心点
    /// * `major_radius` - 長軸の半径
    /// * `minor_radius` - 短軸の半径
    /// * `rotation` - 回転角度
    pub fn new(
        center: Point2DF64,
        major_radius: f64,
        minor_radius: f64,
        rotation: Angle<f64>,
    ) -> Result<Self, EllipseError> {
        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return Err(EllipseError::InvalidAxisLength);
        }
        if major_radius < minor_radius {
            return Err(EllipseError::InvalidAxisOrder);
        }

        Ok(Self {
            center,
            major_radius,
            minor_radius,
            rotation,
        })
    }

    /// 軸平行楕円を作成（回転なし）
    pub fn axis_aligned(
        center: crate::geometry2d::Point2DF64,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, EllipseError> {
        Self::new(center, major_radius, minor_radius, Angle::zero())
    }

    /// 円から楕円を作成
    pub fn from_circle(circle: &Circle<f64>) -> Self {
        let center = circle.center();
        let radius = circle.radius();
        Self::axis_aligned(center, radius, radius).unwrap()
    }

    /// 楕円の中心座標を取得
    pub fn center(&self) -> Point2DF64 {
        self.center
    }

    /// 楕円の長軸半径を取得
    pub fn major_radius(&self) -> f64 {
        self.major_radius
    }

    /// 楕円の短軸半径を取得
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius
    }

    /// 楕円の回転角度を取得
    pub fn rotation(&self) -> Angle<f64> {
        self.rotation
    }

    /// 楕円の面積を計算（analysis クレートの数値解析機能を使用）
    pub fn area(&self) -> f64 {
        analysis::geometry::ellipse_properties::area(
            self.major_radius,
            self.minor_radius
        )
    }

    /// 楕円の周長を計算（analysis クレートの高精度数値解析機能を使用）
    /// 
    /// # 数値解析について
    /// 内部的に `analysis::geometry::ellipse_circumference::ramanujan_approximation` を使用。
    /// ラマヌジャンの近似式で高精度かつ高速な計算を実現します。
    /// 
    /// より高精度が必要な場合は、`analysis` クレートの他の手法も直接利用可能：
    /// - `series_expansion`: 無限級数展開（より高精度）
    /// - `numerical_integration`: 数値積分（最高精度）
    pub fn circumference(&self) -> f64 {
        analysis::geometry::ellipse_circumference::ramanujan_approximation(
            self.major_radius,
            self.minor_radius
        )
    }

    /// 楕円の離心率を計算（analysis クレートの数値解析機能を使用）
    pub fn eccentricity(&self) -> f64 {
        analysis::geometry::ellipse_circumference::eccentricity(
            self.major_radius,
            self.minor_radius
        )
    }

    /// 楕円の焦点距離を計算（analysis クレートの数値解析機能を使用）
    pub fn focal_distance(&self) -> f64 {
        analysis::geometry::ellipse_properties::focal_distance(
            self.major_radius,
            self.minor_radius
        )
    }

    /// 楕円の焦点を取得
    pub fn foci(&self) -> (Point2DF64, Point2DF64) {
        let focal_dist = self.focal_distance();
        let cos_rot = self.rotation.to_radians().cos();
        let sin_rot = self.rotation.to_radians().sin();

        let f1_x = self.center.x() + focal_dist * cos_rot;
        let f1_y = self.center.y() + focal_dist * sin_rot;
        let f2_x = self.center.x() - focal_dist * cos_rot;
        let f2_y = self.center.y() - focal_dist * sin_rot;

        (Point2DF64::new(f1_x, f1_y), Point2DF64::new(f2_x, f2_y))
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        (self.major_radius - self.minor_radius).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 指定された角度での楕円周上の点を取得
    pub fn point_at_angle(&self, angle: f64) -> Point2DF64 {
        let cos_rot = self.rotation.to_radians().cos();
        let sin_rot = self.rotation.to_radians().sin();
        let cos_t = angle.cos();
        let sin_t = angle.sin();

        let x = self.major_radius * cos_t * cos_rot - self.minor_radius * sin_t * sin_rot;
        let y = self.major_radius * cos_t * sin_rot + self.minor_radius * sin_t * cos_rot;

        Point2DF64::new(self.center.x() + x, self.center.y() + y)
    }

    /// 点が楕円内部にあるかを判定
    pub fn contains_point(&self, point: &Point2DF64) -> bool {
        // 楕円の中心を原点とした座標系に変換
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();

        // 回転を考慮した座標変換
        let cos_rot = self.rotation.to_radians().cos();
        let sin_rot = self.rotation.to_radians().sin();
        let x_rot = dx * cos_rot + dy * sin_rot;
        let y_rot = -dx * sin_rot + dy * cos_rot;

        // 楕円の方程式で内部判定
        let normalized = (x_rot / self.major_radius).powi(2) + (y_rot / self.minor_radius).powi(2);
        normalized <= 1.0
    }

    /// 点が楕円境界上にあるかを判定
    pub fn on_boundary(&self, point: &Point2DF64) -> bool {
        // 楕円の中心を原点とした座標系に変換
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();

        // 回転を考慮した座標変換
        let cos_rot = self.rotation.to_radians().cos();
        let sin_rot = self.rotation.to_radians().sin();
        let x_rot = dx * cos_rot + dy * sin_rot;
        let y_rot = -dx * sin_rot + dy * cos_rot;

        // 楕円の方程式で境界判定
        let normalized = (x_rot / self.major_radius).powi(2) + (y_rot / self.minor_radius).powi(2);
        (normalized - 1.0).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 楕円のバウンディングボックスを計算
    pub fn bounding_box(&self) -> BBoxF64 {
        // 回転を考慮した楕円のバウンディングボックス計算
        let cos_rot = self.rotation.to_radians().cos();
        let sin_rot = self.rotation.to_radians().sin();

        let a = self.major_radius;
        let b = self.minor_radius;

        // 楕円の軸に対する最大・最小値を計算
        let x_extent = ((a * cos_rot).powi(2) + (b * sin_rot).powi(2)).sqrt();
        let y_extent = ((a * sin_rot).powi(2) + (b * cos_rot).powi(2)).sqrt();

        BBoxF64::from_two_points(
            Point2DF64::new(self.center.x() - x_extent, self.center.y() - y_extent),
            Point2DF64::new(self.center.x() + x_extent, self.center.y() + y_extent),
        )
    }

    /// 楕円をスケール
    pub fn scale(&self, factor: f64) -> Self {
        Self::new(
            self.center,
            self.major_radius * factor,
            self.minor_radius * factor,
            self.rotation,
        )
        .unwrap()
    }

    /// 楕円を平行移動
    pub fn translate(&self, vector: &Vector2D) -> Self {
        let new_center =
            Point2DF64::new(self.center.x() + vector.x(), self.center.y() + vector.y());
        Self::new(
            new_center,
            self.major_radius,
            self.minor_radius,
            self.rotation,
        )
        .unwrap()
    }

    /// 楕円を回転
    pub fn rotate(&self, angle: Angle<f64>) -> Self {
        Self::new(
            self.center,
            self.major_radius,
            self.minor_radius,
            Angle::from_radians(self.rotation.to_radians() + angle.to_radians()),
        )
        .unwrap()
    }

    /// 指定された点から楕円境界への最短距離を計算（近似）
    pub fn distance_to_point(&self, point: &Point2DF64) -> f64 {
        if self.contains_point(point) {
            0.0
        } else {
            // 簡易実装：楕円境界上の複数点との距離を計算し最小値を返す
            let mut min_dist = f64::INFINITY;
            for i in 0..36 {
                let angle = (i as f64 * 10.0).to_radians();
                let boundary_point = self.point_at_angle(angle);
                let dist = point.distance_to(&boundary_point);
                if dist < min_dist {
                    min_dist = dist;
                }
            }
            min_dist
        }
    }

    /// 楕円を円に変換（長軸の半径を使用）
    pub fn to_circle(&self) -> Circle<f64> {
        Circle::new(self.center, self.major_radius)
    }

    /// 楕円を最小外接円に変換
    pub fn bounding_circle(&self) -> Circle<f64> {
        Circle::new(self.center, self.major_radius)
    }

    /// 楕円を最大内接円に変換
    pub fn inscribed_circle(&self) -> Circle<f64> {
        Circle::new(self.center, self.minor_radius)
    }

    /// 高精度な楕円周長計算（無限級数展開）
    /// 
    /// analysis クレートの級数展開による高精度計算を使用。
    /// 計算コストが高いため、通常は `circumference()` を推奨。
    /// 
    /// # Arguments
    /// * `terms` - 級数の項数（精度と計算量のトレードオフ、通常10-20で十分）
    pub fn circumference_high_precision(&self, terms: usize) -> f64 {
        analysis::geometry::ellipse_circumference::series_expansion(
            self.major_radius,
            self.minor_radius,
            terms
        )
    }

    /// 最高精度な楕円周長計算（数値積分）
    /// 
    /// analysis クレートの数値積分による最高精度計算。研究用途や検証用。
    /// 計算コストが非常に高いため、一般用途では推奨しません。
    /// 
    /// # Arguments
    /// * `n_points` - 積分点数（精度と計算量のトレードオフ、100-1000程度）
    pub fn circumference_numerical(&self, n_points: usize) -> f64 {
        analysis::geometry::ellipse_circumference::numerical_integration(
            self.major_radius,
            self.minor_radius,
            n_points
        )
    }
}

// 手動でPartialEqを実装
impl PartialEq for Ellipse {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && (self.major_radius - other.major_radius).abs() < GEOMETRIC_TOLERANCE
            && (self.minor_radius - other.minor_radius).abs() < GEOMETRIC_TOLERANCE
            && (self.rotation.to_radians() - other.rotation.to_radians()).abs()
                < GEOMETRIC_TOLERANCE
    }
}

// 注意: テストコードは unit_tests/ellipse_tests.rs に分離されています
