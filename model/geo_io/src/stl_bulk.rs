//! STL高速バルク読み込み
//!
//! メモリ連続配置とBinary STL最適化による高速読み込み
//! - Vec連続配置でキャッシュ効率向上
//! - Binary STL (f32) は直接メモリコピー
//! - ASCII STLは事前容量確保で再アロケーション回避

use crate::error::StlError;
use geo_contracts::Scalar;
use geo_primitives::{Point3D, TriangleMesh3D};
use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str::FromStr;

fn io_log_interval() -> u64 {
    frame_interval_from_env("REDRING_LOG_IO_INTERVAL", 50_000)
}

/// STL高速バルク読み込み構造体
///
/// メモリ連続配置により高速アクセスを実現
/// - `vertices`: [x,y,z, x,y,z, x,y,z, ...] の連続配列（3頂点 × N三角形）
/// - `normals`: [nx,ny,nz, nx,ny,nz, ...] の連続配列（オプション）
#[derive(Debug, Clone)]
pub struct StlTriangleBulk<T: Scalar> {
    /// 三角形数
    count: usize,
    /// 連続頂点配列 [x,y,z] × 3頂点 × count
    /// インデックス: triangle_idx * 9 + vertex_idx * 3 + component_idx
    vertices: Vec<T>,
    /// 法線配列（オプション）[nx,ny,nz] × count
    normals: Option<Vec<T>>,
}

/// STLインデックス付きバルク構造体（頂点重複削減版）
///
/// 読み込み時に自動的に頂点重複を削減
/// - `vertices`: ユニークな頂点配列 [x,y,z, x,y,z, ...]
/// - `indices`: 三角形インデックス配列 [[i1,i2,i3], ...]
/// - `normals`: 三角形ごとの法線（オプション）
#[derive(Debug, Clone)]
pub struct StlIndexedBulk<T: Scalar> {
    /// ユニークな頂点配列 [x,y,z] × unique_vertex_count
    vertices: Vec<T>,
    /// 三角形インデックス配列 [i1,i2,i3] × triangle_count
    indices: Vec<usize>,
    /// 三角形数
    triangle_count: usize,
    /// 法線配列（オプション）[nx,ny,nz] × triangle_count
    normals: Option<Vec<T>>,
}

impl<T: Scalar + FromStr> StlTriangleBulk<T> {
    /// 新しいバルクデータを作成
    pub fn new(count: usize) -> Self {
        Self {
            count,
            vertices: Vec::with_capacity(count * 9), // 3頂点 × 3成分
            normals: None,
        }
    }

    /// 法線データを有効化
    pub fn with_normals(mut self) -> Self {
        self.normals = Some(Vec::with_capacity(self.count * 3));
        self
    }

    /// 三角形数を取得
    pub fn triangle_count(&self) -> usize {
        self.count
    }

    /// 指定した三角形の頂点を取得（タプル形式）
    pub fn triangle_vertices(&self, index: usize) -> Option<([T; 3], [T; 3], [T; 3])> {
        if index >= self.count {
            return None;
        }

        let base = index * 9;
        Some((
            [
                self.vertices[base],
                self.vertices[base + 1],
                self.vertices[base + 2],
            ],
            [
                self.vertices[base + 3],
                self.vertices[base + 4],
                self.vertices[base + 5],
            ],
            [
                self.vertices[base + 6],
                self.vertices[base + 7],
                self.vertices[base + 8],
            ],
        ))
    }

    /// 指定した三角形の法線を取得
    pub fn triangle_normal(&self, index: usize) -> Option<[T; 3]> {
        if index >= self.count {
            return None;
        }

        self.normals.as_ref().map(|normals| {
            let base = index * 3;
            [normals[base], normals[base + 1], normals[base + 2]]
        })
    }

    /// 頂点データへの直接アクセス（GPU転送用）
    pub fn vertices_slice(&self) -> &[T] {
        &self.vertices
    }

