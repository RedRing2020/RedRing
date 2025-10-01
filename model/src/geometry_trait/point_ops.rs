pub trait PointOps: Sized + Clone {
    fn origin() -> Self;
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn mul(&self, scalar: f64) -> Self;
    fn div(&self, scalar: f64) -> Self;
    fn add_scaled(&self, other: &Self, scale: f64) -> Self;
}
