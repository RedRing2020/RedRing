//! 整数座標の多角形（穴を含む）の三角形分割
//!
//! 耳刈り法（Triangulation by Ear Clipping）で分ける。穴は、穴の最も左の頂点から左へ伸ばした線が最初に当たる
//! 外周の頂点と橋でつなぎ、1 つの輪にしてから分ける。
//!
//! 一直線に並ぶ頂点（隣の面と共有する頂点）は取り除かずに三角形の頂点として残す。耳の三角形の中や辺の上に
//! ほかの頂点があれば、その耳は切らない（三角形の辺の途中に頂点が残らないようにする）。

/// 輪の頂点（頂点の番号と座標、前後の頂点の位置）
#[derive(Debug, Clone, Copy)]
struct Node {
    id: usize,
    point: [i64; 2],
    prev: usize,
    next: usize,
}

/// `outer`（左回り）と `holes`（右回り）の頂点の番号の輪を三角形に分ける（三角形は左回り）
///
/// 分けられない場合は `None`。
pub(super) fn triangulate(
    points: &[[i64; 2]],
    outer: &[usize],
    holes: &[Vec<usize>],
) -> Option<Vec<[usize; 3]>> {
    let mut nodes = Vec::new();
    let outer_start = link_ring(&mut nodes, points, outer)?;
    let mut hole_starts: Vec<usize> = holes
        .iter()
        .map(|hole| link_ring(&mut nodes, points, hole).map(|start| leftmost(&nodes, start)))
        .collect::<Option<_>>()?;
    hole_starts.sort_by_key(|&start| nodes[start].point);
    for hole in hole_starts {
        let bridge = find_hole_bridge(&nodes, hole, outer_start)?;
        split_polygon(&mut nodes, bridge, hole);
    }
    let start = remove_duplicates(&mut nodes, outer_start);
    clip_ears(&mut nodes, start)
}

fn link_ring(nodes: &mut Vec<Node>, points: &[[i64; 2]], ring: &[usize]) -> Option<usize> {
    if ring.len() < 3 {
        return None;
    }
    let first = nodes.len();
    for (index, &id) in ring.iter().enumerate() {
        nodes.push(Node {
            id,
            point: points[id],
            prev: first + (index + ring.len() - 1) % ring.len(),
            next: first + (index + 1) % ring.len(),
        });
    }
    Some(first)
}

/// 輪の中で最も左（同じなら最も下）の頂点
fn leftmost(nodes: &[Node], start: usize) -> usize {
    let mut best = start;
    let mut node = nodes[start].next;
    while node != start {
        if nodes[node].point < nodes[best].point {
            best = node;
        }
        node = nodes[node].next;
    }
    best
}

