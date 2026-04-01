//! NURBS3D × Primitives 衝突判定実装
//!
//! geo_primitives と geo_nurbs 間の衝突判定を
//! geo_algorithms で集約して実装するモジュールです。
//!
//! NurbsCurve3D / NurbsSurface3D と各種3D Primitive の衝突判定を統一します。
//!
//! ## Orphan Rules への対応
//!
//! 外部 trait を外部型へ直接実装できないため、
//! Newtype（NurbsCurveCollider / NurbsSurfaceCollider）経由で BasicCollision を実装します。
//!
//! ## 設計方針  
//!
//! 1. サンプリング + 数値最適化で距離を評価
//! 2. 形状ペアごとに BasicCollision を実装
//! 3. `#[repr(transparent)]` による軽量ラップを維持
//!
//! ## Public Interface (intersection側で使用)
//!
//! - `nurbscurve3d_XXX_distance`: NurbsCurve3D と各Primitive間の距離計算
//! - `nurbssurface3d_XXX_distance`: NurbsSurface3D と各Primitive間の距離計算
//!   - intersection/nurbs_3d.rs から直接呼び出し可能
//!   - collision 判定と intersection 判定（tolerance比較）で共有

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, SphericalSolid3D,
};
use analysis::linalg::solver::newton::newton_solve_with_numeric_derivative_bounded;
use geo_contracts::{
    BasicCollision, Circle3DProperties, CylindricalSolid3DMeasure, InfiniteLine3DProperties,
    Plane3DProperties, Scalar,
};
use geo_core::Point3D as CorePoint3D;
use geo_nurbs::{constants, NurbsCurve3D, NurbsSurface3D};

const NURBS_CURVE_POINT_BOOTSTRAP_SAMPLES: usize = 20;
const NURBS_CURVE_DISTANCE_SAMPLES: usize = 100;
const NURBS_SURFACE_DISTANCE_SAMPLES_U: usize = 24;
const NURBS_SURFACE_DISTANCE_SAMPLES_V: usize = 24;

// ─────────────────────────────────────────────────────────────────────────
// NurbsCurveCollider Newtype Wrapper
// ─────────────────────────────────────────────────────────────────────────

/// NURBS曲線の衝突判定アダプタ（Newtype パターン）
///
/// orphan rules を回避するため、NurbsCurve3D をラップした型を提供します。
/// この型は BasicCollision トレイトを実装でき、ポリモーフィズムを維持できます。
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NurbsCurveCollider<T: Scalar>(pub NurbsCurve3D<T>);

impl<T: Scalar> NurbsCurveCollider<T> {
    /// NurbsCurve3D からアダプタを作成
    pub fn new(curve: NurbsCurve3D<T>) -> Self {
        Self(curve)
    }

    /// 内部の NurbsCurve3D への参照を取得
    pub fn inner(&self) -> &NurbsCurve3D<T> {
        &self.0
    }

    /// NurbsCurve3D を消費してアダプタから取り出す
    pub fn into_inner(self) -> NurbsCurve3D<T> {
        self.0
    }

    /// Newton法により点への最近接パラメータを精密化（public）
    ///
    /// 目的関数: f(u) = (C(u) - P) · C'(u) = 0
    /// C(u)が点Pに最も近いとき、C(u)-P は C'(u) に直交する
    pub fn newton_refine_closest_point(
        &self,
        point: &CorePoint3D<T>,
        initial_u: T,
        u_min: T,
        u_max: T,
    ) -> T {
        let objective = |u: f64| {
            let u_t = T::from_f64(u);
            let c = self.0.evaluate_at(u_t);
            let dc = self.0.derivative_at(u_t);

            let diff_x = c.x() - point.x();
            let diff_y = c.y() - point.y();
            let diff_z = c.z() - point.z();

            let value_t = diff_x * dc.x() + diff_y * dc.y() + diff_z * dc.z();
            value_t.to_f64()
        };

        let maybe_u = newton_solve_with_numeric_derivative_bounded(
            objective,
            initial_u.to_f64(),
            u_min.to_f64(),
            u_max.to_f64(),
            constants::NEWTON_MAX_ITER,
            constants::NEWTON_TOLERANCE,
            constants::NEWTON_DIFF_STEP,
        );

        maybe_u
            .map(T::from_f64)
            .unwrap_or_else(|| initial_u.clamp(u_min, u_max))
    }
}