    /// 法線データへの直接アクセス（GPU転送用）
    pub fn normals_slice(&self) -> Option<&[T]> {
        self.normals.as_deref()
    }

    /// ASCII STLファイルから読み込み（可変精度対応）
    pub fn from_ascii_stl(path: &Path) -> Result<Self, StlError> {
        tracing::debug!("Started reading ASCII STL: {:?}", path);

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // 1パス目: 三角形数をカウント
        let triangle_count = count_ascii_triangles(path)?;

        let mut bulk = Self::new(triangle_count).with_normals();
        let mut current_vertices = Vec::with_capacity(9);
        let mut current_normal = [T::ZERO; 3];

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.starts_with("facet normal") {
                // 法線解析
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    current_normal[0] = parts[2].parse::<T>().unwrap_or(T::ZERO);
                    current_normal[1] = parts[3].parse::<T>().unwrap_or(T::ZERO);
                    current_normal[2] = parts[4].parse::<T>().unwrap_or(T::ZERO);
                }
                current_vertices.clear();
            } else if line.starts_with("vertex") {
                // 頂点解析
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let x = parts[1].parse::<T>().map_err(|_| {
                        StlError::PrecisionConversion(format!("Failed to parse X: {}", parts[1]))
                    })?;
                    let y = parts[2].parse::<T>().map_err(|_| {
                        StlError::PrecisionConversion(format!("Failed to parse Y: {}", parts[2]))
                    })?;
                    let z = parts[3].parse::<T>().map_err(|_| {
                        StlError::PrecisionConversion(format!("Failed to parse Z: {}", parts[3]))
                    })?;

                    current_vertices.push(x);
                    current_vertices.push(y);
                    current_vertices.push(z);
                }
            } else if line.starts_with("endfacet") {
                // 三角形完成
                if current_vertices.len() == 9 {
                    bulk.vertices.extend_from_slice(&current_vertices);
                    if let Some(ref mut normals) = bulk.normals {
                        normals.extend_from_slice(&current_normal);
                    }

                    let parsed_triangles = bulk.vertices.len() / 9;
                    if should_log_every_n_frames(parsed_triangles as u64, io_log_interval()) {
                        tracing::trace!(
                            "ASCII STL read progress: {}/{} triangles",
                            parsed_triangles,
                            triangle_count
                        );
                    }
                }
            }
        }

        tracing::debug!(
            "Finished reading ASCII STL: {:?}, triangles={}",
            path,
            bulk.triangle_count()
        );

        Ok(bulk)
    }

    /// Binary STLファイルから読み込み（汎用版、型変換あり）
    pub fn from_binary_stl(path: &Path) -> Result<Self, StlError> {
        tracing::debug!("Started reading Binary STL: {:?}", path);

        let mut file = File::open(path)?;

        // Skip 80-byte header
        let mut header = [0u8; 80];
        file.read_exact(&mut header)?;

        // Read triangle count (4 bytes, little endian)
        let mut triangle_count_bytes = [0u8; 4];
        file.read_exact(&mut triangle_count_bytes)?;
        let triangle_count = u32::from_le_bytes(triangle_count_bytes) as usize;

        let mut bulk = Self::new(triangle_count).with_normals();

        // Binary STL構造: [normal(3*f32), vertex1(3*f32), vertex2(3*f32), vertex3(3*f32), attribute(u16)] × count
        for tri_idx in 0..triangle_count {
            // Normal (3 × f32)
            let mut normal = [0f32; 3];
            for value in &mut normal {
                let mut bytes = [0u8; 4];
                file.read_exact(&mut bytes)?;
                *value = f32::from_le_bytes(bytes);
            }

            // Vertices (3 × 3 × f32)
            let mut triangle_verts = [0f32; 9];
            for value in &mut triangle_verts {
                let mut bytes = [0u8; 4];
                file.read_exact(&mut bytes)?;
                *value = f32::from_le_bytes(bytes);
            }

            // Attribute bytes (u16) - skip
            let mut attr = [0u8; 2];
            file.read_exact(&mut attr)?;

            // f32 → T 型変換
            for &v in &triangle_verts {
                bulk.vertices.push(T::from_f64(v as f64));
            }

            if let Some(ref mut normals) = bulk.normals {
                for &n in &normal {
                    normals.push(T::from_f64(n as f64));
                }
            }

            let parsed_triangles = tri_idx + 1;
            if should_log_every_n_frames(parsed_triangles as u64, io_log_interval()) {
                tracing::trace!(
                    "Binary STL read progress: {}/{} triangles",
                    parsed_triangles,
                    triangle_count
                );
            }
        }

        tracing::debug!(
            "Finished reading Binary STL: {:?}, triangles={}",
            path,
            bulk.triangle_count()
        );

        Ok(bulk)
    }

    /// TriangleMesh3Dへ変換（頂点重複削除）
    pub fn to_triangle_mesh(self) -> Result<TriangleMesh3D<T>, StlError> {
        let mut vertex_map: HashMap<(u64, u64, u64), usize> = HashMap::new();
        let mut unique_vertices = Vec::new();
        let mut indices = Vec::with_capacity(self.count);

        for tri_idx in 0..self.count {
            let base = tri_idx * 9;
            let mut triangle_indices = [0usize; 3];

            #[allow(clippy::needless_range_loop)]
            for v_idx in 0..3 {
                let offset = base + v_idx * 3;
                let x = self.vertices[offset];
                let y = self.vertices[offset + 1];
                let z = self.vertices[offset + 2];

                // 浮動小数点数のハッシュキー（ビット表現使用）
                let key = (x.to_bits(), y.to_bits(), z.to_bits());

                let unique_idx = *vertex_map.entry(key).or_insert_with(|| {
                    let idx = unique_vertices.len();
                    unique_vertices.push(Point3D::new(x, y, z));
                    idx
                });

                triangle_indices[v_idx] = unique_idx;
            }

            indices.push(triangle_indices);
        }

        TriangleMesh3D::new(unique_vertices, indices).map_err(StlError::InvalidTriangle)
    }
}

