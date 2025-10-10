//! EllipseArc (楕円弧) トレイト定義
//!
//! 2D/3D空間における楕円弧の抽象的なインターフェースを提供

use crate::abstract_types::geometry::vector::{Vector3D, Vector3DGeometry};
use analysis::AngleType;
use analysis::Scalar;

/// 2D楕円弧の基本操作を定義するトレイト
pub trait EllipseArc2D<T: Scalar> {
    /// 点の型（通常は Point2D）
    type Point;
    /// ベクトルの型（通常は Vector2D）
    type Vector;
    /// 角度の型（通常は Angle）
    type Angle: Copy + AngleType<Scalar = T>;
    /// 楕円の型（通常は Ellipse2D）
    type Ellipse;
    /// 境界ボックスの型（通常は BBox2D）
    type BBox;

    /// 楕円弧の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 楕円弧の長軸半径を取得
    fn semi_major_axis(&self) -> T;

    /// 楕円弧の短軸半径を取得
    fn semi_minor_axis(&self) -> T;

    /// 楕円弧の回転角度を取得（ラジアン）
    fn rotation(&self) -> Self::Angle;

    /// 楕円弧の開始角度を取得（楕円のローカル座標系でのラジアン）
    fn start_angle(&self) -> Self::Angle;

    /// 楕円弧の終了角度を取得（楕円のローカル座標系でのラジアン）
    fn end_angle(&self) -> Self::Angle;

    /// 楕円弧の角度範囲を取得（終了角度 - 開始角度）
    fn angle_range(&self) -> Self::Angle;

    /// 楕円弧が完全な楕円かどうかを判定
    fn is_full_ellipse(&self) -> bool {
        let range = self.angle_range();
        // 2π ラジアン（360度）に近い場合は完全な楕円
        (range.to_radians() - T::TAU).abs() < T::TOLERANCE
    }

    /// 楕円弧が円弧かどうかを判定（長軸と短軸が等しい）
    fn is_circular(&self) -> bool {
        (self.semi_major_axis() - self.semi_minor_axis()).abs() < T::TOLERANCE
    }

    /// 楕円弧の離心率を計算
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

    /// 楕円弧の面積を計算（セクター面積）
    fn area(&self) -> T {
        let range = self.angle_range().to_radians();
        let ellipse_area = T::PI * self.semi_major_axis() * self.semi_minor_axis();
        ellipse_area * range / T::TAU
    }

    /// 楕円弧の弧長を計算
    fn arc_length(&self) -> T;

    /// 楕円弧の境界ボックスを計算
    fn bounding_box(&self) -> Self::BBox;

    /// 指定されたパラメータ位置での点を取得
    /// t: パラメータ（通常は0.0〜1.0、開始角度から終了角度まで）
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定された角度での点を取得（楕円のローカル座標系）
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;

    /// 指定されたパラメータ位置での接線ベクトルを取得
    fn tangent_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定されたパラメータ位置での法線ベクトルを取得
    fn normal_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定された角度での法線ベクトルを取得
    fn normal_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定された点が楕円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が楕円弧の角度範囲内にあるかを判定
    fn point_in_angle_range(&self, point: &Self::Point) -> bool;

    /// 楕円弧の開始点を取得
    fn start_point(&self) -> Self::Point {
        self.point_at_angle(self.start_angle())
    }

    /// 楕円弧の終了点を取得
    fn end_point(&self) -> Self::Point {
        self.point_at_angle(self.end_angle())
    }

    /// 楕円弧の中点を取得
    fn midpoint(&self) -> Self::Point {
        let mid_angle_radians =
            (self.start_angle().to_radians() + self.end_angle().to_radians()) / (T::ONE + T::ONE);
        self.point_at_angle(Self::Angle::from_radians(mid_angle_radians))
    }

    /// 楕円弧を含む完全な楕円を取得
    fn parent_ellipse(&self) -> Self::Ellipse;

    /// 楕円弧を指定した角度分割で近似する点列を取得
    fn approximate_with_points(&self, num_segments: usize) -> Vec<Self::Point>;

    /// 楕円弧を反転（開始角度と終了角度を入れ替え）
    fn reverse(&self) -> Self;
}

/// 3D楕円弧の基本操作を定義するトレイト
pub trait EllipseArc3D<T: Scalar> {
    /// 点の型（通常は Point3D）
    type Point;
    /// ベクトルの型（通常は Vector3D）
    type Vector: Vector3D<T>;
    /// 方向の型（通常は Direction3D）
    type Direction;
    /// 角度の型（通常は Angle）
    type Angle: Copy + AngleType<Scalar = T>;
    /// 楕円の型（通常は Ellipse3D）
    type Ellipse;
    /// 境界ボックスの型（通常は BBox3D）
    type BBox;
    /// 2D楕円弧の型（投影用）
    type EllipseArc2D;

