//! 改善されたCircle (円) トレイト定義
//!
//! ジェネリック設計でf32/f64両対応、2D/3D統一インターフェース

use super::angle::{Angle, Scalar};

/// 統一されたCircleトレイト
///
/// # 特徴
/// - ジェネリック設計でf32/f64両対応
/// - 2D/3D共通のインターフェース
/// - 型安全な角度処理
pub trait Circle<T: Scalar> {
    /// 点の型（Point2D, Point3D など）
    type Point;
    /// ベクトルの型（Vector2D, Vector3D など）
    type Vector;
    /// 境界ボックスの型（通常は BBox2D または BBox3D）
    type BBox;

    /// 次元数を取得（2または3）
    fn dimension() -> usize;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> T;

    /// 円の面積を計算
    fn area(&self) -> T {
        T::pi() * self.radius() * self.radius()
    }

    /// 円の周長（円周）を計算
    fn circumference(&self) -> T {
        T::tau() * self.radius()
    }

    /// 円の直径を計算
    fn diameter(&self) -> T {
        T::two() * self.radius()
    }

    /// 指定された点が円の内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が円周上にあるかを判定（許容誤差内）
    fn on_circumference(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 円周上の指定された角度での点を取得
    ///
    /// # Arguments
    /// * `angle` - 角度（0度で+X軸方向、反時計回り）
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;

    /// 円周上の指定された点での接線ベクトルを取得
    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector>;

    /// 円周上の指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector;

    /// 円の境界ボックス（外接矩形/直方体）を取得
    fn bounding_box(&self) -> Self::BBox;

    /// 円を指定倍率で拡大縮小
    ///
    /// # Arguments
    /// * `factor` - 拡大縮小係数（正の値）
    fn scale(&self, factor: T) -> Self
    where
        Self: Sized;

    /// 円を指定ベクトルで平行移動
    fn translate(&self, vector: &Self::Vector) -> Self
    where
        Self: Sized;

    /// 円が退化しているか（半径が0またはそれに近い）を判定
    fn is_degenerate(&self, tolerance: T) -> bool {
        self.radius() <= tolerance
    }

    /// 指定された点までの符号付き距離を取得
    ///
    /// # Returns
    /// - 正の値: 円の外部の点までの距離
    /// - 負の値: 円の内部の点までの距離（絶対値が円周までの距離）
    /// - 0: 円周上の点
    fn signed_distance_to_point(&self, point: &Self::Point) -> T;
}

/// 2D円専用の追加機能
pub trait Circle2D<T: Scalar>: Circle<T> {
    /// 2つの円の交点を計算
    ///
    /// # Returns
    /// - 0個: 交差しない/同心円
    /// - 1個: 接する
    /// - 2個: 交差する
    fn intersection_points(&self, other: &impl Circle2D<T>) -> Vec<Self::Point>;

    /// 点から円への接線を計算
    ///
    /// # Arguments
    /// * `point` - 外部の点
    ///
    /// # Returns
    /// 接線の角度ペア（通常2本）
    fn tangent_angles_from_point(&self, point: &Self::Point) -> Vec<Angle<T>>;

    /// 円の向きを取得（時計回り/反時計回り）
    fn orientation(&self) -> Orientation;
}

/// 3D円専用の追加機能
pub trait Circle3D<T: Scalar>: Circle<T> {
    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 円の局所X軸（0度方向）を取得
    fn u_axis(&self) -> Self::Vector;

    /// 円の局所Y軸（90度方向）を取得
    fn v_axis(&self) -> Self::Vector;

    /// 指定された点が円の平面上にあるかを判定
    fn point_on_plane(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 点を円の平面に投影
    fn project_point_to_plane(&self, point: &Self::Point) -> Self::Point;

    /// 円の平面への投影を2D円として取得
    fn to_2d(&self) -> impl Circle2D<T>;

    /// 3D空間での円の向きを定義する平面を取得
    fn plane(&self) -> Plane<T, Self::Point, Self::Vector>;
}

/// 円の向き
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    /// 反時計回り（右手系で正の向き）
    CounterClockwise,
    /// 時計回り（右手系で負の向き）
    Clockwise,
}

/// 3D平面を表現する構造体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane<T: Scalar, P, V> {
    /// 平面上の任意の点
    pub point: P,
    /// 平面の法線ベクトル
    pub normal: V,
    /// スカラー型情報
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Scalar, P, V> Plane<T, P, V> {
    /// 新しい平面を作成
    pub fn new(point: P, normal: V) -> Self {
        Self {
            point,
            normal,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// 円の種類を表現する列挙型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircleKind {
    /// 通常の円
    Circle,
    /// 単位円（半径1）
    UnitCircle,
    /// 点円（半径0）
    PointCircle,
}

/// 円の構築エラー
#[derive(Debug, Clone, PartialEq)]
pub enum CircleError {
    /// 無効な半径（負の値またはNaN）
    InvalidRadius,
    /// 無効な中心点
    InvalidCenter,
    /// 無効な法線ベクトル（3D円）
    InvalidNormal,
    /// 共線点（3点円の構築時）
    CollinearPoints,
}

impl std::fmt::Display for CircleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircleError::InvalidRadius => write!(f, "半径は非負の有限値である必要があります"),
            CircleError::InvalidCenter => write!(f, "中心点が無効です"),
            CircleError::InvalidNormal => write!(f, "法線ベクトルが無効です"),
            CircleError::CollinearPoints => write!(f, "3つの点が一直線上にあります"),
        }
    }
}

impl std::error::Error for CircleError {}

/// 円の構築と操作のためのヘルパートレイト
pub trait CircleBuilder<T: Scalar> {
    type Point;
    type Vector;
    type Circle: Circle<T>;

