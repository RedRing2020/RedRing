pub trait PointOps: Sized + Clone {
    fn origin() -> Self;
    fn add_scaled(&self, other: &Self, scale: f64) -> Self;
    fn div(&self, scalar: f64) -> Self;
}