    /// 楕円弧の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 楕円弧の長軸半径を取得
    fn semi_major_axis(&self) -> T;

    /// 楕円弧の短軸半径を取得
    fn semi_minor_axis(&self) -> T;

    /// 楕円弧の法線ベクトルを取得（楕円が存在する平面の法線）
    fn normal(&self) -> Self::Direction;

    /// 楕円弧の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> Self::Direction;

    /// 楕円弧の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> Self::Direction;

    /// 楕円弧の開始角度を取得（楕円のローカル座標系でのラジアン）
    fn start_angle(&self) -> Self::Angle;

    /// 楕円弧の終了角度を取得（楕円のローカル座標系でのラジアン）
    fn end_angle(&self) -> Self::Angle;

    /// 楕円弧の角度範囲を取得
    fn angle_range(&self) -> Self::Angle;

    /// 楕円弧が完全な楕円かどうかを判定
    fn is_full_ellipse(&self) -> bool {
        let range = self.angle_range();
        (range.to_radians() - T::TAU).abs() < T::TOLERANCE
    }

    /// 楕円弧が円弧かどうかを判定
    fn is_circular(&self) -> bool {
        (self.semi_major_axis() - self.semi_minor_axis()).abs() < T::TOLERANCE
    }

    /// 楕円弧の離心率を計算
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

    /// 楕円弧の面積を計算（セクター面積）
    fn area(&self) -> T {
        let range = self.angle_range().to_radians();
        let ellipse_area = T::PI * self.semi_major_axis() * self.semi_minor_axis();
        ellipse_area * range / T::TAU
    }

    /// 楕円弧の弧長を計算
    fn arc_length(&self) -> T;

    /// 楕円弧の境界ボックスを計算
    fn bounding_box(&self) -> Self::BBox;

    /// 指定されたパラメータ位置での点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定された角度での点を取得
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;

    /// 指定されたパラメータ位置での接線ベクトルを取得
    fn tangent_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定されたパラメータ位置での法線ベクトルを取得（楕円平面内）
    fn normal_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定された角度での法線ベクトルを取得（楕円平面内）
    fn normal_at_angle(&self, angle: Self::Angle) -> Self::Vector;

    /// 指定されたパラメータ位置での双法線ベクトルを取得（楕円平面に垂直）
    fn binormal_at_parameter(&self, t: T) -> Self::Vector
    where
        Self::Vector: Vector3DGeometry<T>,
    {
        let tangent = self.tangent_at_parameter(t);
        let normal = self.normal_at_parameter(t);
        tangent.cross(&normal)
    }

    /// 指定されたパラメータ位置での曲率を取得
    fn curvature_at_parameter(&self, t: T) -> T;

    /// 指定されたパラメータ位置での曲率半径を取得
    fn curvature_radius_at_parameter(&self, t: T) -> T {
        let curvature = self.curvature_at_parameter(t);
        if curvature.abs() < T::TOLERANCE {
            T::INFINITY
        } else {
            T::ONE / curvature
        }
    }

    /// 指定されたパラメータ位置での接線と曲率を同時計算（効率的）
    fn tangent_and_curvature_at_parameter(&self, t: T) -> (Self::Vector, T) {
        (self.tangent_at_parameter(t), self.curvature_at_parameter(t))
    }

    /// 楕円弧上の最大曲率位置（短軸端点）を取得
    fn max_curvature_point(&self) -> Option<(T, T)> {
        // 楕円の場合、短軸端点で最大曲率となる
        // デフォルト実装では None、具象型で実装
        None
    }

    /// 楕円弧上の最小曲率位置（長軸端点）を取得
    fn min_curvature_point(&self) -> Option<(T, T)> {
        // 楕円の場合、長軸端点で最小曲率となる
        // デフォルト実装では None、具象型で実装
        None
    }

    /// 指定された点が楕円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が楕円弧の角度範囲内にあるかを判定
    fn point_in_angle_range(&self, point: &Self::Point) -> bool;

    /// 楕円弧の開始点を取得
    fn start_point(&self) -> Self::Point {
        self.point_at_angle(self.start_angle())
    }

    /// 楕円弧の終了点を取得
    fn end_point(&self) -> Self::Point {
        self.point_at_angle(self.end_angle())
    }

    /// 楕円弧の中点を取得
    fn midpoint(&self) -> Self::Point {
        let mid_angle_radians =
            (self.start_angle().to_radians() + self.end_angle().to_radians()) / (T::ONE + T::ONE);
        self.point_at_angle(Self::Angle::from_radians(mid_angle_radians))
    }

    /// 楕円弧を含む完全な楕円を取得
    fn parent_ellipse(&self) -> Self::Ellipse;

