//! TriangleMesh3D のテスト

use crate::{Point3D, TriangleMesh3D};
use geo_contracts::{
    Bounded, PrimitiveKind, PrimitiveMetadata, TolerantEq, Triangle3DBoundaryAccess,
    TriangleMesh3DConstructor, TriangleMesh3DProperties, TriangleMesh3DTriangleAccess,
    TriangleMeshError,
};

fn empty_mesh() -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(Vec::new(), Vec::new()).unwrap()
}

#[test]
fn test_mesh_creation() {
    // 三角形1つのメッシュ
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices = vec![[0, 1, 2]];

    let mesh = TriangleMesh3D::new(vertices, indices);
    assert!(mesh.is_ok());

    let mesh = mesh.unwrap();
    assert_eq!(mesh.vertex_count(), 3);
    assert_eq!(mesh.triangle_count(), 1);
    assert!(mesh.is_valid());
}

#[test]
fn test_invalid_indices() {
    let vertices = vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)];
    // 無効なインデックス（3つの頂点しかないのに3を参照）
    let indices = vec![[0, 1, 3]];

    // 範囲外の頂点番号を報告する
    assert_eq!(
        TriangleMesh3D::new(vertices, indices),
        Err(TriangleMeshError::InvalidVertexIndex {
            triangle: 0,
            corner: 2,
            index: 3,
            vertex_count: 2,
        })
    );
}

#[test]
fn test_empty_mesh() {
    let mesh = empty_mesh();
    assert_eq!(mesh.vertex_count(), 0);
    assert_eq!(mesh.triangle_count(), 0);
    assert!(mesh.is_valid());
    assert!(mesh.is_empty());
}

#[test]
fn test_mesh_access() {
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
        Point3D::new(1.0, 1.0, 0.0),
    ];
    let indices = vec![[0, 1, 2], [1, 3, 2]];

    let mesh = TriangleMesh3D::new(vertices.clone(), indices.clone()).unwrap();

    // 配列アクセス
    assert_eq!(mesh.vertices(), &vertices);
    assert_eq!(mesh.indices(), &indices);
}

#[test]
fn test_triangle_extraction() {
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices = vec![[0, 1, 2]];

    let mesh = TriangleMesh3D::new(vertices.clone(), indices).unwrap();
    let triangle = mesh.triangle(0);

    assert!(triangle.is_some());
    let triangle = triangle.unwrap();
    assert_eq!(triangle.vertex_a(), vertices[0]);
    assert_eq!(triangle.vertex_b(), vertices[1]);
    assert_eq!(triangle.vertex_c(), vertices[2]);

    // 範囲外の番号
    assert!(mesh.triangle(1).is_none());
}

#[test]
fn test_bounding_box() {
    let vertices = vec![
        Point3D::new(-1.0, -2.0, -3.0),
        Point3D::new(2.0, 1.0, 0.0),
        Point3D::new(0.0, 3.0, 1.0),
    ];
    let indices = vec![[0, 1, 2]];

    let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
    let bbox = mesh.aabb().unwrap();

    assert_eq!(bbox.min(), Point3D::new(-1.0, -2.0, -3.0));
    assert_eq!(bbox.max(), Point3D::new(2.0, 3.0, 1.0));
}

#[test]
fn test_empty_bounding_box() {
    assert!(empty_mesh().aabb().is_none());
}

#[test]
fn test_quad_mesh() {
    // 四角形を2つの三角形で構成
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0), // 0
        Point3D::new(1.0, 0.0, 0.0), // 1
        Point3D::new(1.0, 1.0, 0.0), // 2
        Point3D::new(0.0, 1.0, 0.0), // 3
    ];
    let indices = vec![
        [0, 1, 2], // 下三角形
        [0, 2, 3], // 上三角形
    ];

    let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
    assert_eq!(mesh.vertex_count(), 4);
    assert_eq!(mesh.triangle_count(), 2);
    assert!(mesh.is_valid());
}

#[test]
fn test_mesh_display() {
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices = vec![[0, 1, 2]];

    let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
    let display_str = format!("{}", mesh);

    assert!(display_str.contains("TriangleMesh3D"));
    assert!(display_str.contains("3 vertices"));
    assert!(display_str.contains("1 triangles"));
}

#[test]
fn test_tolerant_eq() {
    let vertices1 = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices1 = vec![[0, 1, 2]];
    let vertices2 = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices2 = vec![[0, 1, 2]];

    let mesh1 = TriangleMesh3D::new(vertices1, indices1).unwrap();
    let mesh2 = TriangleMesh3D::new(vertices2, indices2).unwrap();

    assert!(mesh1.tolerant_eq(&mesh2, 0.01));
}

#[test]
fn test_normals_stay_unit_after_similarity_transform() {
    use crate::{Direction3D, SimilarityTransform3D, Vector3D};
    use geo_contracts::{Angle, SimilarityTransformable3D};

    // 回転と一様スケールの後も、法線は回転した単位ベクトルのまま
    let transform = SimilarityTransform3D::rotation_about_axis(
        Point3D::origin(),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Angle::from_degrees(90.0),
    )
    .then(&SimilarityTransform3D::uniform_scale_about(Point3D::origin(), 2.0).unwrap());
    let mesh = TriangleMesh3D {
        vertices: vec![Point3D::new(0.0, 0.0, 0.0)],
        indices: Vec::new(),
        normals: Some(vec![Vector3D::new(1.0, 0.0, 0.0)]),
    };
    let moved = mesh.transform_similarity(&transform).unwrap();
    let normal = moved.normals.unwrap()[0];
    assert!((normal - Vector3D::new(0.0, 1.0, 0.0)).length() < 1e-10);
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices = vec![[0, 1, 2]];

    let mesh = TriangleMesh3D::new(vertices, indices).unwrap();

    assert_eq!(mesh.primitive_kind(), PrimitiveKind::Mesh);

    if let Some(bbox) = mesh.aabb() {
        assert_eq!(bbox.min().x(), 0.0);
        assert_eq!(bbox.max().x(), 1.0);
    }
}