// NurbsCurveCollider vs Point3D

impl<T: Scalar> BasicCollision<T, Point3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 2段階アプローチ: サンプリング + Newton法による精密化
        // 1. サンプリングで初期推定値を見つける
        // 2. Newton法で最近接点を精密計算

        let num_samples = NURBS_CURVE_POINT_BOOTSTRAP_SAMPLES; // サンプル数を削減（Newton法で精密化するため）
        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        let mut best_u = u_min;
        let mut min_dist_sq = T::INFINITY;

        // Phase 1: サンプリングで初期推定
        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);
            let sample_point = CorePoint3D::new(curve_point.x(), curve_point.y(), curve_point.z());
            let dist_sq = sample_point.distance_squared_to(point);

            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_u = u;
            }
        }

        // Phase 2: Newton法で精密化
        // 目的関数: f(u) = (C(u) - P) · C'(u) = 0
        // C(u)が点Pに最も近いとき、C(u)-P が C'(u) に直交する
        let refined_u = self.newton_refine_closest_point(point, best_u, u_min, u_max);

        // 最終的な距離を計算
        let closest_point = self.0.evaluate_at(refined_u);
        let closest_point =
            CorePoint3D::new(closest_point.x(), closest_point.y(), closest_point.z());
        closest_point.distance_to(point)
    }
}

