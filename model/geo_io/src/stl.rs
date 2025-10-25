//! STLファイルの読み書き機能
//!
//! ASCII STLとBinary STLの両方に対応。
//! 自動フォーマット判定機能付き。

use crate::error::StlError;
use geo_foundation::Scalar;
use geo_primitives::{Point3D, TriangleMesh3D, Vector3D};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::str::FromStr;

/// STLファイルを読み込む（自動フォーマット判定）
pub fn load_stl<T: Scalar + FromStr>(path: &Path) -> Result<TriangleMesh3D<T>, StlError>
where
    T::Err: std::fmt::Debug,
{
    if is_binary_stl(path)? {
        load_binary_stl(path)
    } else {
        load_ascii_stl(path)
    }
}

/// ASCII STLファイルを読み込む
pub fn load_ascii_stl<T: Scalar + FromStr>(path: &Path) -> Result<TriangleMesh3D<T>, StlError>
where
    T::Err: std::fmt::Debug,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut current_triangle_vertices = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("solid") {
            // STL header - continue
            continue;
        } else if line.starts_with("endsolid") {
            // End of STL
            break;
        } else if line.starts_with("facet normal") {
            // Start of new triangle - reset vertex collection
            current_triangle_vertices.clear();
        } else if line.starts_with("vertex") {
            // Parse vertex coordinates
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 4 {
                return Err(StlError::InvalidTriangle(format!(
                    "Invalid vertex line: {}",
                    line
                )));
            }

            let x = parts[1].parse::<T>().map_err(|_| {
                StlError::PrecisionConversion(format!("Failed to parse X coordinate: {}", parts[1]))
            })?;
            let y = parts[2].parse::<T>().map_err(|_| {
                StlError::PrecisionConversion(format!("Failed to parse Y coordinate: {}", parts[2]))
            })?;
            let z = parts[3].parse::<T>().map_err(|_| {
                StlError::PrecisionConversion(format!("Failed to parse Z coordinate: {}", parts[3]))
            })?;

            let vertex = Point3D::new(x, y, z);
            let vertex_index = add_or_find_vertex(&mut vertices, vertex);
            current_triangle_vertices.push(vertex_index);
        } else if line.starts_with("endfacet") {
            // End of triangle - create triangle indices
            if current_triangle_vertices.len() != 3 {
                return Err(StlError::InvalidTriangle(format!(
                    "Triangle must have exactly 3 vertices, found {}",
                    current_triangle_vertices.len()
                )));
            }

            indices.push([
                current_triangle_vertices[0],
                current_triangle_vertices[1],
                current_triangle_vertices[2],
            ]);
        }
    }

    TriangleMesh3D::new(vertices, indices).map_err(StlError::InvalidTriangle)
}

/// Binary STLファイルを読み込む
pub fn load_binary_stl<T: Scalar>(path: &Path) -> Result<TriangleMesh3D<T>, StlError> {
    let mut file = File::open(path)?;

    // Skip 80-byte header
    let mut header = [0u8; 80];
    file.read_exact(&mut header)?;

    // Read triangle count (4 bytes, little endian)
    let mut triangle_count_bytes = [0u8; 4];
    file.read_exact(&mut triangle_count_bytes)?;
    let triangle_count = u32::from_le_bytes(triangle_count_bytes);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Read triangles
    for _ in 0..triangle_count {
        // Normal vector (12 bytes, 3 f32s) - we'll ignore this and recalculate
        let mut normal_bytes = [0u8; 12];
        file.read_exact(&mut normal_bytes)?;

        // Three vertices (36 bytes, 9 f32s)
        let mut triangle_vertices = Vec::new();

        for _ in 0..3 {
            let mut vertex_bytes = [0u8; 12]; // 3 f32s
            file.read_exact(&mut vertex_bytes)?;

            let x_bytes = [
                vertex_bytes[0],
                vertex_bytes[1],
                vertex_bytes[2],
                vertex_bytes[3],
            ];
            let y_bytes = [
                vertex_bytes[4],
                vertex_bytes[5],
                vertex_bytes[6],
                vertex_bytes[7],
            ];
            let z_bytes = [
                vertex_bytes[8],
                vertex_bytes[9],
                vertex_bytes[10],
                vertex_bytes[11],
            ];

            let x_f32 = f32::from_le_bytes(x_bytes);
            let y_f32 = f32::from_le_bytes(y_bytes);
            let z_f32 = f32::from_le_bytes(z_bytes);

            // Convert f32 to T
            let x = T::from_f32(x_f32);
            let y = T::from_f32(y_f32);
            let z = T::from_f32(z_f32);

            let vertex = Point3D::new(x, y, z);
            let vertex_index = add_or_find_vertex(&mut vertices, vertex);
            triangle_vertices.push(vertex_index);
        }

        // Attribute byte count (2 bytes) - usually 0
        let mut attr_bytes = [0u8; 2];
        file.read_exact(&mut attr_bytes)?;

        indices.push([
            triangle_vertices[0],
            triangle_vertices[1],
            triangle_vertices[2],
        ]);
    }

    TriangleMesh3D::new(vertices, indices).map_err(StlError::InvalidTriangle)
}

