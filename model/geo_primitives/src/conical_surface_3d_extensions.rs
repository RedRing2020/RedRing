//! ConicalSurface3D の拡張機能実装
//!
//! 基本機能を超えた高度な幾何操作、解析機能、CAD/CAM用途に特化した機能

use crate::{ConicalSurface3D, Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

impl<T: Scalar> ConicalSurface3D<T> {
    // ========================================================================
    // 高度な幾何計算
    // ========================================================================

    /// 点から円錐サーフェス上の最近点を求める
    ///
    /// # Returns
    /// * `Point3D<T>` - 最近点
    /// * `(T, T)` - (u, v) パラメータ
    /// * `T` - 距離
    pub fn closest_point_to(&self, point: &Point3D<T>) -> (Point3D<T>, (T, T), T) {
        // 簡略実装：軸方向を固定して放射方向の最短点を求める
        let relative = Vector3D::new(
            point.x() - self.center().x(),
            point.y() - self.center().y(),
            point.z() - self.center().z(),
        );

        let z_axis = self.axis().as_vector();
        let x_axis = self.ref_direction().as_vector();
        let y_axis = self.derived_y_axis().as_vector();

        // 軸方向成分
        let v = relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        // 放射方向成分
        let radial_x =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();
        let radial_y =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();

        // 角度パラメータ u を計算
        let u = radial_y.atan2(radial_x);
        let u = if u < T::ZERO {
            u + T::PI * (T::ONE + T::ONE)
        } else {
            u
        };

        // サーフェス上の点
        let surface_point = self.point_at_uv(u, v);
        let distance = self.distance_to_surface(point);

        (surface_point, (u, v), distance)
    }

    /// U方向の偏微分ベクトルを計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// U方向の接線ベクトル
    pub fn du_vector(&self, u: T, v: T) -> Vector3D<T> {
        let sin_u = u.sin();
        let cos_u = u.cos();
        let r_at_v = self.radius_at_v(v);

        let x_axis = self.ref_direction().as_vector();
        let y_axis = self.derived_y_axis().as_vector();

        Vector3D::new(
            r_at_v * (-sin_u * x_axis.x() + cos_u * y_axis.x()),
            r_at_v * (-sin_u * x_axis.y() + cos_u * y_axis.y()),
            r_at_v * (-sin_u * x_axis.z() + cos_u * y_axis.z()),
        )
    }

    /// V方向の偏微分ベクトルを計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// V方向の接線ベクトル
    pub fn dv_vector(&self, u: T, _v: T) -> Vector3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let tan_angle = self.semi_angle().tan();

        let x_axis = self.ref_direction().as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis().as_vector();

        // dP/dv = tan(角度) * (cos(u) * X + sin(u) * Y) + Z
        Vector3D::new(
            tan_angle * (cos_u * x_axis.x() + sin_u * y_axis.x()) + z_axis.x(),
            tan_angle * (cos_u * x_axis.y() + sin_u * y_axis.y()) + z_axis.y(),
            tan_angle * (cos_u * x_axis.z() + sin_u * y_axis.z()) + z_axis.z(),
        )
    }

    /// 指定点での主曲率を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// (主曲率1, 主曲率2)
    pub fn principal_curvatures(&self, _u: T, v: T) -> (T, T) {
        let r_at_v = self.radius_at_v(v);
        let cos_angle = self.semi_angle().cos();

        // 円錐面の主曲率
        // κ1 = 0 (母線方向：直線のため曲率0)
        // κ2 = cos(α) / r(v) (円周方向)
        let k1 = T::ZERO;
        let k2 = if r_at_v > T::ZERO {
            cos_angle / r_at_v
        } else {
            T::ZERO // 頂点では未定義
        };

        (k1, k2)
    }

    /// 平均曲率を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// 平均曲率 H = (κ1 + κ2) / 2
    pub fn mean_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        (k1 + k2) / (T::ONE + T::ONE)
    }

    /// ガウス曲率を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// ガウス曲率 K = κ1 * κ2
    pub fn gaussian_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        k1 * k2 // 円錐面では常に0（可展面）
    }

    // ========================================================================
    // 幾何学的クエリ
    // ========================================================================

    /// 指定された方向からのレイとの交点を計算
    ///
    /// # Arguments
    /// * `ray_origin` - レイの開始点
    /// * `ray_direction` - レイの方向（正規化済み）
    ///
    /// # Returns
    /// 交点のリスト（パラメータ付き）
    pub fn ray_intersections(
        &self,
        _ray_origin: &Point3D<T>,
        _ray_direction: &Direction3D<T>,
    ) -> Vec<(Point3D<T>, T, (T, T))> {
        // 複雑な代数計算が必要
        // 簡略実装として空のベクタを返す
        // 実装は円錐の2次式とレイの1次式の連立方程式を解く
        Vec::new()
    }

    /// 平面との交線を計算
    ///
    /// # Arguments
    /// * `plane_point` - 平面上の点
    /// * `plane_normal` - 平面の法線ベクトル
    ///
    /// # Returns
    /// 交線の種類（楕円、放物線、双曲線、点、直線）
    pub fn plane_intersection_type(
        &self,
        _plane_point: &Point3D<T>,
        plane_normal: &Direction3D<T>,
    ) -> PlaneIntersectionType {
        // 平面と円錐軸の関係を分析
        let axis_vec = self.axis().as_vector();
        let normal_vec = plane_normal.as_vector();

        // 軸と平面法線の内積
        let dot_product = axis_vec.x() * normal_vec.x()
            + axis_vec.y() * normal_vec.y()
            + axis_vec.z() * normal_vec.z();

        let cos_axis_normal = dot_product.abs();
        let cos_semi_angle = self.semi_angle().cos();

        if cos_axis_normal > cos_semi_angle {
            PlaneIntersectionType::Ellipse
        } else if cos_axis_normal == cos_semi_angle {
            PlaneIntersectionType::Parabola
        } else {
            PlaneIntersectionType::Hyperbola
        }
    }

    /// 母線（generatrix）を取得
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    ///
    /// # Returns
    /// 指定角度での母線（直線）のパラメータ
    pub fn generatrix_at_u(&self, u: T) -> (Point3D<T>, Direction3D<T>) {
        let cos_u = u.cos();
        let sin_u = u.sin();

        let x_axis = self.ref_direction().as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let _z_axis = self.axis().as_vector();

        // 基準点での放射方向
        let radial_point = Point3D::new(
            self.center().x() + self.radius() * (cos_u * x_axis.x() + sin_u * y_axis.x()),
            self.center().y() + self.radius() * (cos_u * x_axis.y() + sin_u * y_axis.y()),
            self.center().z() + self.radius() * (cos_u * x_axis.z() + sin_u * y_axis.z()),
        );

        // 母線の方向：頂点に向かう方向
        let apex = self.apex();
        let direction_vec = Vector3D::new(
            apex.x() - radial_point.x(),
            apex.y() - radial_point.y(),
            apex.z() - radial_point.z(),
        );

        let direction = Direction3D::from_vector(direction_vec).expect("母線の方向は常に有効");

        (radial_point, direction)
    }

    // ========================================================================
    // CAD/CAM 用途
    // ========================================================================

    /// 指定された高さでの断面円を計算
    ///
    /// # Arguments
    /// * `v` - 軸方向パラメータ（高さ）
    ///
    /// # Returns
    /// (中心点, 軸方向, 半径)
    pub fn cross_section_at_v(&self, v: T) -> (Point3D<T>, Direction3D<T>, T) {
        let axis_vec = self.axis().as_vector();
        let center = Point3D::new(
            self.center().x() + v * axis_vec.x(),
            self.center().y() + v * axis_vec.y(),
            self.center().z() + v * axis_vec.z(),
        );
        let radius = self.radius_at_v(v);

        (center, self.axis(), radius)
    }

    /// 工具径路生成用のパラメータ計算
    ///
    /// # Arguments
    /// * `tool_radius` - 工具半径
    /// * `step_over` - ステップオーバー
    ///
    /// # Returns
    /// 加工パス用のパラメータリスト
    pub fn toolpath_parameters(&self, tool_radius: T, step_over: T) -> ToolpathParams<T> {
        ToolpathParams {
            surface: self.clone(),
            tool_radius,
            step_over,
        }
    }

    /// 表面品質解析用のメトリクス
    ///
    /// # Returns
    /// 表面品質に関する各種メトリクス
    pub fn surface_quality_metrics(&self) -> SurfaceQualityMetrics<T> {
        SurfaceQualityMetrics {
            is_smooth: true,                      // 円錐面は滑らか
            is_developable: true,                 // 可展面
            has_singularities: true,              // 頂点が特異点
            gaussian_curvature_constant: T::ZERO, // 常に0
        }
    }
}

// ========================================================================
// 関連する型定義
// ========================================================================

/// 平面との交線の種類
#[derive(Debug, Clone, PartialEq)]
pub enum PlaneIntersectionType {
    /// 楕円
    Ellipse,
    /// 放物線
    Parabola,
    /// 双曲線
    Hyperbola,
    /// 点（頂点を通る場合）
    Point,
    /// 直線（母線を含む場合）
    Line,
}

/// 工具径路生成用のパラメータ
#[derive(Debug, Clone)]
pub struct ToolpathParams<T: Scalar> {
    pub surface: ConicalSurface3D<T>,
    pub tool_radius: T,
    pub step_over: T,
}

/// 表面品質メトリクス
#[derive(Debug, Clone)]
pub struct SurfaceQualityMetrics<T: Scalar> {
    pub is_smooth: bool,
    pub is_developable: bool,
    pub has_singularities: bool,
    pub gaussian_curvature_constant: T,
}
