//! CylindricalSolid trait定義を capability taxonomy に沿って分離する。

use crate::PointClassification;
use analysis::abstract_types::Scalar;

pub trait CylindricalSolid3DConstructor<T: Scalar> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait CylindricalSolid3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn height(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn diameter(&self) -> T;
}

pub trait CylindricalSolid3DDerived<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
}

pub trait CylindricalSolid3DContainment<T: Scalar> {
    /// 点が立体の内部または表面上にあるか（表面を含む厳密な判定）
    fn contains_point(&self, point: (T, T, T)) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: (T, T, T)) -> PointClassification;
}

pub trait CylindricalSolid3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}