    /// 中心と半径から円を作成
    fn from_center_radius(center: Self::Point, radius: T) -> Result<Self::Circle, CircleError>;

    /// 原点を中心とする円を作成
    fn from_radius(radius: T) -> Result<Self::Circle, CircleError>;

    /// 単位円を作成
    fn unit_circle() -> Self::Circle;

    /// 3点を通る円を作成
    fn from_three_points(
        p1: Self::Point,
        p2: Self::Point,
        p3: Self::Point,
    ) -> Result<Self::Circle, CircleError>;

    /// 直径の両端点から円を作成
    fn from_diameter(p1: Self::Point, p2: Self::Point) -> Result<Self::Circle, CircleError>;
}

/// 円の分析と計算のためのヘルパートレイト
pub trait CircleAnalysis<T: Scalar> {
    /// 円の種類を判定
    fn kind(&self) -> CircleKind;

    /// 円の面積を正確に計算（高精度）
    fn area_precise(&self) -> T;

    /// 円周の長さを正確に計算（高精度）
    fn circumference_precise(&self) -> T;

    /// 円のモーメント（慣性モーメント）を計算
    fn moment_of_inertia(&self) -> T;

    /// 円の重心（中心）を取得
    fn centroid(&self) -> <Self as Circle<T>>::Point
    where
        Self: Circle<T>;
}

// 便利な型エイリアス
pub type Angle32 = Angle<f32>;
pub type Angle64 = Angle<f64>;

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用のモック実装（実際の使用例のデモンストレーション）

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

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct MockBBox2D<T: Scalar> {
        min: MockPoint2D<T>,
        max: MockPoint2D<T>,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct MockCircle2D<T: Scalar> {
        center: MockPoint2D<T>,
        radius: T,
    }

    impl<T: Scalar> Circle<T> for MockCircle2D<T> {
        type Point = MockPoint2D<T>;
        type Vector = MockVector2D<T>;
        type BBox = MockBBox2D<T>;

        fn dimension() -> usize {
            2
        }

        fn center(&self) -> Self::Point {
            self.center
        }

        fn radius(&self) -> T {
            self.radius
        }

        fn contains_point(&self, point: &Self::Point) -> bool {
            let dx = point.x - self.center.x;
            let dy = point.y - self.center.y;
            dx * dx + dy * dy <= self.radius * self.radius
        }

        fn on_circumference(&self, point: &Self::Point, tolerance: T) -> bool {
            let dx = point.x - self.center.x;
            let dy = point.y - self.center.y;
            let distance = (dx * dx + dy * dy).sqrt();
            (distance - self.radius).abs() <= tolerance
        }

        fn point_at_angle(&self, angle: Angle<T>) -> Self::Point {
            let rad = angle.radians();
            MockPoint2D {
                x: self.center.x + self.radius * rad.cos(),
                y: self.center.y + self.radius * rad.sin(),
            }
        }

        fn tangent_at_point(&self, _point: &Self::Point) -> Option<Self::Vector> {
            // 簡略化実装
            Some(MockVector2D {
                x: T::zero(),
                y: T::one(),
            })
        }

        fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector {
            let rad = angle.radians();
            MockVector2D {
                x: -self.radius * rad.sin(),
                y: self.radius * rad.cos(),
            }
        }

        fn bounding_box(&self) -> Self::BBox {
            MockBBox2D {
                min: MockPoint2D {
                    x: self.center.x - self.radius,
                    y: self.center.y - self.radius,
                },
                max: MockPoint2D {
                    x: self.center.x + self.radius,
                    y: self.center.y + self.radius,
                },
            }
        }

        fn scale(&self, factor: T) -> Self {
            MockCircle2D {
                center: self.center,
                radius: self.radius * factor,
            }
        }

        fn translate(&self, vector: &Self::Vector) -> Self {
            MockCircle2D {
                center: MockPoint2D {
                    x: self.center.x + vector.x,
                    y: self.center.y + vector.y,
                },
                radius: self.radius,
            }
        }

        fn signed_distance_to_point(&self, point: &Self::Point) -> T {
            let dx = point.x - self.center.x;
            let dy = point.y - self.center.y;
            let distance = (dx * dx + dy * dy).sqrt();
            distance - self.radius
        }
    }

    #[test]
    fn test_circle_interface() {
        let circle = MockCircle2D {
            center: MockPoint2D { x: 0.0, y: 0.0 },
            radius: 5.0,
        };

        assert_eq!(circle.radius(), 5.0);
        assert!((circle.area() - (std::f64::consts::PI * 25.0)).abs() < 1e-10);
        assert!((circle.circumference() - (std::f64::consts::TAU * 5.0)).abs() < 1e-10);

        let point = MockPoint2D { x: 3.0, y: 4.0 };
        assert!(circle.contains_point(&point));

        let angle = Angle::from_degrees(90.0);
        let point_at_90 = circle.point_at_angle(angle);
        assert!((point_at_90.x - 0.0).abs() < 1e-10);
        assert!((point_at_90.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_circle_operations() {
        let circle = MockCircle2D {
            center: MockPoint2D { x: 1.0, y: 2.0 },
            radius: 3.0,
        };

        let scaled = circle.scale(2.0);
        assert_eq!(scaled.radius(), 6.0);
        assert_eq!(scaled.center().x, 1.0);
        assert_eq!(scaled.center().y, 2.0);

        let vector = MockVector2D { x: 5.0, y: -3.0 };
        let translated = circle.translate(&vector);
        assert_eq!(translated.radius(), 3.0);
        assert_eq!(translated.center().x, 6.0);
        assert_eq!(translated.center().y, -1.0);
    }
}
