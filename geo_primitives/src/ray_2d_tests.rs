//! Ray2D test file - Basic test only
use crate::{Point2D, Ray2D, Vector2D};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let origin = Point2D::new(1.0, 2.0);
        let direction = Vector2D::new(3.0, 4.0);
        let ray = Ray2D::new(origin, direction).unwrap();
        assert_eq!(ray.origin(), origin);
    }
}
