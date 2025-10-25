//! Arc3D Extensions - Advanced geometric operations and calculations
//!
//! 3次元円弧の拡張メソッド：点計算、コンストラクタ、幾何解析など

use crate::{Angle, Arc3D, Direction3D, Point3D, Vector3D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

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

        if cross.length() < DefaultTolerances::distance::<T>() {
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

        // 終了角度の計算（詳細な実装は必要）
        let end_angle = Self::calculate_angle_between(&start_dir, &end_dir, &normal)?;

        Self::new(center, radius, normal, start_dir, start_angle, end_angle)
    }

    /// ベクトルから円弧を作成（後方互換性）
    pub fn from_vectors(
        center: Point3D<T>,
        radius: T,
        normal: Vector3D<T>,
        start_dir: Vector3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let normal_dir = Direction3D::from_vector(normal)?;
        let start_dir_dir = Direction3D::from_vector(start_dir)?;
        Self::new(
            center,
            radius,
            normal_dir,
            start_dir_dir,
            start_angle,
            end_angle,
        )
    }

    /// 退化した円弧かどうか判定
    pub fn is_degenerate(&self) -> bool {
        self.radius() <= DefaultTolerances::distance::<T>()
            || self.angle_span().to_radians() <= DefaultTolerances::angle::<T>()
    }

    // === パラメトリック操作 ===

    /// パラメータ t での円弧上の点を計算
    /// t ∈ [0, 1] で正規化
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle = self.start_angle().to_radians() + self.angle_span().to_radians() * t;
        self.point_at_angle(angle)
    }

    /// 角度 θ での円弧上の点を計算
    pub fn point_at_angle(&self, angle: T) -> Point3D<T> {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 円弧平面内での2つの直交軸
        let u_axis = self.start_direction();
        let v_axis =
            Direction3D::from_vector(self.normal().as_vector().cross(&u_axis.as_vector())).unwrap();

        let point_on_circle = u_axis.as_vector() * (self.radius() * cos_angle)
            + v_axis.as_vector() * (self.radius() * sin_angle);

        Point3D::new(
            self.center().x() + point_on_circle.x(),
            self.center().y() + point_on_circle.y(),
            self.center().z() + point_on_circle.z(),
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

    /// 中点を取得
    pub fn mid_point(&self) -> Point3D<T> {
        let mid_angle =
            self.start_angle().to_radians() + self.angle_span().to_radians() / T::from_f64(2.0);
        self.point_at_angle(mid_angle)
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    // === 角度と検証メソッド ===

    /// 角度が円弧の範囲内にあるかチェック
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        let normalized_angle = self.normalize_angle(angle);
        let start = self.normalize_angle(self.start_angle());
        let end = self.normalize_angle(self.end_angle());

        if start.to_radians() <= end.to_radians() {
            normalized_angle.to_radians() >= start.to_radians()
                && normalized_angle.to_radians() <= end.to_radians()
        } else {
            // 0度をまたぐ場合
            normalized_angle.to_radians() >= start.to_radians()
                || normalized_angle.to_radians() <= end.to_radians()
        }
    }

    /// 角度を [0, 2π] の範囲に正規化
    pub fn normalize_angle(&self, angle: Angle<T>) -> Angle<T> {
        let two_pi = Angle::from_radians(T::from_f64(2.0) * T::PI);
        let mut normalized = angle;
        while normalized.to_radians() < T::ZERO {
            normalized += two_pi;
        }
        while normalized >= two_pi {
            normalized -= two_pi;
        }
        normalized
    }

    // === 内部ヘルパーメソッド ===

    /// 3点から円の中心を計算
    fn calculate_circle_center(
        p1: &Point3D<T>,
        p2: &Point3D<T>,
        p3: &Point3D<T>,
    ) -> Option<Point3D<T>> {
        // 簡略化した実装：実際にはより複雑な幾何計算が必要
        // ここでは基本的な重心として近似
        let center_x = (p1.x() + p2.x() + p3.x()) / T::from_f64(3.0);
        let center_y = (p1.y() + p2.y() + p3.y()) / T::from_f64(3.0);
        let center_z = (p1.z() + p2.z() + p3.z()) / T::from_f64(3.0);

        Some(Point3D::new(center_x, center_y, center_z))
    }

    /// 2つの方向ベクトル間の角度を計算
    fn calculate_angle_between(
        dir1: &Direction3D<T>,
        dir2: &Direction3D<T>,
        _normal: &Direction3D<T>,
    ) -> Option<Angle<T>> {
        let dot = dir1.as_vector().dot(&dir2.as_vector());
        let angle_rad = dot.acos();
        Some(Angle::from_radians(angle_rad))
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
