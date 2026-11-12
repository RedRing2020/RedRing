//! 頂点・頂点番号を GPU のバッファーに置く
//!
//! バッファー 1 つの大きさには上限（`wgpu::Limits::max_buffer_size`）があるため、大きなデータは上限に収まるように
//! 分けて置く。GPU のメモリが足りないなどで置けないときは、アプリを止めずに誤りを返す。

use crate::vertex_3d::MeshVertex;

/// 頂点・頂点番号を GPU に置けない（GPU のメモリが足りないなど）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDataError {
    /// 置こうとしたもの（ログに出す名前）
    pub what: &'static str,
    /// 置こうとした数
    pub count: usize,
    /// GPU の誤り
    pub reason: String,
}

impl std::fmt::Display for GpuDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot place {} {} on the GPU: {}",
            self.count, self.what, self.reason
        )
    }
}

impl std::error::Error for GpuDataError {}

/// バイト列 `contents` を置いた、使い方 `usage` のバッファー
///
/// GPU のメモリが足りない・GPU の内部の誤り・検証の誤り（大きさが上限を超えるなど）のときは、アプリを止めずに
/// 誤りの理由を返す（`create_buffer_init` は、置けないときに止まるため使わない）。
pub(crate) fn checked_buffer(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    contents: &[u8],
) -> Result<wgpu::Buffer, String> {
    // 書き込む大きさは、コピーの単位の倍数で 0 より大きくする
    let align = wgpu::COPY_BUFFER_ALIGNMENT;
    let size = (contents.len() as u64).div_ceil(align).max(1) * align;
    let out_of_memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: true,
    });
    let mapped = match buffer.get_mapped_range_mut(..) {
        Ok(mut range) => {
            range.slice(..contents.len()).copy_from_slice(contents);
            drop(range);
            buffer.unmap();
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    };
    // 積んだ順と逆に取り出す
    let validation = pollster::block_on(validation.pop());
    let internal = pollster::block_on(internal.pop());
    let out_of_memory = pollster::block_on(out_of_memory.pop());
    match (out_of_memory.or(internal).or(validation), mapped) {
        (Some(error), _) => Err(error.to_string()),
        (None, Err(error)) => Err(error),
        (None, Ok(())) => Ok(buffer),
    }
}

/// バッファー 1 つの大きさの上限 `max_buffer_size`（バイト）に収まる、メッシュの頂点の数と頂点番号の数（頂点番号は
/// 三角形を途中で分けないよう 3 の倍数。少なくとも 1 つの三角形）
pub(crate) fn mesh_counts_per_buffer(max_buffer_size: u64) -> (usize, usize) {
    // 描くときの数は u32 で渡す
    let count = |size: usize| {
        let count = (max_buffer_size / size as u64).min(u64::from(u32::MAX));
        usize::try_from(count).unwrap_or(usize::MAX)
    };
    let vertices = count(std::mem::size_of::<MeshVertex>()).max(3);
    let indices = (count(std::mem::size_of::<u32>()) / 3 * 3).max(3);
    (vertices, indices)
}

/// メッシュの塊の頂点
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PartVertices<'a, T> {
    /// 元の頂点のすべて（塊の間で共有する。頂点番号は元のまま）
    Shared,
    /// 塊が使う頂点だけ（頂点番号は塊の中の番号）
    Own(&'a [T]),
}

