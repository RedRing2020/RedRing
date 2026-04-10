use crate::{Point3D, Triangle3D};
use geo_contracts::{Scalar, Triangle3DBoundaryAccess};

pub(crate) fn triangle3d_vertex_points<T: Scalar>(triangle: &Triangle3D<T>) -> [Point3D<T>; 3] {
    let (ax, ay, az) = Triangle3DBoundaryAccess::vertex_a(triangle);
    let (bx, by, bz) = Triangle3DBoundaryAccess::vertex_b(triangle);
    let (cx, cy, cz) = Triangle3DBoundaryAccess::vertex_c(triangle);
    [
        Point3D::new(ax, ay, az),
        Point3D::new(bx, by, bz),
        Point3D::new(cx, cy, cz),
    ]
}