/// ASCII STLファイルとして保存
pub fn save_ascii_stl<T: Scalar>(mesh: &TriangleMesh3D<T>, path: &Path) -> Result<(), StlError> {
    let mut file = File::create(path)?;

    writeln!(file, "solid exported_mesh")?;

    for i in 0..mesh.triangle_count() {
        if let Some(triangle) = mesh.triangle(i) {
            // Calculate normal
            let normal = triangle.normal().unwrap_or_else(|| {
                Vector3D::new(T::ZERO, T::ZERO, T::ONE) // Default to Z-up if degenerate
            });

            writeln!(
                file,
                "  facet normal {} {} {}",
                normal.x().to_f64(),
                normal.y().to_f64(),
                normal.z().to_f64()
            )?;
            writeln!(file, "    outer loop")?;

            writeln!(
                file,
                "      vertex {} {} {}",
                triangle.vertex_a().x().to_f64(),
                triangle.vertex_a().y().to_f64(),
                triangle.vertex_a().z().to_f64()
            )?;
            writeln!(
                file,
                "      vertex {} {} {}",
                triangle.vertex_b().x().to_f64(),
                triangle.vertex_b().y().to_f64(),
                triangle.vertex_b().z().to_f64()
            )?;
            writeln!(
                file,
                "      vertex {} {} {}",
                triangle.vertex_c().x().to_f64(),
                triangle.vertex_c().y().to_f64(),
                triangle.vertex_c().z().to_f64()
            )?;

            writeln!(file, "    endloop")?;
            writeln!(file, "  endfacet")?;
        }
    }

    writeln!(file, "endsolid exported_mesh")?;
    Ok(())
}

/// STLファイルとして保存（デフォルトはASCII）
pub fn save_stl<T: Scalar>(mesh: &TriangleMesh3D<T>, path: &Path) -> Result<(), StlError> {
    save_ascii_stl(mesh, path)
}

/// ファイルがBinary STLかどうかを判定
fn is_binary_stl(path: &Path) -> Result<bool, StlError> {
    let mut file = File::open(path)?;

    // Read first 80 bytes (header)
    let mut header = [0u8; 80];
    if file.read_exact(&mut header).is_err() {
        return Ok(false); // Too small to be binary STL
    }

    // Read triangle count
    let mut count_bytes = [0u8; 4];
    if file.read_exact(&mut count_bytes).is_err() {
        return Ok(false); // No triangle count
    }

    let triangle_count = u32::from_le_bytes(count_bytes);

    // Check if file size matches expected binary STL size
    let expected_size = 80 + 4 + (triangle_count as u64 * 50); // Header + count + triangles
    let metadata = std::fs::metadata(path)?;
    let actual_size = metadata.len();

    // If sizes match, likely binary STL
    if actual_size == expected_size {
        return Ok(true);
    }

    // Check if header contains "solid" (could be ASCII)
    let header_str = String::from_utf8_lossy(&header);
    if header_str.trim_start().starts_with("solid") {
        return Ok(false); // Likely ASCII
    }

    // Default to binary if no clear ASCII indicators
    Ok(true)
}

/// 頂点を追加するか、既存の頂点のインデックスを見つける
/// 重複する頂点を自動的にマージ
fn add_or_find_vertex<T: Scalar>(vertices: &mut Vec<Point3D<T>>, vertex: Point3D<T>) -> usize {
    // Simple linear search for exact matches
    // TODO: In Phase 2, replace with spatial hash or other optimization
    for (index, existing_vertex) in vertices.iter().enumerate() {
        if vertex_equals(*existing_vertex, vertex) {
            return index;
        }
    }

    // Not found, add new vertex
    vertices.push(vertex);
    vertices.len() - 1
}

/// 頂点が等しいかどうかを判定（浮動小数点比較）
fn vertex_equals<T: Scalar>(a: Point3D<T>, b: Point3D<T>) -> bool {
    let tolerance = T::EPSILON;
    (a.x() - b.x()).abs() <= tolerance
        && (a.y() - b.y()).abs() <= tolerance
        && (a.z() - b.z()).abs() <= tolerance
}