/// 三角形のメッシュ（頂点 `vertices` と、3 つずつ三角形を表す頂点番号 `indices`）を、頂点の数が `max_vertices` 以下、
/// 頂点番号の数が `max_indices` 以下の塊に分け、塊を 1 つずつ作って `place` に渡す（渡した後に放す。すべての塊を
/// 同時には持たない）
///
/// - 頂点番号が整っていて（3 の倍数で、すべて頂点の範囲の中）、頂点が 1 つの塊に収まるときは、頂点は共有し、頂点
///   番号だけを三角形の境で区切る（写さない。収まるメッシュは 1 つの塊になる）
/// - そうでないときは、三角形を途中で分けず、塊ごとに使う頂点だけを持ち、頂点番号を付け直す。頂点の範囲の外の
///   頂点番号を持つ三角形と、3 つに満たない最後の頂点番号は描けないため除く
///
/// # Errors
///
/// `place` の誤り（残りの塊は作らない）。
pub(crate) fn for_each_mesh_part<T: Copy, E>(
    vertices: &[T],
    indices: &[u32],
    (max_vertices, max_indices): (usize, usize),
    mut place: impl FnMut(PartVertices<'_, T>, &[u32]) -> Result<(), E>,
) -> Result<(), E> {
    let well_formed = indices.len().is_multiple_of(3)
        && indices
            .iter()
            .all(|&index| (index as usize) < vertices.len());
    if well_formed && vertices.len() <= max_vertices {
        for part in indices.chunks(max_indices) {
            place(PartVertices::Shared, part)?;
        }
        return Ok(());
    }
    // 元の頂点番号から、今の塊の中の番号（塊にない頂点は `UNUSED`）
    const UNUSED: u32 = u32::MAX;
    let mut local = vec![UNUSED; vertices.len()];
    // 今の塊の頂点の、元の頂点番号（塊を渡した後に `local` を戻す）
    let mut sources: Vec<u32> = Vec::new();
    let mut part_vertices: Vec<T> = Vec::new();
    let mut part_indices: Vec<u32> = Vec::new();
    for triangle in indices.as_chunks::<3>().0 {
        if triangle
            .iter()
            .any(|&index| index as usize >= vertices.len())
        {
            continue;
        }
        // この三角形で新しく足す頂点の数（同じ頂点を 2 度数えない）
        let new = triangle
            .iter()
            .enumerate()
            .filter(|&(i, &index)| {
                local[index as usize] == UNUSED && !triangle[..i].contains(&index)
            })
            .count();
        if !part_indices.is_empty()
            && (part_vertices.len() + new > max_vertices || part_indices.len() + 3 > max_indices)
        {
            place(PartVertices::Own(&part_vertices), &part_indices)?;
            for source in sources.drain(..) {
                local[source as usize] = UNUSED;
            }
            part_vertices.clear();
            part_indices.clear();
        }
        for &index in triangle {
            let slot = &mut local[index as usize];
            if *slot == UNUSED {
                *slot = part_vertices.len() as u32;
                part_vertices.push(vertices[index as usize]);
                sources.push(index);
            }
            part_indices.push(*slot);
        }
    }
    if !part_indices.is_empty() {
        place(PartVertices::Own(&part_vertices), &part_indices)?;
    }
    Ok(())
}

/// GPU に置いたメッシュの塊
pub(crate) struct IndexedPart {
    /// 頂点のバッファー（塊の間で共有することがある）
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl IndexedPart {
    /// 頂点番号で三角形を描く（パイプラインと uniform は設定済み）
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

/// 三角形のメッシュを、バッファー 1 つの大きさの上限に収まる塊に分けて GPU に置く（`label` はバッファーの名前）
///
/// # Errors
///
/// GPU のメモリが足りないなどで置けない場合（残りの塊は作らない）。
pub(crate) fn indexed_parts(
    device: &wgpu::Device,
    label: &str,
    vertices: &[MeshVertex],
    indices: &[u32],
) -> Result<Vec<IndexedPart>, GpuDataError> {
    let error = |reason| GpuDataError {
        what: "mesh triangles",
        count: indices.len() / 3,
        reason,
    };
    let place = |usage, contents: &[u8]| checked_buffer(device, label, usage, contents);
    let mut shared: Option<wgpu::Buffer> = None;
    let mut parts = Vec::new();
    for_each_mesh_part(
        vertices,
        indices,
        mesh_counts_per_buffer(device.limits().max_buffer_size),
        |part_vertices, part_indices| {
            let vertex_buffer = match part_vertices {
                PartVertices::Shared => match &shared {
                    Some(buffer) => buffer.clone(),
                    None => {
                        let buffer =
                            place(wgpu::BufferUsages::VERTEX, bytemuck::cast_slice(vertices))?;
                        shared = Some(buffer.clone());
                        buffer
                    }
                },
                PartVertices::Own(own) => {
                    place(wgpu::BufferUsages::VERTEX, bytemuck::cast_slice(own))?
                }
            };
            let index_buffer = place(
                wgpu::BufferUsages::INDEX,
                bytemuck::cast_slice(part_indices),
            )?;
            parts.push(IndexedPart {
                vertex_buffer,
                index_buffer,
                // 塊の頂点番号の数は u32 に収まる（`mesh_counts_per_buffer`）
                index_count: part_indices.len() as u32,
            });
            Ok(())
        },
    )
    .map_err(error)?;
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// メッシュを塊に分け、塊ごとの頂点の数・頂点番号の数と、塊が表す三角形（元の頂点の値の 3 つ組）と、頂点を
    /// 共有したかを返す
    fn split(
        vertices: &[u32],
        indices: &[u32],
        max: (usize, usize),
    ) -> (Vec<(usize, usize)>, Vec<[u32; 3]>, bool) {
        let (mut sizes, mut triangles, mut shared) = (Vec::new(), Vec::new(), false);
        for_each_mesh_part(vertices, indices, max, |part_vertices, part_indices| {
            let part_vertices = match part_vertices {
                PartVertices::Shared => {
                    shared = true;
                    vertices
                }
                PartVertices::Own(own) => own,
            };
            assert_eq!(part_indices.len() % 3, 0);
            sizes.push((part_vertices.len(), part_indices.len()));
            triangles.extend(
                part_indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|t| t.map(|i| part_vertices[i as usize])),
            );
            Ok::<(), ()>(())
        })
        .unwrap();
        (sizes, triangles, shared)
    }

    /// 4 × 4 の格子の頂点（値は元の番号）と、18 個の三角形の頂点番号
    fn grid() -> (Vec<u32>, Vec<u32>) {
        let vertices: Vec<u32> = (0..16).collect();
        let mut indices = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                let v = y * 4 + x;
                indices.extend([v, v + 1, v + 5, v, v + 5, v + 4]);
            }
        }
        (vertices, indices)
    }

    #[test]
    fn mesh_counts_fit_in_a_buffer_without_splitting_a_triangle() {
        // 既定の上限 256 MiB に、24 バイトの頂点が 11184810 個、4 バイトの頂点番号が 67108863 個（3 の倍数）入る
        assert_eq!(mesh_counts_per_buffer(256 << 20), (11_184_810, 67_108_863));
        // 三角形 1 つが入らないほど小さくても、少なくとも 1 つずつ置く
        assert_eq!(mesh_counts_per_buffer(0), (3, 3));
        let (vertices, indices) = mesh_counts_per_buffer(u64::MAX);
        assert!(vertices <= u32::MAX as usize && indices <= u32::MAX as usize);
        assert_eq!(indices % 3, 0);
    }

    #[test]
    fn a_mesh_that_fits_is_one_part_sharing_the_vertices() {
        let (vertices, indices) = grid();
        let (sizes, triangles, shared) = split(&vertices, &indices, (16, 54));
        assert_eq!(sizes, vec![(16, 54)]);
        assert!(shared);
        assert_eq!(triangles, indices.as_chunks::<3>().0.to_vec());
    }

    #[test]
    fn a_large_mesh_is_split_into_parts_keeping_every_triangle() {
        let (vertices, indices) = grid();
        let expected: Vec<[u32; 3]> = indices.as_chunks::<3>().0.to_vec();
        // 頂点が収まるときは頂点を共有し、頂点番号だけを区切る。収まらないときは頂点を分ける
        for (max, shares) in [
            ((1000, 9), true),
            ((16, 12), true),
            ((5, 1000), false),
            ((3, 3), false),
            ((7, 12), false),
        ] {
            let (sizes, triangles, shared) = split(&vertices, &indices, max);
            assert!(sizes.len() > 1, "{max:?}");
            assert_eq!(shared, shares, "{max:?}");
            for (part_vertices, part_indices) in sizes {
                assert!(shared || part_vertices <= max.0, "{max:?}");
                assert!(part_indices <= max.1, "{max:?}");
            }
            // 三角形を順に、欠けずに描く
            assert_eq!(triangles, expected, "{max:?}");
        }
    }

    #[test]
    fn triangles_that_cannot_be_drawn_are_left_out_whatever_the_size() {
        let vertices = [10, 11, 12];
        // 範囲の外の頂点番号を持つ三角形と、3 つに満たない最後の頂点番号（収まる大きさでも除く）
        let indices = [0, 1, 2, 0, 1, 9, 2, 1, 0, 0];
        for max in [(3, 6), (100, 100)] {
            let (_, triangles, shared) = split(&vertices, &indices, max);
            assert_eq!(triangles, vec![[10, 11, 12], [12, 11, 10]], "{max:?}");
            assert!(!shared);
        }
    }
}
