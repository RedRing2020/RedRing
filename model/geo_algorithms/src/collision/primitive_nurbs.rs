//! NURBS × Primitives 衝突判定実装
//!
//! geo_primitives と geo_nurbs 間の衝突判定を
//! geo_algorithms で集約して実装するモジュールです。
//!
//! ## Orphan Rules への対応
//!
//! 外部 trait を外部型へ直接実装できないため、
//! Newtype（NurbsCurveCollider）経由で BasicCollision を実装します。
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
//!   - intersection/primitive_nurbs.rs から直接呼び出し可能
//!   - collision 判定と intersection 判定（tolerance比較）で共有

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Ray3D, SphericalSolid3D,
};
use analysis::linalg::solver::newton::newton_solve_with_numeric_derivative_bounded;
use geo_contracts::{
    BasicCollision, Circle3DProperties, CylindricalSolid3DMeasure, InfiniteLine3DProperties,
    Plane3DProperties, Scalar,
};
use geo_core::Point3D;
use geo_nurbs::NurbsCurve3D;

// Newtype Wrapper for NurbsCurve3D

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
        point: &Point3D<T>,
        initial_u: T,
        u_min: T,
        u_max: T,
    ) -> T {
        let max_iter = 20;
        let tolerance = 1e-10_f64;
        let diff_step = 1e-7_f64;

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
            max_iter,
            tolerance,
            diff_step,
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

        let num_samples = 20; // サンプル数を削減（Newton法で精密化するため）
        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        let mut best_u = u_min;
        let mut min_dist_sq = T::INFINITY;

        // Phase 1: サンプリングで初期推定
        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            let dx = curve_point.x() - point.x();
            let dy = curve_point.y() - point.y();
            let dz = curve_point.z() - point.z();
            let dist_sq = dx * dx + dy * dy + dz * dz;

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
        let dx = closest_point.x() - point.x();
        let dy = closest_point.y() - point.y();
        let dz = closest_point.z() - point.z();
        (dx * dx + dy * dy + dz * dz).sqrt()
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
        let num_samples = 100;
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
            let dx = curve_point.x() - closest_x;
            let dy = curve_point.y() - closest_y;
            let dz = curve_point.z() - closest_z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

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
        let num_samples = 100;
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
            let dx = curve_point.x() - closest_x;
            let dy = curve_point.y() - closest_y;
            let dz = curve_point.z() - closest_z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

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
        let num_samples = 100;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        let (px, py, pz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::point(line);
        let point_on_line = Point3D::new(px, py, pz);
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
            let dx = curve_point.x() - closest_x;
            let dy = curve_point.y() - closest_y;
            let dz = curve_point.z() - closest_z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

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
        let num_samples = 100;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // Circle3Dの中心からの距離を計算し、半径を引く
            let (cx, cy, cz) = <Circle3D<T> as Circle3DProperties<T>>::center(circle);
            let center = Point3D::new(cx, cy, cz);
            let radius = <Circle3D<T> as Circle3DProperties<T>>::radius(circle);

            let dx = curve_point.x() - center.x();
            let dy = curve_point.y() - center.y();
            let dz = curve_point.z() - center.z();
            let dist_to_center = (dx * dx + dy * dy + dz * dz).sqrt();

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
        let num_samples = 100;
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
        let num_samples = 100;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_vec = self.0.evaluate_at(u);
            let curve_point = Point3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());

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
        let num_samples = 100;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_vec = self.0.evaluate_at(u);
            let curve_point = Point3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());

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
        let num_samples = 100;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_vec = self.0.evaluate_at(u);
            let curve_point = Point3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());

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

// ── Public Distance Functions (shared with intersection module) ──

/// NurbsCurve3D から Point3D への距離を計算
pub fn nurbscurve3d_point3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, point: &Point3D<T>) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(point)
}

/// NurbsCurve3D から LineSegment3D への距離を計算
pub fn nurbscurve3d_line_segment3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(segment)
}

/// NurbsCurve3D から Ray3D への距離を計算
pub fn nurbscurve3d_ray3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, ray: &Ray3D<T>) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(ray)
}

/// NurbsCurve3D から InfiniteLine3D への距離を計算
pub fn nurbscurve3d_infinite_line3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(line)
}

/// NurbsCurve3D から Circle3D への距離を計算
pub fn nurbscurve3d_circle3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    circle: &Circle3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(circle)
}

/// NurbsCurve3D から Plane3D への距離を計算
pub fn nurbscurve3d_plane3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, plane: &Plane3D<T>) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(plane)
}

/// NurbsCurve3D から SphericalSolid3D への距離を計算
pub fn nurbscurve3d_spherical_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(sphere)
}

/// NurbsCurve3D から EllipsoidalSolid3D への距離を計算
pub fn nurbscurve3d_ellipsoidal_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(ellipsoid)
}