// NurbsCurveCollider vs LineSegment3D

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(segment) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        // 2段階アプローチ: NURBS曲線と線分の両方をサンプリング
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        // NURBS曲線上の各サンプル点から線分への最短距離を計算
        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // 線分上の最近接点を計算（線分のパラメトリック表現を使用）
            let start = segment.start();
            let end = segment.end();

            // 線分の方向ベクトル
            let seg_dir_x = end.x() - start.x();
            let seg_dir_y = end.y() - start.y();
            let seg_dir_z = end.z() - start.z();

            // 曲線点から線分始点へのベクトル
            let to_point_x = curve_point.x() - start.x();
            let to_point_y = curve_point.y() - start.y();
            let to_point_z = curve_point.z() - start.z();

            // 線分上のパラメータ t を計算（投影）
            let seg_len_sq = seg_dir_x * seg_dir_x + seg_dir_y * seg_dir_y + seg_dir_z * seg_dir_z;
            let t = if seg_len_sq.is_zero() {
                T::ZERO
            } else {
                let dot = to_point_x * seg_dir_x + to_point_y * seg_dir_y + to_point_z * seg_dir_z;
                (dot / seg_len_sq).clamp(T::ZERO, T::ONE)
            };

            // 線分上の最近接点
            let closest_x = start.x() + t * seg_dir_x;
            let closest_y = start.y() + t * seg_dir_y;
            let closest_z = start.z() + t * seg_dir_z;

            // 距離を計算
            let sample_point = CorePoint3D::new(curve_point.x(), curve_point.y(), curve_point.z());
            let closest_point = CorePoint3D::new(closest_x, closest_y, closest_z);
            let distance = sample_point.distance_to(&closest_point);

            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs Ray3D

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点からRayへの距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        let origin = ray.origin();
        let direction_vec = ray.direction_vector();

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // 点からRayへの距離（Ray上の最近接点を計算）
            let to_point_x = curve_point.x() - origin.x();
            let to_point_y = curve_point.y() - origin.y();
            let to_point_z = curve_point.z() - origin.z();

            // Rayの方向への投影
            let t = (to_point_x * direction_vec.x()
                + to_point_y * direction_vec.y()
                + to_point_z * direction_vec.z())
            .max(T::ZERO); // Rayは一方向のみ

            // Ray上の最近接点
            let closest_x = origin.x() + t * direction_vec.x();
            let closest_y = origin.y() + t * direction_vec.y();
            let closest_z = origin.z() + t * direction_vec.z();

            // 距離を計算
            let sample_point = CorePoint3D::new(curve_point.x(), curve_point.y(), curve_point.z());
            let closest_point = CorePoint3D::new(closest_x, closest_y, closest_z);
            let distance = sample_point.distance_to(&closest_point);

            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs InfiniteLine3D

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点から無限直線への距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        let (px, py, pz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::point(line);
        let point_on_line = CorePoint3D::new(px, py, pz);
        let (dx, dy, dz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::direction(line);
        let direction_vec = geo_core::Vector3D::new(dx, dy, dz);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // 点から無限直線への距離
            let to_point_x = curve_point.x() - point_on_line.x();
            let to_point_y = curve_point.y() - point_on_line.y();
            let to_point_z = curve_point.z() - point_on_line.z();

            // 直線の方向への投影
            let t = to_point_x * direction_vec.x()
                + to_point_y * direction_vec.y()
                + to_point_z * direction_vec.z();

            // 直線上の最近接点
            let closest_x = point_on_line.x() + t * direction_vec.x();
            let closest_y = point_on_line.y() + t * direction_vec.y();
            let closest_z = point_on_line.z() + t * direction_vec.z();

            // 距離を計算
            let sample_point = CorePoint3D::new(curve_point.x(), curve_point.y(), curve_point.z());
            let closest_point = CorePoint3D::new(closest_x, closest_y, closest_z);
            let distance = sample_point.distance_to(&closest_point);

            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs Circle3D

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点からCircle3Dへの距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // Circle3Dの中心からの距離を計算し、半径を引く
            let (cx, cy, cz) = <Circle3D<T> as Circle3DProperties<T>>::center(circle);
            let center = CorePoint3D::new(cx, cy, cz);
            let radius = <Circle3D<T> as Circle3DProperties<T>>::radius(circle);

            let sample_point = CorePoint3D::new(curve_point.x(), curve_point.y(), curve_point.z());
            let dist_to_center = sample_point.distance_to(&center);

            // 円周までの距離
            let distance = (dist_to_center - radius).abs();

            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs Plane3D

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点から平面への距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        let point_on_plane = plane.origin();
        let (nx, ny, nz) = <Plane3D<T> as Plane3DProperties<T>>::normal(plane);
        let normal_vec = geo_core::Vector3D::new(nx, ny, nz);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // 点から平面への符号付き距離
            let to_point_x = curve_point.x() - point_on_plane.x();
            let to_point_y = curve_point.y() - point_on_plane.y();
            let to_point_z = curve_point.z() - point_on_plane.z();

            let distance = (to_point_x * normal_vec.x()
                + to_point_y * normal_vec.y()
                + to_point_z * normal_vec.z())
            .abs();

            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs SphericalSolid3D

impl<T: Scalar> BasicCollision<T, SphericalSolid3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, sphere: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(sphere) <= tolerance
    }

    fn overlaps(&self, sphere: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(sphere, tolerance)
    }

    fn distance_to(&self, sphere: &SphericalSolid3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点から球への距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_vec = self.0.evaluate_at(u);
            let curve_point = CorePoint3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());

            let distance = sphere.distance_to_surface(curve_point);
            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs EllipsoidalSolid3D

impl<T: Scalar> BasicCollision<T, EllipsoidalSolid3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ellipsoid: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(ellipsoid) <= tolerance
    }

    fn overlaps(&self, ellipsoid: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(ellipsoid, tolerance)
    }

    fn distance_to(&self, ellipsoid: &EllipsoidalSolid3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点から楕円体への距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_vec = self.0.evaluate_at(u);
            let curve_point = CorePoint3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());

            let distance = if ellipsoid.contains_point(&curve_point) {
                T::ZERO
            } else {
                ellipsoid.distance_to_surface(&curve_point)
            };
            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// NurbsCurveCollider vs CylindricalSolid3D