// f32専用高速実装
impl StlTriangleBulk<f32> {
    /// Binary STL高速読み込み（f32専用、最適化版）
    ///
    /// Binary STLはf32固定仕様のため、直接メモリコピーで高速化
    pub fn from_binary_stl_fast(path: &Path) -> Result<Self, StlError> {
        tracing::debug!("Started fast Binary STL read: {:?}", path);

        let mut file = File::open(path)?;

        // Skip 80-byte header
        let mut header = [0u8; 80];
        file.read_exact(&mut header)?;

        // Read triangle count
        let mut triangle_count_bytes = [0u8; 4];
        file.read_exact(&mut triangle_count_bytes)?;
        let triangle_count = u32::from_le_bytes(triangle_count_bytes) as usize;

        let mut bulk = Self::new(triangle_count).with_normals();

        // 一括読み込み用バッファ（50バイト = 12*f32 + 2バイト属性）
        let bytes_per_triangle = 50;
        let mut buffer = vec![0u8; triangle_count * bytes_per_triangle];
        file.read_exact(&mut buffer)?;

        // 高速パース（memcpy相当）
        for tri_idx in 0..triangle_count {
            let offset = tri_idx * bytes_per_triangle;

            // Normal (3 × f32)
            for i in 0..3 {
                let bytes = &buffer[offset + i * 4..offset + (i + 1) * 4];
                let normal_val = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                if let Some(ref mut normals) = bulk.normals {
                    normals.push(normal_val);
                }
            }

            // Vertices (9 × f32)
            for i in 0..9 {
                let base = offset + 12 + i * 4; // 12バイト = 法線3*4
                let bytes = &buffer[base..base + 4];
                let vertex_val = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                bulk.vertices.push(vertex_val);
            }

            // 属性バイトはスキップ（既にバッファに含まれている）

            let parsed_triangles = tri_idx + 1;
            if should_log_every_n_frames(parsed_triangles as u64, io_log_interval()) {
                tracing::trace!(
                    "Fast Binary STL read progress: {}/{} triangles",
                    parsed_triangles,
                    triangle_count
                );
            }
        }

        tracing::debug!(
            "Finished fast Binary STL read: {:?}, triangles={}",
            path,
            bulk.triangle_count()
        );

        Ok(bulk)
    }
}

