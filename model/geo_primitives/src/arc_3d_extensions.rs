//! Arc3D Extensions - Advanced geometric operations and calculations
//!
//! 3次元円弧の拡張メソッド：点計算、コンストラクタ、幾何解析など

use crate::{Angle, Arc3D, Direction3D, Point3D, Vector3D};
use geo_contracts::Scalar;
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, default_kernel_numerical_zero_tolerance,
};

impl<T: Scalar> Arc3D<T> {
    /// 3点を通る円弧を作成
    ///
    /// # 引数
    /// * `start` - 開始点
    /// * `middle` - 中間点
    /// * `end` - 終了点
    pub fn from_three_points(
        start: Point3D<T>,
        middle: Point3D<T>,
        end: Point3D<T>,
    ) -> Option<Self> {
        // 3点が同一直線上でないことを確認
        let v1 = Vector3D::from_points(&start, &middle);
        let v2 = Vector3D::from_points(&middle, &end);
        let cross = v1.cross(&v2);

        if cross.length() < default_distance_tolerance::<T>() {
            return None; // 同一直線上
        }

        // 円の中心と半径を計算
        let center = Self::calculate_circle_center(&start, &middle, &end)?;
        let radius = center.distance_to(&start);

        // 法線ベクトル（外積から）
        let normal = Direction3D::from_vector(cross)?;

        // 開始方向ベクトル
        let start_vec = Vector3D::from_points(&center, &start);
        let start_dir = Direction3D::from_vector(start_vec)?;

        // 角度計算
        let start_angle = Angle::from_radians(T::ZERO); // 基準角度
        let end_vec = Vector3D::from_points(&center, &end);
        let end_dir = Direction3D::from_vector(end_vec)?;

        // 開始点 → 中間点 → 終了点の順に、法線まわりに反時計回りとなる
        let end_angle = Self::calculate_angle_between(&start_dir, &end_dir, &normal)?;

        Self::new(center, radius, normal, start_dir, start_angle, end_angle)
    }

    /// 退化した円弧かどうか判定
    pub fn is_degenerate(&self) -> bool {
        self.radius_internal() <= default_distance_tolerance::<T>()
            || self.angle_span().to_radians() <= default_angle_tolerance::<T>()
    }

    /// Arc trim-local parameter `t` (`0 <= t <= 1`) で円弧上の点を計算
    ///
    /// 母円の local angle parameter へ線形写像して評価する。
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle = self.start_angle().to_radians() + self.angle_span().to_radians() * t;
        self.point_at_angle(angle)
    }

    /// Primitive 局所角度系の角度で円弧上の点を計算する convenience API
    pub fn point_at_angle(&self, angle: T) -> Point3D<T> {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 円弧平面内での2つの直交軸
        let u_axis = self.start_direction();
        let v_axis =
            Direction3D::from_vector(self.normal().as_vector().cross(&u_axis.as_vector())).unwrap();

        let point_on_circle = u_axis.as_vector() * (self.radius_internal() * cos_angle)
            + v_axis.as_vector() * (self.radius_internal() * sin_angle);

        Point3D::new(
            self.center_internal().x() + point_on_circle.x(),
            self.center_internal().y() + point_on_circle.y(),
            self.center_internal().z() + point_on_circle.z(),
        )
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point3D<T> {
        self.point_at_angle(self.start_angle().to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.point_at_angle(self.end_angle().to_radians())
    }

    /// Arc trim-local parameter の有効範囲 `[0, 1]` を返す
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// 3点の外心（3点を通る円の中心）を計算
    ///
    /// `a = p1 - p3`、`b = p2 - p3` として
    /// `p3 + ((|a|² b - |b|² a) × (a × b)) / (2 |a × b|²)`。
    fn calculate_circle_center(
        p1: &Point3D<T>,
        p2: &Point3D<T>,
        p3: &Point3D<T>,
    ) -> Option<Point3D<T>> {
        let a = Vector3D::from_points(p3, p1);
        let b = Vector3D::from_points(p3, p2);
        let a_cross_b = a.cross(&b);
        let denominator = (T::ONE + T::ONE) * a_cross_b.length_squared();
        if denominator <= default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }
        let numerator = (b * a.length_squared() - a * b.length_squared()).cross(&a_cross_b);
        Some(*p3 + numerator * (T::ONE / denominator))
    }

    /// `dir1` から `dir2` まで、法線まわりに反時計回り（右手系の正の向き）に測った角度
    ///
    /// 3点から作る円弧は 180° を超えうるため、`acos` ではなく符号付きの角度で
    /// (0, 2π] の範囲に求める。
    fn calculate_angle_between(
        dir1: &Direction3D<T>,
        dir2: &Direction3D<T>,
        normal: &Direction3D<T>,
    ) -> Option<Angle<T>> {
        let u = dir1.as_vector();
        let v = dir2.as_vector();
        let sin = normal.as_vector().dot(&u.cross(&v));
        let cos = u.dot(&v);
        let mut angle = sin.atan2(cos);
        if angle <= T::ZERO {
            angle += T::TAU;
        }
        Some(Angle::from_radians(angle))
    }

    /// 円弧を等間隔でサンプリング
    ///
    /// # 引数
    /// * `num_points` - 生成する点の数
    ///
    /// # 戻り値
    /// 円弧上の等間隔な点のベクトル
    pub fn sample_points(&self, num_points: usize) -> Vec<Point3D<T>> {
        if num_points == 0 {
            return Vec::new();
        }

        let mut points = Vec::with_capacity(num_points);

        // usizeをT型に変換
        let mut num_points_scalar = T::ZERO;
        for _ in 0..num_points {
            num_points_scalar += T::ONE;
        }

        if num_points == 1 {
            points.push(self.start_point());
            return points;
        }

        let num_segments = num_points_scalar - T::ONE;
        for i in 0..num_points {
            let mut i_scalar = T::ZERO;
            for _ in 0..i {
                i_scalar += T::ONE;
            }

            let t = i_scalar / num_segments;
            points.push(self.point_at_parameter(t));
        }

        points
    }
}
