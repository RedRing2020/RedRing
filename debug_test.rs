//! 一時的なデバッグテスト

#[cfg(test)]
mod debug_tests {
    use geo_foundation::Point2D;

    #[test]
    fn debug_point_operations() {
        let p1 = Point2D::<f64>::new(1.0, 2.0);
        let p2 = Point2D::<f64>::new(3.0, 4.0);

        println!("p1: ({}, {})", p1.x(), p1.y());
        println!("p2: ({}, {})", p2.x(), p2.y());

        let diff = p2 - p1;
        println!("p2 - p1 = ({}, {})", diff.x(), diff.y());

        let vector_to = p1.vector_to(&p2);
        println!("p1.vector_to(p2) = ({}, {})", vector_to.x(), vector_to.y());
    }
}