/// ASCII STLファイルの三角形数をカウント
fn count_ascii_triangles(path: &Path) -> Result<usize, StlError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("endfacet") {
            count += 1;
        }
    }

    Ok(count)
}

impl<T: Scalar + FromStr> StlIndexedBulk<T> {
    /// ASCII STLから読み込み（重複削減）
    pub fn from_ascii_stl(path: &Path) -> Result<Self, StlError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // 1パス目: 三角形数をカウント
        let triangle_count = count_ascii_triangles(path)?;

        let mut vertex_map: HashMap<(u64, u64, u64), usize> = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::with_capacity(triangle_count * 3);
        let mut normals = Vec::with_capacity(triangle_count * 3);

        let mut current_vertices = Vec::with_capacity(3);
        let mut current_normal = [T::ZERO; 3];

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.starts_with("facet normal") {
                // 法線解析
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    current_normal[0] = parts[2].parse::<T>().unwrap_or(T::ZERO);
                    current_normal[1] = parts[3].parse::<T>().unwrap_or(T::ZERO);
                    current_normal[2] = parts[4].parse::<T>().unwrap_or(T::ZERO);
                }
                current_vertices.clear();
            } else if line.starts_with("vertex") {
                // 頂点解析
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let x = parts[1].parse::<T>().map_err(|_| {
                        StlError::PrecisionConversion(format!("Failed to parse X: {}", parts[1]))
                    })?;
                    let y = parts[2].parse::<T>().map_err(|_| {
                        StlError::PrecisionConversion(format!("Failed to parse Y: {}", parts[2]))
                    })?;
                    let z = parts[3].parse::<T>().map_err(|_| {
                        StlError::PrecisionConversion(format!("Failed to parse Z: {}", parts[3]))
                    })?;

                    // 重複チェック
                    let key = (x.to_bits(), y.to_bits(), z.to_bits());
                    let vertex_idx = *vertex_map.entry(key).or_insert_with(|| {
                        let idx = vertices.len() / 3;
                        vertices.push(x);
                        vertices.push(y);
                        vertices.push(z);
                        idx
                    });

                    current_vertices.push(vertex_idx);
                }
            } else if line.starts_with("endfacet") {
                // 三角形完成
                if current_vertices.len() == 3 {
                    indices.extend_from_slice(&current_vertices);
                    normals.extend_from_slice(&current_normal);
                }
            }
        }

        Ok(Self {
            vertices,
            indices,
            triangle_count,
            normals: Some(normals),
        })
    }

    /// Binary STLから読み込み（重複削減）
    pub fn from_binary_stl(path: &Path) -> Result<Self, StlError> {
        let mut file = File::open(path)?;

        // Skip 80-byte header
        let mut header = [0u8; 80];
        file.read_exact(&mut header)?;

        // Read triangle count
        let mut triangle_count_bytes = [0u8; 4];
        file.read_exact(&mut triangle_count_bytes)?;
        let triangle_count = u32::from_le_bytes(triangle_count_bytes) as usize;

        let mut vertex_map: HashMap<(u64, u64, u64), usize> = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::with_capacity(triangle_count * 3);
        let mut normals = Vec::with_capacity(triangle_count * 3);

        // Binary STL構造: [normal(3*f32), vertex1(3*f32), vertex2(3*f32), vertex3(3*f32), attribute(u16)]
        for _ in 0..triangle_count {
            // Normal (3 × f32)
            let mut normal = [T::ZERO; 3];
            for value in &mut normal {
                let mut bytes = [0u8; 4];
                file.read_exact(&mut bytes)?;
                let n = f32::from_le_bytes(bytes);
                *value = T::from_f64(f64::from(n));
            }

            // Vertices (3 × 3 × f32)
            for _ in 0..3 {
                let mut vertex = [T::ZERO; 3];
                for value in &mut vertex {
                    let mut bytes = [0u8; 4];
                    file.read_exact(&mut bytes)?;
                    let v = f32::from_le_bytes(bytes);
                    *value = T::from_f64(f64::from(v));
                }

                // 重複チェック
                let key = (
                    vertex[0].to_bits(),
                    vertex[1].to_bits(),
                    vertex[2].to_bits(),
                );
                let vertex_idx = *vertex_map.entry(key).or_insert_with(|| {
                    let idx = vertices.len() / 3;
                    vertices.extend_from_slice(&vertex);
                    idx
                });

                indices.push(vertex_idx);
            }

            // Attribute bytes (u16) - skip
            let mut attr = [0u8; 2];
            file.read_exact(&mut attr)?;

            normals.extend_from_slice(&normal);
        }

        Ok(Self {
            vertices,
            indices,
            triangle_count,
            normals: Some(normals),
        })
    }

    /// 三角形数を取得
    pub fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    /// ユニークな頂点数を取得
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    /// 頂点配列への直接アクセス（GPU転送用）
    pub fn vertices_slice(&self) -> &[T] {
        &self.vertices
    }

    /// インデックス配列への直接アクセス
    pub fn indices_slice(&self) -> &[usize] {
        &self.indices
    }

    /// 法線配列への直接アクセス
    pub fn normals_slice(&self) -> Option<&[T]> {
        self.normals.as_deref()
    }

    /// 指定した三角形のインデックスを取得
    pub fn triangle_indices(&self, index: usize) -> Option<[usize; 3]> {
        if index >= self.triangle_count {
            return None;
        }

        let base = index * 3;
        Some([
            self.indices[base],
            self.indices[base + 1],
            self.indices[base + 2],
        ])
    }

    /// 指定した頂点の座標を取得
    pub fn vertex(&self, index: usize) -> Option<[T; 3]> {
        if index >= self.vertex_count() {
            return None;
        }

        let base = index * 3;
        Some([
            self.vertices[base],
            self.vertices[base + 1],
            self.vertices[base + 2],
        ])
    }

    /// TriangleMesh3Dへ変換
    pub fn to_triangle_mesh(self) -> Result<TriangleMesh3D<T>, StlError> {
        let mut point_vertices = Vec::with_capacity(self.vertex_count());
        for i in 0..self.vertex_count() {
            let base = i * 3;
            point_vertices.push(Point3D::new(
                self.vertices[base],
                self.vertices[base + 1],
                self.vertices[base + 2],
            ));
        }

        let mut triangle_indices = Vec::with_capacity(self.triangle_count);
        for i in 0..self.triangle_count {
            let base = i * 3;
            triangle_indices.push([
                self.indices[base],
                self.indices[base + 1],
                self.indices[base + 2],
            ]);
        }

        TriangleMesh3D::new(point_vertices, triangle_indices).map_err(StlError::InvalidTriangle)
    }

    /// メモリ削減率を計算（%）
    pub fn memory_reduction(&self) -> f32 {
        let original_vertices = self.triangle_count * 3;
        let unique_vertices = self.vertex_count();
        if original_vertices == 0 {
            return 0.0;
        }
        (1.0 - (unique_vertices as f32 / original_vertices as f32)) * 100.0
    }
}