    /// 楕円弧をXY平面に投影して2D楕円弧を取得
    fn project_to_xy(&self) -> Self::EllipseArc2D;

    /// 楕円弧を指定した平面に投影
    fn project_to_plane(
        &self,
        plane_normal: &Self::Direction,
        plane_point: &Self::Point,
    ) -> Self::EllipseArc2D;

    /// 楕円弧を指定した角度分割で近似する点列を取得
    fn approximate_with_points(&self, num_segments: usize) -> Vec<Self::Point>;

    /// 楕円弧を反転（開始角度と終了角度を入れ替え）
    fn reverse(&self) -> Self;

    /// ローカル座標系を構築（長軸、短軸、法線の直交座標系）
    fn local_coordinate_system(&self) -> (Self::Direction, Self::Direction, Self::Direction) {
        (
            self.major_axis_direction(),
            self.minor_axis_direction(),
            self.normal(),
        )
    }
}

/// 楕円弧の構築に関するトレイト
pub trait EllipseArcBuilder<T: Scalar> {
    /// エラー型
    type Error;
    /// 構築される楕円弧の型
    type EllipseArc;
    /// 点の型
    type Point;
    /// 角度の型
    type Angle: AngleType<Scalar = T>;
    /// 楕円の型
    type Ellipse;

    /// 中心、軸半径、回転角、角度範囲から楕円弧を構築
    fn from_center_and_angles(
        center: Self::Point,
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: Self::Angle,
        start_angle: Self::Angle,
        end_angle: Self::Angle,
    ) -> Result<Self::EllipseArc, Self::Error>;

    /// 楕円と角度範囲から楕円弧を構築
    fn from_ellipse_and_angles(
        ellipse: Self::Ellipse,
        start_angle: Self::Angle,
        end_angle: Self::Angle,
    ) -> Result<Self::EllipseArc, Self::Error>;

    /// 3点を通る楕円弧を構築
    fn from_three_points(
        start: Self::Point,
        middle: Self::Point,
        end: Self::Point,
    ) -> Result<Self::EllipseArc, Self::Error>;

    /// 開始点、終了点、中間点の接線から楕円弧を構築
    fn from_points_and_tangents(
        start: Self::Point,
        end: Self::Point,
        start_tangent: Self::Point,
        end_tangent: Self::Point,
    ) -> Result<Self::EllipseArc, Self::Error>;
}

/// 楕円弧の変換操作に関するトレイト
pub trait EllipseArcTransform<T: Scalar> {
    /// 変換後の楕円弧の型
    type TransformedEllipseArc;
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 角度の型
    type Angle: AngleType<Scalar = T>;

    /// 楕円弧を平行移動
    fn translate(&self, translation: Self::Vector) -> Self::TransformedEllipseArc;

    /// 楕円弧を指定した点を中心に回転
    fn rotate(&self, center: Self::Point, angle: Self::Angle) -> Self::TransformedEllipseArc;

    /// 楕円弧を指定した点を中心にスケール
    fn scale(&self, center: Self::Point, scale_factor: T) -> Self::TransformedEllipseArc;

    /// 楕円弧を非一様スケール（X軸とY軸で異なる倍率）
    fn scale_non_uniform(
        &self,
        center: Self::Point,
        scale_x: T,
        scale_y: T,
    ) -> Self::TransformedEllipseArc;

    /// 楕円弧をミラー（指定した軸を中心に反転）
    fn mirror(
        &self,
        axis_point: Self::Point,
        axis_direction: Self::Vector,
    ) -> Self::TransformedEllipseArc;
}

/// 楕円弧の分割・切断操作に関するトレイト
pub trait EllipseArcSplit<T: Scalar> {
    /// 角度の型
    type Angle: AngleType<Scalar = T>;
    /// エラー型
    type Error;

    /// 指定した角度で楕円弧を分割
    fn split_at_angle(&self, angle: Self::Angle) -> Result<(Self, Self), Self::Error>
    where
        Self: Sized;

    /// 指定したパラメータで楕円弧を分割
    fn split_at_parameter(&self, t: T) -> Result<(Self, Self), Self::Error>
    where
        Self: Sized;

    /// 楕円弧を等角度で指定した数に分割
    fn split_into_equal_angles(&self, num_segments: usize) -> Result<Vec<Self>, Self::Error>
    where
        Self: Sized;

    /// 楕円弧を等弧長で指定した数に分割（近似）
    fn split_into_equal_lengths(&self, num_segments: usize) -> Result<Vec<Self>, Self::Error>
    where
        Self: Sized;

    /// 指定した角度範囲を切り取り
    fn trim_to_angles(
        &self,
        new_start_angle: Self::Angle,
        new_end_angle: Self::Angle,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
