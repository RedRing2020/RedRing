//! 改善されたArc (円弧) トレイト定義
//!
//! Circleを基盤とした円弧の統一インターフェース

use super::angle::{Angle, Scalar};
use super::improved_circle::{Circle, Circle2D, Circle3D, CircleError};

/// 統一されたArcトレイト
///
/// # 特徴
/// - Circle構造体を内包する設計
/// - 角度範囲による円弧の定義
/// - 共通の円弧操作インターフェース
pub trait Arc<T: Scalar> {
    /// 基底の円の型
    type Circle: Circle<T>;

    /// 基底の円を取得
    fn circle(&self) -> &Self::Circle;

    /// 円弧の開始角度を取得
    fn start_angle(&self) -> Angle<T>;

    /// 円弧の終了角度を取得
    fn end_angle(&self) -> Angle<T>;

    /// 円弧の中心座標を取得（基底円の中心）
    fn center(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().center()
    }

    /// 円弧の半径を取得（基底円の半径）
    fn radius(&self) -> T {
        self.circle().radius()
    }

    /// 円弧の角度範囲を取得
    ///
    /// # Returns
    /// 開始角度から終了角度への角度差（常に正の値、最短経路）
    fn angle_span(&self) -> Angle<T> {
        let diff = self.start_angle().difference(&self.end_angle());
        if diff.radians() < T::zero() {
            diff + Angle::from_radians(T::tau())
        } else {
            diff
        }
    }

    /// 円弧の弧長を計算
    fn arc_length(&self) -> T {
        self.radius() * self.angle_span().radians()
    }

    /// 円弧の開始点を取得
    fn start_point(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().point_at_angle(self.start_angle())
    }

    /// 円弧の終了点を取得
    fn end_point(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().point_at_angle(self.end_angle())
    }

    /// 円弧の中点を取得
    fn midpoint(&self) -> <Self::Circle as Circle<T>>::Point {
        let mid_angle = self
            .start_angle()
            .lerp(&self.end_angle(), T::one() / T::two());
        self.circle().point_at_angle(mid_angle)
    }

    /// 円弧上の指定されたパラメータ（0.0〜1.0）での点を取得
    ///
    /// # Arguments
    /// * `t` - パラメータ（0.0=開始点, 1.0=終了点）
    fn point_at_parameter(&self, t: T) -> <Self::Circle as Circle<T>>::Point {
        let interpolated_angle = self.start_angle().lerp(&self.end_angle(), t);
        self.circle().point_at_angle(interpolated_angle)
    }

    /// 円弧上の指定された角度での点を取得
    ///
    /// # Arguments
    /// * `angle` - 角度（円弧の角度範囲内）
    ///
    /// # Returns
    /// 角度が範囲内の場合は点、範囲外の場合はNone
    fn point_at_angle(&self, angle: Angle<T>) -> Option<<Self::Circle as Circle<T>>::Point> {
        if self.contains_angle(angle) {
            Some(self.circle().point_at_angle(angle))
        } else {
            None
        }
    }

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        angle.is_within_range(&self.start_angle(), &self.end_angle())
    }

    /// 指定された点が円弧上にあるかを判定
    ///
    /// # Arguments
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    fn contains_point(&self, point: &<Self::Circle as Circle<T>>::Point, tolerance: T) -> bool {
        // 1. 点が基底円の円周上にあるかチェック
        if !self.circle().on_circumference(point, tolerance) {
            return false;
        }

        // 2. 点が円弧の角度範囲内にあるかチェック
        self.contains_point_on_circumference(point)
    }

    /// 円周上の点が円弧の角度範囲内にあるかを判定
    ///
    /// # Note
    /// この関数は点が既に円周上にあることを前提とする
    fn contains_point_on_circumference(&self, point: &<Self::Circle as Circle<T>>::Point) -> bool;

    /// 円弧を指定倍率で拡大縮小
    fn scale(&self, factor: T) -> Self
    where
        Self: Sized,
        Self::Circle: Clone;

    /// 円弧を指定ベクトルで平行移動
    fn translate(&self, vector: &<Self::Circle as Circle<T>>::Vector) -> Self
    where
        Self: Sized,
        Self::Circle: Clone;

    /// 円弧を反転（開始角度と終了角度を交換）
    fn reverse(&self) -> Self
    where
        Self: Sized;

    /// 円弧が退化しているか（角度範囲が0またはそれに近い）を判定
    fn is_degenerate(&self, tolerance: T) -> bool {
        self.angle_span().radians() <= tolerance
    }

    /// 円弧が完全な円に近いかを判定
    fn is_nearly_full_circle(&self, tolerance: T) -> bool {
        (self.angle_span().radians() - T::tau()).abs() <= tolerance
    }

    /// 円弧の境界ボックスを取得
    fn bounding_box(&self) -> <Self::Circle as Circle<T>>::BoundingBox;
}