// f32専用高速実装
impl StlIndexedBulk<f32> {
    /// Binary STL高速読み込み（f32専用、重複削減版）
    pub fn from_binary_stl_fast(path: &Path) -> Result<Self, StlError> {
        let mut file = File::open(path)?;

        // Skip 80-byte header
        let mut header = [0u8; 80];
        file.read_exact(&mut header)?;

        // Read triangle count
        let mut triangle_count_bytes = [0u8; 4];
        file.read_exact(&mut triangle_count_bytes)?;
        let triangle_count = u32::from_le_bytes(triangle_count_bytes) as usize;

        let mut vertex_map: HashMap<(u32, u32, u32), usize> = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::with_capacity(triangle_count * 3);
        let mut normals = Vec::with_capacity(triangle_count * 3);

        // 一括読み込み用バッファ（50バイト = 12*f32 + 2バイト属性）
        let bytes_per_triangle = 50;
        let mut buffer = vec![0u8; triangle_count * bytes_per_triangle];
        file.read_exact(&mut buffer)?;

        // 高速パース
        for tri_idx in 0..triangle_count {
            let offset = tri_idx * bytes_per_triangle;

            // Normal (3 × f32)
            for i in 0..3 {
                let bytes = &buffer[offset + i * 4..offset + (i + 1) * 4];
                let normal_val = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                normals.push(normal_val);
            }

            // Vertices (9 × f32)
            for v_idx in 0..3 {
                let base = offset + 12 + v_idx * 12; // 12バイト = 法線3*4
                let x_bytes = &buffer[base..base + 4];
                let y_bytes = &buffer[base + 4..base + 8];
                let z_bytes = &buffer[base + 8..base + 12];

                let x = f32::from_le_bytes([x_bytes[0], x_bytes[1], x_bytes[2], x_bytes[3]]);
                let y = f32::from_le_bytes([y_bytes[0], y_bytes[1], y_bytes[2], y_bytes[3]]);
                let z = f32::from_le_bytes([z_bytes[0], z_bytes[1], z_bytes[2], z_bytes[3]]);

                // 重複チェック（f32専用）
                let key = (x.to_bits(), y.to_bits(), z.to_bits());
                let vertex_idx = *vertex_map.entry(key).or_insert_with(|| {
                    let idx = vertices.len() / 3;
                    vertices.push(x);
                    vertices.push(y);
                    vertices.push(z);
                    idx
                });

                indices.push(vertex_idx);
            }
        }

        Ok(Self {
            vertices,
            indices,
            triangle_count,
            normals: Some(normals),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_bulk_structure() {
        let bulk = StlTriangleBulk::<f64>::new(2);
        assert_eq!(bulk.triangle_count(), 2);
        assert_eq!(bulk.vertices.capacity(), 18); // 2三角形 × 9値
    }

    #[test]
    fn test_triangle_access() {
        let mut bulk = StlTriangleBulk::<f64>::new(1);
        bulk.vertices
            .extend_from_slice(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

        let (v1, v2, v3) = bulk.triangle_vertices(0).unwrap();
        assert_eq!(v1, [0.0, 0.0, 0.0]);
        assert_eq!(v2, [1.0, 0.0, 0.0]);
        assert_eq!(v3, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_ascii_bulk_load() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "solid test
facet normal 0 0 1
  outer loop
    vertex 0 0 0
    vertex 1 0 0
    vertex 0 1 0
  endloop
endfacet
endsolid test"
        )
        .unwrap();

        let bulk = StlTriangleBulk::<f64>::from_ascii_stl(temp_file.path()).unwrap();
        assert_eq!(bulk.triangle_count(), 1);

        let (v1, v2, v3) = bulk.triangle_vertices(0).unwrap();
        assert_eq!(v1, [0.0, 0.0, 0.0]);
        assert_eq!(v2, [1.0, 0.0, 0.0]);
        assert_eq!(v3, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_to_triangle_mesh() {
        let mut bulk = StlTriangleBulk::<f64>::new(1);
        bulk.vertices
            .extend_from_slice(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

        let mesh = bulk.to_triangle_mesh().unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn test_indexed_bulk_structure() {
        use tempfile::NamedTempFile;

        // ASCII STL作成（重複頂点あり）
        let stl_content = r#"solid test
facet normal 0.0 0.0 1.0
  outer loop
    vertex 0.0 0.0 0.0
    vertex 1.0 0.0 0.0
    vertex 0.0 1.0 0.0
  endloop
endfacet
facet normal 0.0 0.0 1.0
  outer loop
    vertex 1.0 0.0 0.0
    vertex 1.0 1.0 0.0
    vertex 0.0 1.0 0.0
  endloop
endfacet
endsolid test
"#;

        let mut tmp = NamedTempFile::new().unwrap();
        use std::io::Write;
        tmp.write_all(stl_content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let indexed = StlIndexedBulk::<f64>::from_ascii_stl(tmp.path()).unwrap();

        assert_eq!(indexed.triangle_count(), 2);
        assert_eq!(indexed.vertex_count(), 4); // 6頂点→4ユニーク頂点
        assert_eq!(indexed.indices.len(), 6); // 2三角形×3
        assert!(indexed.normals.is_some());
    }

    #[test]
    fn test_indexed_memory_reduction() {
        use tempfile::NamedTempFile;

        // 重複頂点を持つ立方体の一部（共有頂点あり）
        let stl_content = r#"solid test
facet normal 0.0 0.0 1.0
  outer loop
    vertex 0.0 0.0 0.0
    vertex 1.0 0.0 0.0
    vertex 1.0 1.0 0.0
  endloop
endfacet
facet normal 0.0 0.0 1.0
  outer loop
    vertex 0.0 0.0 0.0
    vertex 1.0 1.0 0.0
    vertex 0.0 1.0 0.0
  endloop
endfacet
endsolid test
"#;

        let mut tmp = NamedTempFile::new().unwrap();
        use std::io::Write;
        tmp.write_all(stl_content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let indexed = StlIndexedBulk::<f32>::from_ascii_stl(tmp.path()).unwrap();

        // 2三角形 = 6頂点 → 4ユニーク頂点
        // 削減率 = (1 - 4/6) * 100 = 33.33%
        let reduction = indexed.memory_reduction();
        assert!(reduction > 33.0 && reduction < 34.0);
        println!("Memory reduction: {:.1}%", reduction);
    }

    #[test]
    fn test_indexed_vertex_access() {
        use tempfile::NamedTempFile;

        let stl_content = r#"solid test
facet normal 0.0 0.0 1.0
  outer loop
    vertex 0.0 0.0 0.0
    vertex 1.0 0.0 0.0
    vertex 0.0 1.0 0.0
  endloop
endfacet
endsolid test
"#;

        let mut tmp = NamedTempFile::new().unwrap();
        use std::io::Write;
        tmp.write_all(stl_content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let indexed = StlIndexedBulk::<f64>::from_ascii_stl(tmp.path()).unwrap();

        // 頂点アクセス
        let v0 = indexed.vertex(0).unwrap();
        assert_eq!(v0, [0.0, 0.0, 0.0]);

        let v1 = indexed.vertex(1).unwrap();
        assert_eq!(v1, [1.0, 0.0, 0.0]);

        // インデックスアクセス
        let tri_indices = indexed.triangle_indices(0).unwrap();
        assert_eq!(tri_indices.len(), 3);
    }

    #[test]
    fn test_indexed_to_triangle_mesh() {
        use tempfile::NamedTempFile;

        let stl_content = r#"solid test
facet normal 0.0 0.0 1.0
  outer loop
    vertex 0.0 0.0 0.0
    vertex 1.0 0.0 0.0
    vertex 0.0 1.0 0.0
  endloop
endfacet
facet normal 0.0 0.0 1.0
  outer loop
    vertex 1.0 0.0 0.0
    vertex 1.0 1.0 0.0
    vertex 0.0 1.0 0.0
  endloop
endfacet
endsolid test
"#;

        let mut tmp = NamedTempFile::new().unwrap();
        use std::io::Write;
        tmp.write_all(stl_content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let indexed = StlIndexedBulk::<f32>::from_ascii_stl(tmp.path()).unwrap();
        let mesh = indexed.to_triangle_mesh().unwrap();

        assert_eq!(mesh.vertex_count(), 4); // 4ユニーク頂点
        assert_eq!(mesh.triangle_count(), 2); // 2三角形
    }
}