/// NurbsCurve3D から CylindricalSolid3D への距離を計算
pub fn nurbscurve3d_cylindrical_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    cylinder: &CylindricalSolid3D<T>,
) -> T {
    NurbsCurveCollider::new(curve.clone()).distance_to(cylinder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::NurbsCurve3DConstructor;

    fn create_test_curve<T: Scalar>() -> NurbsCurve3D<T> {
        use analysis::linalg::vector::vector3::Vector3;

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

        // テスト用の NURBS 曲線を constructor trait で生成
        // シグネチャ: new(degree, knots, control_points: Vec<(T,T,T)>, weights)
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

    #[test]
    fn test_point_on_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(0.0, 0.0, 0.0); // 曲線の始点
        let tolerance = 1e-6;

        assert!(collider.intersects(&point, tolerance));
    }

    #[test]
    fn test_point_near_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(0.1, 0.0, 0.0); // 曲線の近く
        let distance = collider.distance_to(&point);

        assert!(distance < 0.2); // 適度に近い
        assert!(distance > 0.0); // 完全には一致しない
    }

    #[test]
    fn test_point_far_from_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(10.0, 10.0, 10.0); // 曲線から遠い
        let distance = collider.distance_to(&point);

        assert!(distance > 10.0);
    }

    #[test]
    fn test_symmetry() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(0.5, 0.5, 0.0);

        // Newtype パターンのため一方向のみテスト
        let distance = collider.distance_to(&point);
        assert!(distance >= 0.0); // 距離は非負
    }

    #[test]
    fn test_newtype_conversions() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve.clone());

        // inner() で参照を取得
        let _inner_ref = collider.inner();

        // into_inner() で元の NurbsCurve3D を取得
        let recovered_curve = collider.into_inner();

        // 元の曲線と同じパラメータ範囲を持つことを確認
        assert_eq!(recovered_curve.parameter_domain(), curve.parameter_domain());
    }

    // LineSegment3D tests

    #[test]
    fn test_line_segment_intersecting() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の始点を通る線分
        let segment =
            LineSegment3D::new(Point3D::new(-0.1, 0.0, 0.0), Point3D::new(0.1, 0.0, 0.0)).unwrap();

        let tolerance = 0.1;
        assert!(collider.intersects(&segment, tolerance));
    }

    #[test]
    fn test_line_segment_near() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の近くの線分
        let segment =
            LineSegment3D::new(Point3D::new(0.5, 0.5, 0.5), Point3D::new(1.5, 0.5, 0.5)).unwrap();

        let distance = collider.distance_to(&segment);
        assert!(distance > 0.0);
        assert!(distance < 1.0);
    }

    // Ray3D tests

    #[test]
    fn test_ray_intersecting() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の始点を通るRay
        let ray = Ray3D::new(
            Point3D::new(-1.0, 0.0, 0.0),
            geo_core::Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let tolerance = 0.1;
        assert!(collider.intersects(&ray, tolerance));
    }

    // InfiniteLine3D tests

    #[test]
    fn test_infinite_line_intersecting() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の始点を通る無限直線
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let tolerance = 0.1;
        assert!(collider.intersects(&line, tolerance));
    }

    // Circle3D tests

    #[test]
    fn test_circle_near() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の近くの円
        use crate::Direction3D;
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::from_vector(geo_core::Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            0.1,
        )
        .unwrap();

        let distance = collider.distance_to(&circle);
        assert!(distance >= 0.0);
    }

    // Plane3D tests

    #[test]
    fn test_plane_intersecting() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線が乗っているxy平面
        let plane = Plane3D::xy_plane(0.0);

        let tolerance = 1e-6;
        assert!(collider.intersects(&plane, tolerance));
    }

    #[test]
    fn test_plane_distance() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線から離れた平面（z = 1.0）
        let plane = Plane3D::xy_plane(1.0);

        let distance = collider.distance_to(&plane);
        assert!((distance - 1.0).abs() < 0.1); // 約1.0の距離
    }

    // SphericalSolid3D tests

    #[test]
    fn test_spherical_solid_intersecting() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の始点を含む球
        use geo_core::Vector3D;
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            0.5,
        )
        .unwrap();

        let tolerance = 1e-6;
        assert!(collider.intersects(&sphere, tolerance));
    }

    #[test]
    fn test_spherical_solid_separate() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線から離れた球
        use geo_core::Vector3D;
        let sphere = SphericalSolid3D::new(
            Point3D::new(10.0, 10.0, 10.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            0.5,
        )
        .unwrap();

        let distance = collider.distance_to(&sphere);
        assert!(distance > 10.0);
    }

    // EllipsoidalSolid3D tests

    #[test]
    fn test_ellipsoidal_solid_intersecting() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の始点を含む楕円体
        use geo_core::Vector3D;
        let ellipsoid = EllipsoidalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            0.5,
            0.5,
            0.5,
        )
        .unwrap();

        let tolerance = 1e-6;
        assert!(collider.intersects(&ellipsoid, tolerance));
    }

    // CylindricalSolid3D tests

    #[test]
    fn test_cylindrical_solid_near() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);

        // 曲線の近くの円柱
        use geo_core::Vector3D;
        let cylinder = CylindricalSolid3D::new(
            Point3D::new(0.5, 0.5, -1.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            0.1,
            2.0,
        )
        .unwrap();

        let distance = collider.distance_to(&cylinder);
        assert!(distance >= 0.0);
    }
}
