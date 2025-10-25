//! STLローダーのテスト

use geo_io::stl;
use geo_primitives::{Point3D, TriangleMesh3D};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_ascii_stl_roundtrip() {
    // シンプルな三角形メッシュを作成
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let indices = vec![[0, 1, 2]];

    let original_mesh = TriangleMesh3D::new(vertices, indices).unwrap();

    // 一時ファイルに保存
    let temp_file = NamedTempFile::new().unwrap();
    stl::save_ascii_stl(&original_mesh, temp_file.path()).unwrap();

    // 読み込み
    let loaded_mesh: TriangleMesh3D<f64> = stl::load_ascii_stl(temp_file.path()).unwrap();

    // 検証
    assert_eq!(loaded_mesh.triangle_count(), 1);
    assert_eq!(loaded_mesh.vertex_count(), 3); // 重複頂点がマージされる

    let triangle = loaded_mesh.triangle(0).unwrap();
    assert_eq!(triangle.vertex_a(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(triangle.vertex_b(), Point3D::new(1.0, 0.0, 0.0));
    assert_eq!(triangle.vertex_c(), Point3D::new(0.0, 1.0, 0.0));
}

#[test]
fn test_ascii_stl_manual_content() {
    // 手動でSTLコンテンツを作成
    let stl_content = r#"solid test
  facet normal 0 0 1
    outer loop
      vertex 0.0 0.0 0.0
      vertex 1.0 0.0 0.0
      vertex 0.0 1.0 0.0
    endloop
  endfacet
endsolid test
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(stl_content.as_bytes()).unwrap();

    let mesh: TriangleMesh3D<f64> = stl::load_ascii_stl(temp_file.path()).unwrap();

    assert_eq!(mesh.triangle_count(), 1);
    assert_eq!(mesh.vertex_count(), 3);

    let triangle = mesh.triangle(0).unwrap();
    assert_eq!(triangle.vertex_a(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(triangle.vertex_b(), Point3D::new(1.0, 0.0, 0.0));
    assert_eq!(triangle.vertex_c(), Point3D::new(0.0, 1.0, 0.0));
}

#[test]
fn test_quad_mesh_ascii_stl() {
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

    let original_mesh = TriangleMesh3D::new(vertices, indices).unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    stl::save_ascii_stl(&original_mesh, temp_file.path()).unwrap();

    let loaded_mesh: TriangleMesh3D<f64> = stl::load_ascii_stl(temp_file.path()).unwrap();

    assert_eq!(loaded_mesh.triangle_count(), 2);
    assert_eq!(loaded_mesh.vertex_count(), 4);
}

#[test]
fn test_empty_stl() {
    let empty_mesh = TriangleMesh3D::<f64>::empty();

    let temp_file = NamedTempFile::new().unwrap();
    stl::save_ascii_stl(&empty_mesh, temp_file.path()).unwrap();

    let loaded_mesh: TriangleMesh3D<f64> = stl::load_ascii_stl(temp_file.path()).unwrap();

    assert_eq!(loaded_mesh.triangle_count(), 0);
    assert_eq!(loaded_mesh.vertex_count(), 0);
    assert!(loaded_mesh.is_empty());
}

#[test]
fn test_load_stl_auto_detection() {
    // ASCII STLファイルの自動判定テスト
    let stl_content = r#"solid test
  facet normal 0 0 1
    outer loop
      vertex 0.0 0.0 0.0
      vertex 1.0 0.0 0.0
      vertex 0.0 1.0 0.0
    endloop
  endfacet
endsolid test
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(stl_content.as_bytes()).unwrap();

    // 自動判定で読み込み
    let mesh: TriangleMesh3D<f64> = stl::load_stl(temp_file.path()).unwrap();

    assert_eq!(mesh.triangle_count(), 1);
    assert_eq!(mesh.vertex_count(), 3);
}
