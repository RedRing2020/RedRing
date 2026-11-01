//! 三角形メッシュの隣接（辺ごとの三角形・頂点につながる辺）と、三角形の向きをそろえること

use std::collections::{HashMap, VecDeque};

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
    /// 三角形ごとの、頂点 0→1・1→2・2→0 の辺の番号
    triangle_edges: Vec<[usize; 3]>,
}

/// 三角形の向きをそろえた結果（メッシュは書き換えない）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleMeshOrientation {
    /// 三角形ごとの、頂点の順を逆にするか
    pub reversed: Vec<bool>,
    /// 三角形ごとの、つながった塊の番号（面がちょうど 2 つの辺を渡ってたどれる三角形の集まり）
    pub components: Vec<usize>,
    /// 塊ごとの、向きをそろえられたか（メビウスの帯のように、たどる道によって向きが食い違う塊は偽）
    pub orientable: Vec<bool>,
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
        let mut triangle_edges = vec![[0; 3]; triangles.len()];
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
                triangle_edges[triangle][k] = edge;
            }
        }
        Self {
            edges,
            vertex_edges,
            triangle_edges,
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

    /// 三角形 `triangles`（隣接を作ったものと同じ）の向きを、塊ごとにそろえる
    ///
    /// 面がちょうど 2 つの辺で、2 つの三角形が辺を逆の向きにたどるように、塊の最初の三角形から順に向きを決める
    /// （最初の三角形は逆にしない）。縁の辺・3 つ以上の面の辺は渡らない。
    ///
    /// # Panics
    ///
    /// `triangles` が隣接を作った三角形と数が違う場合。
    pub fn orient(&self, triangles: &[[usize; 3]]) -> TriangleMeshOrientation {
        assert_eq!(
            triangles.len(),
            self.triangle_edges.len(),
            "the triangles must be those of the adjacency"
        );
        // 三角形 `triangle` が、向き `reversed` のとき、辺 `k`（頂点 k→k+1）を辺の頂点番号の小さい順にたどるか
        let forward = |triangle: usize, k: usize, reversed: bool| {
            let corners = triangles[triangle];
            (corners[k] < corners[(k + 1) % 3]) != reversed
        };
        let mut reversed = vec![false; triangles.len()];
        let mut components = vec![usize::MAX; triangles.len()];
        let mut orientable = Vec::new();
        let mut queue = VecDeque::new();
        for seed in 0..triangles.len() {
            if components[seed] != usize::MAX {
                continue;
            }
            let component = orientable.len();
            orientable.push(true);
            components[seed] = component;
            queue.push_back(seed);
            while let Some(triangle) = queue.pop_front() {
                for (k, &edge) in self.triangle_edges[triangle].iter().enumerate() {
                    let [first, second] = self.edges[edge].triangles[..] else {
                        continue;
                    };
                    let neighbor = if first == triangle { second } else { first };
                    let k_neighbor = self.triangle_edges[neighbor]
                        .iter()
                        .position(|&other| other == edge)
                        .expect("the neighbor has the shared edge");
                    // 隣は、辺を逆の向きにたどる
                    let along = forward(triangle, k, reversed[triangle]);
                    let neighbor_reversed = forward(neighbor, k_neighbor, false) == along;
                    if components[neighbor] == usize::MAX {
                        components[neighbor] = component;
                        reversed[neighbor] = neighbor_reversed;
                        queue.push_back(neighbor);
                    } else if reversed[neighbor] != neighbor_reversed {
                        orientable[component] = false;
                    }
                }
            }
        }
        TriangleMeshOrientation {
            reversed,
            components,
            orientable,
        }
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

    /// 三角形 `triangle` を、向き `reversed` で頂点の順にたどった辺（向き付き）
    fn directed_edges(triangle: [usize; 3], reversed: bool) -> [[usize; 2]; 3] {
        let [a, b, c] = if reversed {
            [triangle[0], triangle[2], triangle[1]]
        } else {
            triangle
        };
        [[a, b], [b, c], [c, a]]
    }

    #[test]
    fn orienting_makes_shared_edges_run_in_opposite_directions() {
        // 3 × 3 の格子の 2 つの正方形を三角形に分け、1 つおきに頂点の順を逆にしたもの
        let triangles = [
            [0, 1, 4],
            [0, 3, 4],
            [1, 2, 5],
            [1, 5, 4],
            [3, 4, 7],
            [3, 6, 7],
        ];
        let adjacency = TriangleMeshAdjacency::from_triangles(8, &triangles);
        let orientation = adjacency.orient(&triangles);
        assert_eq!(orientation.components, vec![0; 6]);
        assert_eq!(orientation.orientable, vec![true]);
        assert!(!orientation.reversed[0]);
        for edge in adjacency.edges() {
            let [first, second] = edge.triangles[..] else {
                continue;
            };
            let runs = |triangle: usize| {
                directed_edges(triangles[triangle], orientation.reversed[triangle])
                    .into_iter()
                    .find(|directed| {
                        directed[0].min(directed[1]) == edge.vertices[0]
                            && directed[0].max(directed[1]) == edge.vertices[1]
                    })
                    .unwrap()
            };
            assert_eq!(runs(first), [runs(second)[1], runs(second)[0]]);
        }
    }

    #[test]
    fn separate_pieces_are_separate_components_and_a_mobius_strip_cannot_be_oriented() {
        // 5 つの三角形を輪にしたメビウスの帯と、離れた三角形 1 つ
        let triangles = [
            [0, 1, 2],
            [1, 2, 3],
            [2, 3, 4],
            [3, 4, 0],
            [4, 0, 1],
            [5, 6, 7],
        ];
        let adjacency = TriangleMeshAdjacency::from_triangles(8, &triangles);
        let orientation = adjacency.orient(&triangles);
        assert_eq!(orientation.components, vec![0, 0, 0, 0, 0, 1]);
        assert_eq!(orientation.orientable, vec![false, true]);
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