impl<T: Scalar> BasicCollision<T, CylindricalSolid3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, cylinder: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(cylinder) <= tolerance
    }

    fn overlaps(&self, cylinder: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(cylinder, tolerance)
    }

    fn distance_to(&self, cylinder: &CylindricalSolid3D<T>) -> T {
        // NURBS曲線をサンプリングして、各点から円柱への距離を計算
        let num_samples = NURBS_CURVE_DISTANCE_SAMPLES;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_vec = self.0.evaluate_at(u);
            let curve_point = CorePoint3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());

            // 円柱の distance_to_point メソッドを使用
            let distance =
                <CylindricalSolid3D<T> as CylindricalSolid3DMeasure<T>>::distance_to_point(
                    cylinder,
                    (curve_point.x(), curve_point.y(), curve_point.z()),
                );
            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

// ─────────────────────────────────────────────────────────────────────────
// NurbsSurfaceCollider Newtype Wrapper
// ─────────────────────────────────────────────────────────────────────────

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NurbsSurfaceCollider<T: Scalar>(pub NurbsSurface3D<T>);

impl<T: Scalar> NurbsSurfaceCollider<T> {
    pub fn new(surface: NurbsSurface3D<T>) -> Self {
        Self(surface)
    }

    fn nearest_sample_to_point(&self, point: &Point3D<T>) -> (Point3D<T>, T) {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut best_point = Point3D::new(T::ZERO, T::ZERO, T::ZERO);
        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);
                let surface_point = Point3D::new(p.x(), p.y(), p.z());

                let d = surface_point.distance_to(point);

                if d < min_dist {
                    min_dist = d;
                    best_point = surface_point;
                }
            }
        }

        (best_point, min_dist)
    }

    fn min_distance_to_plane(&self, plane: &Plane3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let origin = plane.origin();
        let (nx, ny, nz) = <Plane3D<T> as Plane3DProperties<T>>::normal(plane);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let dx = p.x() - origin.x();
                let dy = p.y() - origin.y();
                let dz = p.z() - origin.z();
                let d = (dx * nx + dy * ny + dz * nz).abs();
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_ray(&self, ray: &Ray3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let origin = ray.origin();
        let direction = ray.direction_vector();

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let to_px = p.x() - origin.x();
                let to_py = p.y() - origin.y();
                let to_pz = p.z() - origin.z();

                let t = (to_px * direction.x() + to_py * direction.y() + to_pz * direction.z())
                    .max(T::ZERO);

                let closest_x = origin.x() + t * direction.x();
                let closest_y = origin.y() + t * direction.y();
                let closest_z = origin.z() + t * direction.z();

                let surface_point = Point3D::new(p.x(), p.y(), p.z());
                let closest_point = Point3D::new(closest_x, closest_y, closest_z);
                let d = surface_point.distance_to(&closest_point);

                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_line_segment(&self, segment: &LineSegment3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let start = segment.start();
        let end = segment.end();
        let seg_dx = end.x() - start.x();
        let seg_dy = end.y() - start.y();
        let seg_dz = end.z() - start.z();
        let seg_len_sq = start.distance_squared_to(&end);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let to_px = p.x() - start.x();
                let to_py = p.y() - start.y();
                let to_pz = p.z() - start.z();

                let t = if seg_len_sq.is_zero() {
                    T::ZERO
                } else {
                    ((to_px * seg_dx + to_py * seg_dy + to_pz * seg_dz) / seg_len_sq)
                        .clamp(T::ZERO, T::ONE)
                };

                let cx = start.x() + t * seg_dx;
                let cy = start.y() + t * seg_dy;
                let cz = start.z() + t * seg_dz;

                let surface_point = Point3D::new(p.x(), p.y(), p.z());
                let closest_point = Point3D::new(cx, cy, cz);
                let d = surface_point.distance_to(&closest_point);
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_infinite_line(&self, line: &InfiniteLine3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let (px, py, pz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::point(line);
        let (dx, dy, dz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::direction(line);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let tx = p.x() - px;
                let ty = p.y() - py;
                let tz = p.z() - pz;
                let t = tx * dx + ty * dy + tz * dz;

                let cx = px + t * dx;
                let cy = py + t * dy;
                let cz = pz + t * dz;

                let surface_point = Point3D::new(p.x(), p.y(), p.z());
                let closest_point = Point3D::new(cx, cy, cz);
                let d = surface_point.distance_to(&closest_point);
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_circle(&self, circle: &Circle3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let (cx, cy, cz) = <Circle3D<T> as Circle3DProperties<T>>::center(circle);
        let radius = <Circle3D<T> as Circle3DProperties<T>>::radius(circle);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let surface_point = Point3D::new(p.x(), p.y(), p.z());
                let center = Point3D::new(cx, cy, cz);
                let distance_to_center = surface_point.distance_to(&center);
                let d = (distance_to_center - radius).abs();
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_spherical_solid(&self, sphere: &SphericalSolid3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);
                let surface_point = CorePoint3D::new(p.x(), p.y(), p.z());
                min_dist = min_dist.min(sphere.distance_to_surface(surface_point));
            }
        }

        min_dist
    }

    fn min_distance_to_ellipsoidal_solid(&self, ellipsoid: &EllipsoidalSolid3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);
                let surface_point = CorePoint3D::new(p.x(), p.y(), p.z());

                let d = if ellipsoid.contains_point(&surface_point) {
                    T::ZERO
                } else {
                    ellipsoid.distance_to_surface(&surface_point)
                };
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_cylindrical_solid(&self, cylinder: &CylindricalSolid3D<T>) -> T {
        let samples_u = NURBS_SURFACE_DISTANCE_SAMPLES_U;
        let samples_v = NURBS_SURFACE_DISTANCE_SAMPLES_V;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let d = <CylindricalSolid3D<T> as CylindricalSolid3DMeasure<T>>::distance_to_point(
                    cylinder,
                    (p.x(), p.y(), p.z()),
                );
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }
}

impl<T: Scalar> BasicCollision<T, Point3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.nearest_sample_to_point(point).1
    }
}

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        self.min_distance_to_plane(plane)
    }
}

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.min_distance_to_ray(ray)
    }
}

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(segment) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        self.min_distance_to_line_segment(segment)
    }
}

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.min_distance_to_infinite_line(line)
    }
}

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        self.min_distance_to_circle(circle)
    }
}