/// 2 倍の符号付き面積（左回りが正）
fn area(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let (a, b, c) = (
        [i128::from(a[0]), i128::from(a[1])],
        [i128::from(b[0]), i128::from(b[1])],
        [i128::from(c[0]), i128::from(c[1])],
    );
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// 点 `p` が左回りの三角形 `abc` の中か辺の上にあるか
fn in_triangle(a: [i64; 2], b: [i64; 2], c: [i64; 2], p: [i64; 2]) -> bool {
    area(a, b, p) >= 0 && area(b, c, p) >= 0 && area(c, a, p) >= 0
}

/// 頂点 `ear` が耳か（凸で、三角形の中と辺の上にほかの頂点がない）
fn is_ear(nodes: &[Node], ear: usize) -> bool {
    let Node { prev, next, .. } = nodes[ear];
    let (a, b, c) = (nodes[prev], nodes[ear], nodes[next]);
    if area(a.point, b.point, c.point) <= 0 {
        return false;
    }
    let mut node = c.next;
    while node != prev {
        let p = nodes[node];
        if ![a.id, b.id, c.id].contains(&p.id) && in_triangle(a.point, b.point, c.point, p.point) {
            return false;
        }
        node = p.next;
    }
    true
}

fn remove(nodes: &mut [Node], node: usize) {
    let Node { prev, next, .. } = nodes[node];
    nodes[prev].next = next;
    nodes[next].prev = prev;
}

/// 同じ頂点が続く場合に 1 つにする（残った頂点を返す）
fn remove_duplicates(nodes: &mut [Node], start: usize) -> usize {
    let mut start = start;
    let mut node = start;
    loop {
        let next = nodes[node].next;
        if next != node && nodes[next].id == nodes[node].id {
            remove(nodes, next);
            if next == start {
                start = node;
            }
            continue;
        }
        node = next;
        if node == start {
            return start;
        }
    }
}

/// 耳を切り取っていき、三角形の列にする（面積のない輪が残った場合も終わる）
fn clip_ears(nodes: &mut [Node], start: usize) -> Option<Vec<[usize; 3]>> {
    let mut triangles = Vec::new();
    let mut ear = start;
    let mut stop = ear;
    loop {
        let Node { prev, next, .. } = nodes[ear];
        if prev == next {
            return Some(triangles);
        }
        if is_ear(nodes, ear) {
            triangles.push([nodes[prev].id, nodes[ear].id, nodes[next].id]);
            remove(nodes, ear);
            ear = nodes[next].next;
            stop = ear;
            continue;
        }
        ear = next;
        if ear == stop {
            return (ring_area(nodes, ear) == 0).then_some(triangles);
        }
    }
}

fn ring_area(nodes: &[Node], start: usize) -> i128 {
    let mut sum = 0;
    let mut node = start;
    loop {
        let next = nodes[node].next;
        sum += area([0, 0], nodes[node].point, nodes[next].point);
        node = next;
        if node == start {
            return sum;
        }
    }
}

/// 穴の頂点 `hole` から左へ伸ばした線で見える外周の頂点
fn find_hole_bridge(nodes: &[Node], hole: usize, outer: usize) -> Option<usize> {
    let [hx, hy] = nodes[hole].point;
    let (hx_f, hy_f) = (hx as f64, hy as f64);
    let mut qx = f64::NEG_INFINITY;
    let mut candidate = None;
    let mut node = outer;
    loop {
        let (p, next) = (nodes[node], nodes[nodes[node].next]);
        if hy <= p.point[1] && hy >= next.point[1] && next.point[1] != p.point[1] {
            let x = p.point[0] as f64
                + (hy_f - p.point[1] as f64) * (next.point[0] - p.point[0]) as f64
                    / (next.point[1] - p.point[1]) as f64;
            if x <= hx_f && x > qx {
                qx = x;
                let m = if p.point[0] < next.point[0] {
                    node
                } else {
                    p.next
                };
                if x == hx_f {
                    return Some(m);
                }
                candidate = Some(m);
            }
        }
        node = p.next;
        if node == outer {
            break;
        }
    }
    let mut m = candidate?;
    // 線と候補の頂点の間の三角形の中に頂点があれば、線との角度が最も小さいものを選ぶ
    let stop = m;
    let [mx, my] = nodes[m].point;
    let mut tan_min = f64::INFINITY;
    let mut node = m;
    loop {
        let p = nodes[node];
        let [px, py] = p.point;
        let (ax, cx) = if hy < my { (hx_f, qx) } else { (qx, hx_f) };
        if hx >= px
            && px >= mx
            && hx != px
            && in_triangle_f64(
                [ax, hy_f],
                [mx as f64, my as f64],
                [cx, hy_f],
                [px as f64, py as f64],
            )
        {
            let tan = (hy - py).abs() as f64 / (hx - px) as f64;
            if locally_inside(nodes, node, hole)
                && (tan < tan_min
                    || (tan == tan_min
                        && (px > nodes[m].point[0]
                            || (px == nodes[m].point[0]
                                && sector_contains_sector(nodes, m, node)))))
            {
                m = node;
                tan_min = tan;
            }
        }
        node = p.next;
        if node == stop {
            return Some(m);
        }
    }
}

fn in_triangle_f64(a: [f64; 2], b: [f64; 2], c: [f64; 2], p: [f64; 2]) -> bool {
    (c[0] - p[0]) * (a[1] - p[1]) >= (a[0] - p[0]) * (c[1] - p[1])
        && (a[0] - p[0]) * (b[1] - p[1]) >= (b[0] - p[0]) * (a[1] - p[1])
        && (b[0] - p[0]) * (c[1] - p[1]) >= (c[0] - p[0]) * (b[1] - p[1])
}

/// 頂点 `a` から `b` への対角線が、`a` の周りで多角形の内側を通るか
fn locally_inside(nodes: &[Node], a: usize, b: usize) -> bool {
    let (prev, point, next) = (
        nodes[nodes[a].prev].point,
        nodes[a].point,
        nodes[nodes[a].next].point,
    );
    let target = nodes[b].point;
    if area(prev, point, next) > 0 {
        area(point, next, target) >= 0 && area(point, target, prev) >= 0
    } else {
        area(point, target, next) < 0 || area(point, prev, target) < 0
    }
}

/// 頂点 `m` の角の範囲が、頂点 `p` の角の範囲を含むか
fn sector_contains_sector(nodes: &[Node], m: usize, p: usize) -> bool {
    let (m_prev, m_point, m_next) = (
        nodes[nodes[m].prev].point,
        nodes[m].point,
        nodes[nodes[m].next].point,
    );
    let (p_prev, p_next) = (nodes[nodes[p].prev].point, nodes[nodes[p].next].point);
    area(m_prev, m_point, p_prev) > 0 && area(p_next, m_point, m_next) > 0
}

/// 頂点 `a` と `b` を橋でつなぎ、輪を 1 つにする（`a` と `b` の複製を作る）
fn split_polygon(nodes: &mut Vec<Node>, a: usize, b: usize) {
    let a2 = nodes.len();
    let b2 = a2 + 1;
    let (an, bp) = (nodes[a].next, nodes[b].prev);
    nodes.push(nodes[a]);
    nodes.push(nodes[b]);
    nodes[a].next = b;
    nodes[b].prev = a;
    nodes[a2].next = an;
    nodes[an].prev = a2;
    nodes[b2].next = a2;
    nodes[a2].prev = b2;
    nodes[bp].next = b2;
    nodes[b2].prev = bp;
}