/// 2D円弧専用の追加機能
pub trait Arc2D<T: Scalar>: Arc<T>
where
    Self::Circle: Circle2D<T>,
{
    /// 円弧の面積（扇形の面積）を計算
    fn sector_area(&self) -> T {
        let circle_area = self.circle().area();
        let angle_ratio = self.angle_span().radians() / T::tau();
        circle_area * angle_ratio
    }

    /// 円弧と弦で囲まれた弓形の面積を計算
    fn segment_area(&self) -> T {
        let sector_area = self.sector_area();
        let triangle_area = self.chord_triangle_area();
        sector_area - triangle_area
    }

    /// 円弧の弦の長さを計算
    fn chord_length(&self) -> T {
        let start = self.start_point();
        let end = self.end_point();
        self.point_distance(&start, &end)
    }

    /// 円弧の矢高（サジタ）を計算
    ///
    /// # Returns
    /// 弦の中点から円弧までの距離
    fn sagitta(&self) -> T {
        let half_chord = self.chord_length() / T::two();
        let radius = self.radius();
        radius - (radius * radius - half_chord * half_chord).sqrt()
    }

    /// 2つの点間の距離を計算（2D特有）
    fn point_distance(
        &self,
        p1: &<Self::Circle as Circle<T>>::Point,
        p2: &<Self::Circle as Circle<T>>::Point,
    ) -> T;

    /// 弦で作られる三角形の面積を計算
    fn chord_triangle_area(&self) -> T {
        let half_chord = self.chord_length() / T::two();
        let radius = self.radius();
        let height = (radius * radius - half_chord * half_chord).sqrt();
        half_chord * height
    }

    /// 指定された点からの接線を計算
    fn tangent_from_point(&self, point: &<Self::Circle as Circle<T>>::Point) -> Vec<Angle<T>>;
}

/// 3D円弧専用の追加機能
pub trait Arc3D<T: Scalar>: Arc<T>
where
    Self::Circle: Circle3D<T>,
{
    /// 円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> <Self::Circle as Circle<T>>::Vector {
        self.circle().normal()
    }

    /// 円弧の局所座標系を取得
    fn local_coordinate_system(
        &self,
    ) -> (
        <Self::Circle as Circle<T>>::Vector, // U軸
        <Self::Circle as Circle<T>>::Vector, // V軸
        <Self::Circle as Circle<T>>::Vector, // 法線
    ) {
        (
            self.circle().u_axis(),
            self.circle().v_axis(),
            self.normal(),
        )
    }

    /// 円弧を2D平面に投影
    fn to_2d(&self) -> impl Arc2D<T>;

    /// 指定された点が円弧の平面上にあるかを判定
    fn point_on_plane(&self, point: &<Self::Circle as Circle<T>>::Point, tolerance: T) -> bool {
        self.circle().point_on_plane(point, tolerance)
    }
}

/// 円弧の構築エラー
#[derive(Debug, Clone, PartialEq)]
pub enum ArcError {
    /// 基底円のエラー
    CircleError(CircleError),
    /// 無効な角度範囲
    InvalidAngleRange,
    /// 角度の順序が不正
    InvalidAngleOrder,
}

impl<T: Scalar> From<CircleError> for ArcError {
    fn from(err: CircleError) -> Self {
        ArcError::CircleError(err)
    }
}

impl std::fmt::Display for ArcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArcError::CircleError(err) => write!(f, "円エラー: {}", err),
            ArcError::InvalidAngleRange => write!(f, "無効な角度範囲です"),
            ArcError::InvalidAngleOrder => write!(f, "角度の順序が不正です"),
        }
    }
}

impl std::error::Error for ArcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArcError::CircleError(err) => Some(err),
            _ => None,
        }
    }
}

/// 円弧の構築のためのヘルパートレイト
pub trait ArcBuilder<T: Scalar> {
    type Circle: Circle<T>;
    type Arc: Arc<T, Circle = Self::Circle>;

    /// 円と角度範囲から円弧を作成
    fn from_circle_angles(
        circle: Self::Circle,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Result<Self::Arc, ArcError>;

    /// 中心、半径、角度範囲から円弧を作成
    fn from_center_radius_angles(
        center: <Self::Circle as Circle<T>>::Point,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Result<Self::Arc, ArcError>;

    /// 3点を通る円弧を作成
    fn from_three_points(
        start: <Self::Circle as Circle<T>>::Point,
        middle: <Self::Circle as Circle<T>>::Point,
        end: <Self::Circle as Circle<T>>::Point,
    ) -> Result<Self::Arc, ArcError>;

    /// 開始点、終了点、バルジ（膨らみ）から円弧を作成
    ///
    /// # Arguments
    /// * `start` - 開始点
    /// * `end` - 終了点
    /// * `bulge` - バルジ値（0=直線、正=反時計回り、負=時計回り）
    fn from_start_end_bulge(
        start: <Self::Circle as Circle<T>>::Point,
        end: <Self::Circle as Circle<T>>::Point,
        bulge: T,
    ) -> Result<Self::Arc, ArcError>;
}

/// 円弧の種類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcKind {
    /// 劣弧（180度未満）
    MinorArc,
    /// 優弧（180度以上）
    MajorArc,
    /// 半円（180度）
    Semicircle,
    /// 完全円（360度）
    FullCircle,
}

/// 円弧の分析のためのヘルパートレイト
pub trait ArcAnalysis<T: Scalar> {
    /// 円弧の種類を判定
    fn kind(&self) -> ArcKind;