impl<T: Scalar> BasicCollision<T, SphericalSolid3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, sphere: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(sphere) <= tolerance
    }

    fn overlaps(&self, sphere: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(sphere, tolerance)
    }

    fn distance_to(&self, sphere: &SphericalSolid3D<T>) -> T {
        self.min_distance_to_spherical_solid(sphere)
    }
}

impl<T: Scalar> BasicCollision<T, EllipsoidalSolid3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ellipsoid: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(ellipsoid) <= tolerance
    }

    fn overlaps(&self, ellipsoid: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(ellipsoid, tolerance)
    }

    fn distance_to(&self, ellipsoid: &EllipsoidalSolid3D<T>) -> T {
        self.min_distance_to_ellipsoidal_solid(ellipsoid)
    }
}

impl<T: Scalar> BasicCollision<T, CylindricalSolid3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, cylinder: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(cylinder) <= tolerance
    }

    fn overlaps(&self, cylinder: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(cylinder, tolerance)
    }

    fn distance_to(&self, cylinder: &CylindricalSolid3D<T>) -> T {
        self.min_distance_to_cylindrical_solid(cylinder)
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Public Distance Functions (shared with intersection module)
// ─────────────────────────────────────────────────────────────────────────

// NurbsCurve3D distance functions

pub fn nurbscurve3d_point3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, point: &Point3D<T>) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(point)
}

pub fn nurbscurve3d_line_segment3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(segment)
}

pub fn nurbscurve3d_ray3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, ray: &Ray3D<T>) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(ray)
}

pub fn nurbscurve3d_infinite_line3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(line)
}

pub fn nurbscurve3d_circle3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    circle: &Circle3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(circle)
}

pub fn nurbscurve3d_plane3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, plane: &Plane3D<T>) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(plane)
}

pub fn nurbscurve3d_spherical_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(sphere)
}

pub fn nurbscurve3d_ellipsoidal_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(ellipsoid)
}

