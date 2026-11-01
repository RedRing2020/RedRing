//! 三角形メッシュの隣接（辺ごとの三角形・頂点につながる辺）

use std::collections::HashMap;

/// 三角形メッシュの 1 本の辺
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleMeshEdge {
    /// 辺の両端の頂点番号（小さい順）
    pub vertices: [usize; 2],
    /// この辺を持つ三角形の番号（1 つなら縁の辺、3 つ以上なら多様体でない辺）
    pub triangles: Vec<usize>,
}

/// 三角形メッシュの隣接
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleMeshAdjacency {
    edges: Vec<TriangleMeshEdge>,
    /// 頂点ごとの、つながる辺の番号
    vertex_edges: Vec<Vec<usize>>,
}

impl TriangleMeshAdjacency {
    /// 頂点の数 `vertex_count` と三角形の頂点番号の列 `triangles` から作る（辺は、三角形の順に最初に現れた順に並ぶ）
    ///
    /// # Panics
    ///
    /// 三角形の頂点番号が `vertex_count` 以上の場合。
    pub fn from_triangles(vertex_count: usize, triangles: &[[usize; 3]]) -> Self {
        let mut edges: Vec<TriangleMeshEdge> = Vec::new();
        let mut vertex_edges = vec![Vec::new(); vertex_count];
        let mut index: HashMap<[usize; 2], usize> = HashMap::new();
        for (triangle, corners) in triangles.iter().enumerate() {
            for k in 0..3 {
                let (a, b) = (corners[k], corners[(k + 1) % 3]);
                let vertices = [a.min(b), a.max(b)];
                let edge = *index.entry(vertices).or_insert_with(|| {
                    edges.push(TriangleMeshEdge {
                        vertices,
                        triangles: Vec::new(),
                    });
                    let edge = edges.len() - 1;
                    for vertex in vertices {
                        vertex_edges[vertex].push(edge);
                    }
                    edge
                });
                edges[edge].triangles.push(triangle);
            }
        }
        Self {
            edges,
            vertex_edges,
        }
    }

    /// すべての辺
    pub fn edges(&self) -> &[TriangleMeshEdge] {
        &self.edges
    }

    /// 頂点 `vertex` につながる辺の番号（`edges` の添字）
    ///
    /// # Panics
    ///
    /// `vertex` が頂点の数以上の場合。
    pub fn vertex_edges(&self, vertex: usize) -> &[usize] {
        &self.vertex_edges[vertex]
    }
}

#[cfg(test)]
mod tests {
    use super::TriangleMeshAdjacency;

    /// 辺 (1, 2) を共有する 2 つの三角形と、同じ辺に付いた 3 つ目の三角形
    const TRIANGLES: [[usize; 3]; 3] = [[0, 1, 2], [2, 1, 3], [1, 2, 4]];

    #[test]
    fn shared_edges_list_every_triangle_and_vertices_list_their_edges() {
        let adjacency = TriangleMeshAdjacency::from_triangles(6, &TRIANGLES[..2]);
        let edges = adjacency.edges();
        assert_eq!(edges.len(), 5);
        let shared = edges.iter().find(|edge| edge.vertices == [1, 2]).unwrap();
        assert_eq!(shared.triangles, vec![0, 1]);
        // 共有しない辺は縁の辺
        assert_eq!(
            edges
                .iter()
                .filter(|edge| edge.triangles.len() == 1)
                .count(),
            4
        );
        let around_one: Vec<[usize; 2]> = adjacency
            .vertex_edges(1)
            .iter()
            .map(|&edge| edges[edge].vertices)
            .collect();
        assert_eq!(around_one, vec![[0, 1], [1, 2], [1, 3]]);
        // 三角形に使われていない頂点
        assert!(adjacency.vertex_edges(5).is_empty());
    }

    #[test]
    fn an_edge_of_three_triangles_lists_all_of_them() {
        let adjacency = TriangleMeshAdjacency::from_triangles(5, &TRIANGLES);
        let shared = adjacency
            .edges()
            .iter()
            .find(|edge| edge.vertices == [1, 2])
            .unwrap();
        assert_eq!(shared.triangles, vec![0, 1, 2]);
    }
}