    /// 円弧の向きを判定
    fn is_counter_clockwise(&self) -> bool;

    /// 円弧の向きを判定
    fn is_clockwise(&self) -> bool {
        !self.is_counter_clockwise()
    }

    /// 円弧の中心角を取得
    fn central_angle(&self) -> Angle<T>
    where
        Self: Arc<T>;
}

#[cfg(test)]
mod tests {
    use super::super::improved_circle::MockCircle2D;
    use super::*;

    // テスト用のモック実装

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct MockPoint2D<T: Scalar> {
        x: T,
        y: T,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct MockVector2D<T: Scalar> {
        x: T,
        y: T,
    }

    #[derive(Debug, Clone)]
    struct MockArc2D<T: Scalar> {
        circle: MockCircle2D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    }

    impl<T: Scalar> Arc<T> for MockArc2D<T> {
        type Circle = MockCircle2D<T>;

        fn circle(&self) -> &Self::Circle {
            &self.circle
        }

        fn start_angle(&self) -> Angle<T> {
            self.start_angle
        }

        fn end_angle(&self) -> Angle<T> {
            self.end_angle
        }

        fn contains_point_on_circumference(&self, _point: &MockPoint2D<T>) -> bool {
            // 簡略化実装
            true
        }

        fn scale(&self, factor: T) -> Self {
            MockArc2D {
                circle: self.circle.scale(factor),
                start_angle: self.start_angle,
                end_angle: self.end_angle,
            }
        }

        fn translate(&self, vector: &MockVector2D<T>) -> Self {
            MockArc2D {
                circle: self.circle.translate(vector),
                start_angle: self.start_angle,
                end_angle: self.end_angle,
            }
        }

        fn reverse(&self) -> Self {
            MockArc2D {
                circle: self.circle,
                start_angle: self.end_angle,
                end_angle: self.start_angle,
            }
        }

        fn bounding_box(&self) -> <Self::Circle as Circle<T>>::BoundingBox {
            // 簡略化：基底円の境界ボックスを返す
            self.circle.bounding_box()
        }
    }

    #[test]
    fn test_arc_interface() {
        let circle = MockCircle2D {
            center: MockPoint2D { x: 0.0, y: 0.0 },
            radius: 5.0,
        };

        let arc = MockArc2D {
            circle,
            start_angle: Angle::from_degrees(0.0),
            end_angle: Angle::from_degrees(90.0),
        };

        assert_eq!(arc.radius(), 5.0);
        assert!((arc.angle_span().degrees() - 90.0).abs() < 1e-10);

        let expected_arc_length = 5.0 * std::f64::consts::PI / 2.0;
        assert!((arc.arc_length() - expected_arc_length).abs() < 1e-10);

        let midpoint = arc.midpoint();
        let expected_mid_angle = 45.0_f64.to_radians();
        assert!((midpoint.x - 5.0 * expected_mid_angle.cos()).abs() < 1e-10);
        assert!((midpoint.y - 5.0 * expected_mid_angle.sin()).abs() < 1e-10);
    }

    #[test]
    fn test_arc_operations() {
        let circle = MockCircle2D {
            center: MockPoint2D { x: 0.0, y: 0.0 },
            radius: 2.0,
        };

        let arc = MockArc2D {
            circle,
            start_angle: Angle::from_degrees(0.0),
            end_angle: Angle::from_degrees(180.0),
        };

        let scaled = arc.scale(2.0);
        assert_eq!(scaled.radius(), 4.0);
        assert!((scaled.angle_span().degrees() - 180.0).abs() < 1e-10);

        let reversed = arc.reverse();
        assert!((reversed.start_angle().degrees() - 180.0).abs() < 1e-10);
        assert!((reversed.end_angle().degrees() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_parameter_point() {
        let circle = MockCircle2D {
            center: MockPoint2D { x: 0.0, y: 0.0 },
            radius: 1.0,
        };

        let arc = MockArc2D {
            circle,
            start_angle: Angle::from_degrees(0.0),
            end_angle: Angle::from_degrees(90.0),
        };

        let point_at_0 = arc.point_at_parameter(0.0);
        assert!((point_at_0.x - 1.0).abs() < 1e-10);
        assert!((point_at_0.y - 0.0).abs() < 1e-10);

        let point_at_1 = arc.point_at_parameter(1.0);
        assert!((point_at_1.x - 0.0).abs() < 1e-10);
        assert!((point_at_1.y - 1.0).abs() < 1e-10);

        let point_at_half = arc.point_at_parameter(0.5);
        let expected_x = 45.0_f64.to_radians().cos();
        let expected_y = 45.0_f64.to_radians().sin();
        assert!((point_at_half.x - expected_x).abs() < 1e-10);
        assert!((point_at_half.y - expected_y).abs() < 1e-10);
    }
}
