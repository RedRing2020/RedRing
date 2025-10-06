use geo_primitives::f64geom::*;

#[test]
fn vector_ops() {
    let a = FVector3::new(1.0,2.0,3.0);
    let b = FVector3::new(-2.0,0.5,4.0);
    let c = a + b; // ( -1, 2.5, 7 )
    assert!((c.x() +1.0).abs() < 1e-12);
    assert!((c.y() -2.5).abs() < 1e-12);
    assert!((c.z() -7.0).abs() < 1e-12);
    let cross = a.cross(&b);
    // 手計算 cross(a,b)
    let expected = FVector3::new( (2.0*4.0 - 3.0*0.5), (3.0*-2.0 - 1.0*4.0), (1.0*0.5 - 2.0*-2.0) );
    assert!((cross.x()-expected.x()).abs()<1e-12);
    assert!((cross.y()-expected.y()).abs()<1e-12);
    assert!((cross.z()-expected.z()).abs()<1e-12);
    assert!(a.normalize().is_some());
}

#[test]
fn point_vector_arith() {
    let p = FPoint3::new(1.0,2.0,3.0);
    let v = FVector3::new(-1.0,0.5,2.0);
    let q = p + v;
    assert!((q.x() - 0.0).abs()<1e-12);
    assert!((q.y() - 2.5).abs()<1e-12);
    assert!((q.z() - 5.0).abs()<1e-12);
    let back = q - v;
    assert!((back.x()-p.x()).abs()<1e-12);
    let pv = q - p;
    assert!((pv.x() - (-1.0)).abs()<1e-12);
}

#[test]
fn direction_basis() {
    let d = FDirection3::new(0.0,0.0,1.0).unwrap();
    let (u,v) = d.orthonormal_basis();
    // u,v should be orthogonal and unit length (within tolerance)
    let dot = u.dot(&v);
    assert!(dot.abs()<1e-12);
    assert!((u.norm()-1.0).abs()<1e-12);
    assert!((v.norm()-1.0).abs()<1e-12);
}

#[test]
fn line_segment_props() {
    let p0 = FPoint3::new(0.0,0.0,0.0);
    let p1 = FPoint3::new(0.0,3.0,4.0);
    let seg = FLineSegment3::new(p0,p1);
    assert!((seg.length()-5.0).abs()<1e-12);
    let dir = seg.direction().unwrap();
    assert!((dir.norm()-1.0).abs()<1e-12);
    let mid = seg.midpoint();
    assert!((mid.y()-1.5).abs()<1e-12);
}

#[test]
fn plane_projection() {
    let origin = FPoint3::new(0.0,0.0,0.0);
    let n = FDirection3::new(0.0,1.0,0.0).unwrap();
    let plane = FPlane::new(origin,n);
    let p = FPoint3::new(1.0, 2.0, 3.0);
    let proj = plane.project_point(&p);
    assert!((proj.y()).abs()<1e-12);
    assert!((proj.x()-1.0).abs()<1e-12);
}

#[test]
fn circle_point_at() {
    let center = FPoint3::new(0.0,0.0,0.0);
    let n = FDirection3::new(0.0,0.0,1.0).unwrap();
    let circle = FCircle3::new(center,n,2.0).unwrap();
    let p = circle.point_at(std::f64::consts::FRAC_PI_2); // 90 deg
    assert!((p.x()).abs()<1e-12);
    assert!((p.y()-2.0).abs()<1e-12);
    assert!((p.z()).abs()<1e-12);
}