pub fn nurbscurve3d_cylindrical_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    cylinder: &CylindricalSolid3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(cylinder)
}

// NurbsSurface3D distance functions

pub fn nurbssurface3d_point3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(point)
}

pub fn nurbssurface3d_plane3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    plane: &Plane3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(plane)
}

pub fn nurbssurface3d_ray3d_distance<T: Scalar>(surface: &NurbsSurface3D<T>, ray: &Ray3D<T>) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(ray)
}

pub fn nurbssurface3d_line_segment3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(segment)
}

pub fn nurbssurface3d_infinite_line3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(line)
}

pub fn nurbssurface3d_circle3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    circle: &Circle3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(circle)
}

pub fn nurbssurface3d_spherical_solid3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(sphere)
}

pub fn nurbssurface3d_ellipsoidal_solid3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(ellipsoid)
}

pub fn nurbssurface3d_cylindrical_solid3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    cylinder: &CylindricalSolid3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(cylinder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3D;
    use geo_contracts::NurbsSurface3DConstructor;
    use geo_nurbs::NurbsCurve3D;

    const TEST_TOLERANCE: f64 = 1e-6;
    const TEST_FAR_POINT_COORD: f64 = 10.0;
    const TEST_MIN_FAR_DISTANCE: f64 = 10.0;

    fn create_test_curve<T: Scalar>() -> NurbsCurve3D<T> {
        use analysis::linalg::vector::vector3::Vector3;
        use geo_contracts::NurbsCurve3DConstructor;

        // 3次 NURBS 曲線（制御点4つ、次数3）
        let control_points = vec![
            Vector3::new(T::ZERO, T::ZERO, T::ZERO),
            Vector3::new(T::ONE, T::ZERO, T::ZERO),
            Vector3::new(T::ONE, T::ONE, T::ZERO),
            Vector3::new(T::ZERO, T::ONE, T::ZERO),
        ];

        let weights = Some(vec![T::ONE, T::ONE, T::ONE, T::ONE]);
        let knots = vec![
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ONE,
            T::ONE,
            T::ONE,
        ];

        let control_points_tuples = control_points
            .into_iter()
            .map(|v| (v.x(), v.y(), v.z()))
            .collect();
        <NurbsCurve3D<T> as NurbsCurve3DConstructor<T>>::new(
            3,
            knots,
            control_points_tuples,
            weights,
        )
        .unwrap()
    }

    fn create_test_surface<T: Scalar>() -> NurbsSurface3D<T> {
        <NurbsSurface3D<T> as NurbsSurface3DConstructor<T>>::unit_plane()
    }

    #[test]
    fn test_nurbscurve3d_point3d_distance_zero_on_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = CorePoint3D::new(0.0, 0.0, 0.0);
        let tolerance = TEST_TOLERANCE;

        assert!(collider.intersects(&point, tolerance));
    }

    #[test]
    fn test_nurbscurve3d_point3d_distance_far() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = CorePoint3D::new(
            TEST_FAR_POINT_COORD,
            TEST_FAR_POINT_COORD,
            TEST_FAR_POINT_COORD,
        );
        let distance = collider.distance_to(&point);

        assert!(distance > TEST_MIN_FAR_DISTANCE);
    }

    #[test]
    fn test_nurbssurface3d_point3d_distance_zero_on_surface() {
        let surface = create_test_surface::<f64>();
        let point = Point3D::new(0.5, 0.5, 0.0);
        let d = nurbssurface3d_point3d_distance(&surface, &point);
        assert!(d <= TEST_TOLERANCE);
    }

    #[test]
    fn test_nurbssurface3d_plane3d_distance_zero_for_xy_plane() {
        let surface = create_test_surface::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        let d = nurbssurface3d_plane3d_distance(&surface, &plane);
        assert!(d <= TEST_TOLERANCE);
    }

    #[test]
    fn test_nurbssurface3d_ray3d_distance_zero_for_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let ray = Ray3D::new(
            CorePoint3D::new(0.5, 0.5, -1.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let d = nurbssurface3d_ray3d_distance(&surface, &ray);
        assert!(d <= TEST_TOLERANCE);
    }
}
