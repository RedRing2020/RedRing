use crate::{Point3D, Triangle3D};
use geo_contracts::{Scalar, Triangle3DBoundaryAccess};

pub(crate) fn triangle3d_vertex_points<T: Scalar>(triangle: &Triangle3D<T>) -> [Point3D<T>; 3] {
    let triangle_vertex_a = Triangle3DBoundaryAccess::vertex_a(triangle);
    let (ax, ay, az) = (
        triangle_vertex_a.x(),
        triangle_vertex_a.y(),
        triangle_vertex_a.z(),
    );
    let triangle_vertex_b = Triangle3DBoundaryAccess::vertex_b(triangle);
    let (bx, by, bz) = (
        triangle_vertex_b.x(),
        triangle_vertex_b.y(),
        triangle_vertex_b.z(),
    );
    let triangle_vertex_c = Triangle3DBoundaryAccess::vertex_c(triangle);
    let (cx, cy, cz) = (
        triangle_vertex_c.x(),
        triangle_vertex_c.y(),
        triangle_vertex_c.z(),
    );
    [
        Point3D::new(ax, ay, az),
        Point3D::new(bx, by, bz),
        Point3D::new(cx, cy, cz),
    ]
}
