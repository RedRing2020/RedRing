//! 掃引プリミティブの包含箱の BVH（一括で構築し、包含箱の中心の中央値で分割する）

use geo_algorithms::{Aabb3D, Point3D};

/// 葉に置く包含箱の数の上限
const LEAF_SIZE: usize = 4;

#[derive(Debug, Clone)]
enum Node {
    /// `order[first..first + count]` の包含箱を持つ葉
    Leaf {
        bounds: Aabb3D<f64>,
        first: usize,
        count: usize,
    },
    /// 2 つの子（`nodes` の添字）を持つ節
    Branch {
        bounds: Aabb3D<f64>,
        left: usize,
        right: usize,
    },
}

impl Node {
    fn bounds(&self) -> &Aabb3D<f64> {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => bounds,
        }
    }
}

/// 包含箱の列に対する BVH（包含箱の添字を返す）
#[derive(Debug, Clone, Default)]
pub(super) struct BoundingVolumeHierarchy {
    nodes: Vec<Node>,
    /// 葉が参照する包含箱の添字の並び
    order: Vec<usize>,
}

impl BoundingVolumeHierarchy {
    pub(super) fn build(bounds: &[Aabb3D<f64>]) -> Self {
        let mut hierarchy = Self {
            nodes: Vec::new(),
            order: (0..bounds.len()).collect(),
        };
        if !bounds.is_empty() {
            hierarchy.build_node(bounds, 0, bounds.len());
        }
        hierarchy
    }

    /// `order[first..end]` の節を作り、`nodes` の添字を返す
    fn build_node(&mut self, bounds: &[Aabb3D<f64>], first: usize, end: usize) -> usize {
        let node_bounds = self.order[first..end]
            .iter()
            .map(|&index| bounds[index])
            .reduce(|a, b| {
                Aabb3D::from_points(&[a.min(), a.max(), b.min(), b.max()])
                    .expect("the corners of two boxes are not empty")
            })
            .expect("a node covers at least one bounding box");
        let count = end - first;
        if count <= LEAF_SIZE {
            self.nodes.push(Node::Leaf {
                bounds: node_bounds,
                first,
                count,
            });
            return self.nodes.len() - 1;
        }

        // 包含箱の中心の範囲が最も広い軸で、中央値で 2 つに分ける
        let center = |index: usize| {
            let center = bounds[index].center();
            [center.x(), center.y(), center.z()]
        };
        let mut low = [f64::INFINITY; 3];
        let mut high = [f64::NEG_INFINITY; 3];
        for &index in &self.order[first..end] {
            for (axis, value) in center(index).into_iter().enumerate() {
                low[axis] = low[axis].min(value);
                high[axis] = high[axis].max(value);
            }
        }
        let axis = (0..3)
            .max_by(|&a, &b| (high[a] - low[a]).total_cmp(&(high[b] - low[b])))
            .expect("three axes");
        let middle = first + count / 2;
        self.order[first..end].select_nth_unstable_by(count / 2, |&a, &b| {
            center(a)[axis].total_cmp(&center(b)[axis])
        });

        let slot = self.nodes.len();
        self.nodes.push(Node::Leaf {
            bounds: node_bounds,
            first,
            count: 0,
        });
        let left = self.build_node(bounds, first, middle);
        let right = self.build_node(bounds, middle, end);
        self.nodes[slot] = Node::Branch {
            bounds: node_bounds,
            left,
            right,
        };
        slot
    }

    /// 点を含む包含箱のうち、`predicate` を満たすものがあるか
    pub(super) fn any_containing(
        &self,
        point: Point3D<f64>,
        mut predicate: impl FnMut(usize) -> bool,
    ) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let mut stack = vec![0];
        while let Some(node) = stack.pop() {
            let node = &self.nodes[node];
            if !node.bounds().contains_point(point) {
                continue;
            }
            match *node {
                Node::Leaf { first, count, .. } => {
                    if self.order[first..first + count]
                        .iter()
                        .any(|&index| predicate(index))
                    {
                        return true;
                    }
                }
                Node::Branch { left, right, .. } => stack.extend([left, right]),
            }
        }
        false
    }
}
