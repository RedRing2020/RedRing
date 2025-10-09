//! Ellipse - 楕円の抽象化トレイト
//!
//! CAD/CAM システムで使用される楕円の抽象化インターフェース

use crate::abstract_types::geometry::vector::Vector3D;
use crate::Scalar;

/// 2D楕円の基本操作を定義するトレイト
pub trait Ellipse2D<T: Scalar> {
    /// 点の型（通常は Point2D）
    type Point;
    /// ベクトルの型（通常は Vector2D）
    type Vector;
    /// 角度の型（通常は Angle）
    type Angle: Copy;
    /// 境界ボックスの型（通常は BBox2D）
    type BBox;
    /// 楕円弧の型
    type EllipseArc;

    /// 楕円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 楕円の長軸半径を取得
    fn semi_major_axis(&self) -> T;

    /// 楕円の短軸半径を取得
    fn semi_minor_axis(&self) -> T;

    /// 楕円の回転角度を取得（ラジアン）
    fn rotation(&self) -> Self::Angle;

    /// 楕円が円かどうかを判定（長軸と短軸が等しい）
    fn is_circle(&self) -> bool {
        (self.semi_major_axis() - self.semi_minor_axis()).abs() < T::TOLERANCE
    }

    /// 楕円の離心率を計算
    /// 0に近いほど円に近く、1に近いほど細長い楕円
    fn eccentricity(&self) -> T {
        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        if a <= T::TOLERANCE {
            return T::ZERO;
        }
        let c_squared = a * a - b * b;
        if c_squared <= T::ZERO {
            T::ZERO
        } else {
            (c_squared.sqrt()) / a
        }
    }

    /// 楕円の面積を計算
    fn area(&self) -> T {
        T::PI * self.semi_major_axis() * self.semi_minor_axis()
    }

    /// 楕円の周長を計算（ラマヌジャンの近似式を使用）
    fn circumference(&self) -> T {
        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        let h = ((a - b) / (a + b)).powi(2);
        T::PI
            * (a + b)
            * (T::ONE
                + (T::from_f64(3.0) * h)
                    / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
    }

    /// 楕円の焦点を取得（2つの焦点）
    fn foci(&self) -> (Self::Point, Self::Point);

    /// 楕円の境界ボックスを計算
    fn bounding_box(&self) -> Self::BBox;

    /// 指定された角度での楕円周上の点を取得
    /// angle: 楕円のローカル座標系でのラジアン（長軸方向が0度）
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;

    /// 指定されたパラメータでの楕円周上の点を取得
    /// t: パラメータ（0.0〜1.0で一周）
    fn point_at_parameter(&self, t: T) -> Self::Point {
        let angle_radians = t * T::TAU;
        self.point_at_angle(Self::Angle::from_radians(angle_radians))
    }

    /// 指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定された角度での法線ベクトルを取得
    fn normal_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定された点が楕円の内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が楕円周上にあるかを判定
    fn on_boundary(&self, point: &Self::Point) -> bool;

    /// 指定された点から楕円周までの最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 楕円を指定した角度範囲で楕円弧に変換
    fn to_arc(&self, start_angle: Self::Angle, end_angle: Self::Angle) -> Self::EllipseArc;

    /// 楕円を均等分割した点列を取得
    fn approximate_with_points(&self, num_points: usize) -> Vec<Self::Point>;
}

/// 3D楕円の基本操作を定義するトレイト
pub trait Ellipse3D<T: Scalar> {
    /// 点の型（通常は Point3D）
    type Point;
    /// ベクトルの型（通常は Vector3D）
    type Vector: Vector3D;
    /// 方向の型（通常は Direction3D）
    type Direction;
    /// 角度の型（通常は Angle）
    type Angle: Copy;
    /// 境界ボックスの型（通常は BBox3D）
    type BBox;
    /// 2D楕円の型（投影用）
    type Ellipse2D;
    /// 楕円弧の型
    type EllipseArc;

    /// 楕円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 楕円の長軸半径を取得
    fn semi_major_axis(&self) -> T;

    /// 楕円の短軸半径を取得
    fn semi_minor_axis(&self) -> T;

    /// 楕円の法線ベクトルを取得（楕円が存在する平面の法線）
    fn normal(&self) -> Self::Direction;

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> Self::Direction;

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> Self::Direction;

    /// 楕円が円かどうかを判定
    fn is_circle(&self) -> bool {
        (self.semi_major_axis() - self.semi_minor_axis()).abs() < T::TOLERANCE
    }

    /// 楕円の離心率を計算
    fn eccentricity(&self) -> T {
        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        if a <= T::TOLERANCE {
            return T::ZERO;
        }
        let c_squared = a * a - b * b;
        if c_squared <= T::ZERO {
            T::ZERO
        } else {
            (c_squared.sqrt()) / a
        }
    }

    /// 楕円の面積を計算
    fn area(&self) -> T {
        T::PI * self.semi_major_axis() * self.semi_minor_axis()
    }

    /// 楕円の周長を計算
    fn circumference(&self) -> T {
        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        let h = ((a - b) / (a + b)).powi(2);
        T::PI
            * (a + b)
            * (T::ONE
                + (T::from_f64(3.0) * h)
                    / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
    }

    /// 楕円の焦点を取得（2つの焦点）
    fn foci(&self) -> (Self::Point, Self::Point);

    /// 楕円の境界ボックスを計算
    fn bounding_box(&self) -> Self::BBox;

    /// 指定された角度での楕円周上の点を取得
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;

    /// 指定されたパラメータでの楕円周上の点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point {
        let angle_radians = t * T::TAU;
        self.point_at_angle(Self::Angle::from_radians(angle_radians))
    }

    /// 指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定された角度での法線ベクトルを取得（楕円平面内）
    fn normal_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定された角度での双法線ベクトルを取得（楕円平面に垂直）
    fn binormal_at_angle(&self, angle: Self::Angle) -> Self::Vector {
        let tangent = self.tangent_at_angle(angle);
        let normal = self.normal_at_angle(angle);
        tangent.cross(&normal)
    }

    /// 指定された点が楕円の内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が楕円周上にあるかを判定
    fn on_boundary(&self, point: &Self::Point) -> bool;

    /// 指定された点から楕円周までの最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 楕円をXY平面に投影して2D楕円を取得
    fn project_to_xy(&self) -> Self::Ellipse2D;

    /// 楕円を指定した平面に投影
    fn project_to_plane(
        &self,
        plane_normal: &Self::Direction,
        plane_point: &Self::Point,
    ) -> Self::Ellipse2D;

    /// 楕円を指定した角度範囲で楕円弧に変換
    fn to_arc(&self, start_angle: Self::Angle, end_angle: Self::Angle) -> Self::EllipseArc;

    /// 楕円を均等分割した点列を取得
    fn approximate_with_points(&self, num_points: usize) -> Vec<Self::Point>;

    /// ローカル座標系を構築（長軸、短軸、法線の直交座標系）
    fn local_coordinate_system(&self) -> (Self::Direction, Self::Direction, Self::Direction) {
        (
            self.major_axis_direction(),
            self.minor_axis_direction(),
            self.normal(),
        )
    }
}

/// 楕円の構築に関するトレイト
pub trait EllipseBuilder<T: Scalar> {
    /// エラー型
    type Error;
    /// 構築される楕円の型
    type Ellipse;
    /// 点の型
    type Point;
    /// 角度の型
    type Angle;
    /// 方向の型（3Dの場合）
    type Direction;

    /// 中心、軸半径、回転角から楕円を構築
    fn from_center_and_radii(
        center: Self::Point,
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: Self::Angle,
    ) -> Result<Self::Ellipse, Self::Error>;

    /// 5点から楕円を構築
    fn from_five_points(points: [Self::Point; 5]) -> Result<Self::Ellipse, Self::Error>;

    /// 中心と2つの焦点から楕円を構築
    fn from_center_and_foci(
        center: Self::Point,
        focus1: Self::Point,
        focus2: Self::Point,
    ) -> Result<Self::Ellipse, Self::Error>;

    /// 楕円の接線条件から構築
    fn from_tangent_conditions(
        center: Self::Point,
        tangent_points: &[Self::Point],
        tangent_directions: &[Self::Direction],
    ) -> Result<Self::Ellipse, Self::Error>;
}

/// 楕円の変換操作に関するトレイト
pub trait EllipseTransform<T: Scalar> {
    /// 変換後の楕円の型
    type TransformedEllipse;
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 角度の型
    type Angle;

    /// 楕円を平行移動
    fn translate(&self, translation: Self::Vector) -> Self::TransformedEllipse;

    /// 楕円を指定した点を中心に回転
    fn rotate(&self, center: Self::Point, angle: Self::Angle) -> Self::TransformedEllipse;

    /// 楕円を指定した点を中心にスケール
    fn scale(&self, center: Self::Point, scale_factor: T) -> Self::TransformedEllipse;

    /// 楕円を非一様スケール
    fn scale_non_uniform(
        &self,
        center: Self::Point,
        scale_x: T,
        scale_y: T,
    ) -> Self::TransformedEllipse;

    /// 楕円をミラー（指定した軸を中心に反転）
    fn mirror(
        &self,
        axis_point: Self::Point,
        axis_direction: Self::Vector,
    ) -> Self::TransformedEllipse;

    /// 楕円をせん断変形
    fn shear(&self, shear_vector: Self::Vector) -> Self::TransformedEllipse;
}

/// 楕円の解析操作に関するトレイト
pub trait EllipseAnalysis<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 角度の型
    type Angle;

    /// 指定した点での楕円の曲率を計算
    fn curvature_at_point(&self, point: &Self::Point) -> T;

    /// 指定した角度での楕円の曲率を計算
    fn curvature_at_angle(&self, angle: Self::Angle) -> T;

    /// 楕円の最大曲率を取得
    fn max_curvature(&self) -> T {
        // 楕円の最大曲率は短軸の端点で発生し、1/a（aは長軸半径）
        T::ONE / self.semi_major_axis()
    }

    /// 楕円の最小曲率を取得
    fn min_curvature(&self) -> T {
        // 楕円の最小曲率は長軸の端点で発生し、1/b（bは短軸半径）
        T::ONE / self.semi_minor_axis()
    }

    /// 楕円の軸半径を取得（必要なトレイトメソッド）
    fn semi_major_axis(&self) -> T;
    fn semi_minor_axis(&self) -> T;

    /// 指定した直線との交点を計算
    fn intersect_with_line(
        &self,
        line_point: Self::Point,
        line_direction: Self::Vector,
    ) -> Vec<Self::Point>;

    /// 他の楕円との交点を計算
    fn intersect_with_ellipse(&self, other: &Self) -> Vec<Self::Point>
    where
        Self: Sized;

    /// 指定した点から楕円への接線を計算
    fn tangent_lines_from_point(&self, point: Self::Point) -> Vec<(Self::Point, Self::Vector)>;
}
