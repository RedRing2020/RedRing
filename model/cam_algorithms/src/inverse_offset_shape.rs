//! 逆オフセット形状（工具を逆さまにして製品形状の上で掃引した形。その上側の面が CL の面）
//!
//! 製品形状の三角形メッシュの面・頂点・辺を工具の形でオフセットした要素の集まりとして持つ。工具は 3 次元の向きの
//! オフセット r2 と径の向きのオフセット r1 に分けて扱う（ボールは r1 = 0、フラットは r2 = 0）。
//!
//! | 要素 | ボール（半径 r2） | フラット（半径 r1） |
//! |---|---|---|
//! | 面 | 上向きの法線の向きに r2 ずらした三角形 | 上向きの法線を水平面に射影した向きに r1 ずらした三角形 |
//! | 頂点 | 球面。つながる辺に垂直な平面で切り取った部分 | 頂点の高さの水平な円板 |
//! | 辺 | 円柱面。軸から離れる向きが、隣り合うどの面の側にも向かない部分 | 水平な円板を辺に沿って動かした形 |
//!
//! 形状ポリゴンの位相化（隣接）で、上から見て谷（凹）か平らな辺と、縁でなくつながる辺がすべて谷か平らな頂点を
//! 外す。外した要素は、隣り合う面の要素の下にあり、上側の面を変えない。判定と範囲には面の向き（法線）を使わず、
//! 辺から面の内側へ向かう向き（面の 3 つ目の頂点で決まる）を使う（STL・単体の面は向きがそろうとは限らないため）。
//! 縁の辺・3 つ以上の面の辺・鉛直な辺は判定せず残す。
//!
//! 要素と鉛直線の交点は geo_algorithms の交差演算で、要素の式から求める（要素を三角形に分けない）。

use geo_algorithms::{
    Circle2D, CrossIntersection, CylindricalSurface3D, Direction3D, IntersectionGeometry,
    LineSegment2D, Point2D, Point3D, Ray3D, SphericalSurface3D, Triangle3D, TriangleMesh3D,
    TriangleMeshAdjacency, Vector3D,
};
use geo_contracts::{
    Circle2DConstructor, CylindricalSurface3DConstructor, LineSegment2DConstructor,
    Ray3DConstructor, SphericalSurface3DConstructor, ToleranceSettings, Triangle3DConstructor,
    default_kernel_numerical_zero_tolerance, default_orthogonality_dot_error_tolerance,
};

use crate::inverse_offset::{CutterShape, MeshTriangles, XyBucketIndex, cross, sub};
use crate::solver::CamSolverError;

/// 逆オフセット形状
#[derive(Debug, Clone)]
pub struct InverseOffsetShape {
    shape: CutterShape,
    faces: Vec<Triangle3D<f64>>,
    vertices: Vec<VertexElement>,
    edges: Vec<EdgeElement>,
    /// 要素の XY 範囲の索引（`elements` の添字を返す）
    index: XyBucketIndex,
    elements: Vec<Element>,
    xy_min: [f64; 2],
    xy_max: [f64; 2],
    /// 鉛直線を表す半直線の始点の高さ（すべての要素より下）
    ray_start_z: f64,
}

#[derive(Debug, Clone, Copy)]
enum Element {
    Face(usize),
    Vertex(usize),
    Edge(usize),
}

/// 頂点のオフセット
#[derive(Debug, Clone)]
struct VertexElement {
    point: [f64; 3],
    /// ボールの球面（フラットは `None`）
    sphere: Option<SphericalSurface3D<f64>>,
    /// つながる辺の、もう一方の端点（球面を切り取る平面を決める）
    neighbors: Vec<[f64; 3]>,
}

/// 辺のオフセット
#[derive(Debug, Clone)]
struct EdgeElement {
    ends: [[f64; 3]; 2],
    /// ボールの円柱面（フラットは `None`）
    cylinder: Option<CylindricalSurface3D<f64>>,
    /// 隣り合う面ごとの、辺から面の内側へ向かう向き（辺に垂直）
    into_faces: Vec<[f64; 3]>,
}

impl InverseOffsetShape {
    /// 製品形状の三角形メッシュと工具の形から作る（面積のない三角形は使わない）
    ///
    /// # Errors
    ///
    /// 面積のある三角形がない場合。
    pub fn new(mesh: &TriangleMesh3D<f64>, shape: CutterShape) -> Result<Self, CamSolverError> {
        Self::build(mesh, shape, true)
    }

    /// `exclude_concave` が偽の場合は、凹の辺・凹の頂点も外さない（外しても上側の面が変わらないことの確かめに使う）
    fn build(
        mesh: &TriangleMesh3D<f64>,
        shape: CutterShape,
        exclude_concave: bool,
    ) -> Result<Self, CamSolverError> {
        let MeshTriangles {
            vertices,
            triangles,
            xy_min,
            xy_max,
        } = MeshTriangles::new(mesh)?;
        let radius = shape.radius();
        let normals: Vec<Option<[f64; 3]>> = triangles
            .iter()
            .map(|triangle| upward_unit_normal(triangle.map(|vertex| vertices[vertex])))
            .collect();
        let adjacency = TriangleMeshAdjacency::from_triangles(vertices.len(), &triangles);
        let into_faces: Vec<Vec<[f64; 3]>> = adjacency
            .edges()
            .iter()
            .map(|edge| {
                let [start, end] = edge.vertices;
                let direction = sub(vertices[end], vertices[start]);
                edge.triangles
                    .iter()
                    .filter_map(|&triangle| {
                        let &third = triangles[triangle]
                            .iter()
                            .find(|vertex| !edge.vertices.contains(vertex))?;
                        Some(across(sub(vertices[third], vertices[start]), direction))
                    })
                    .collect()
            })
            .collect();
        let kept_edges: Vec<bool> = adjacency
            .edges()
            .iter()
            .zip(&into_faces)
            .map(|(edge, into_faces)| {
                let [start, end] = edge.vertices.map(|vertex| vertices[vertex]);
                !exclude_concave || !is_valley_or_flat(sub(end, start), into_faces)
            })
            .collect();

        let mut bounds = Vec::new();
        let mut elements = Vec::new();
        let mut faces = Vec::new();
        for (triangle, normal) in triangles.iter().zip(&normals) {
            // 鉛直な面は、XY で面積がないため鉛直線の高さを決めない（辺・頂点で決まる）
            let Some(normal) = normal else { continue };
            let offset = match shape {
                CutterShape::Ball { radius } => normal.map(|component| component * radius),
                CutterShape::Flat { radius } => {
                    let slope = normal[0].hypot(normal[1]);
                    if slope < default_kernel_numerical_zero_tolerance::<f64>() {
                        [0.0; 3]
                    } else {
                        [radius * normal[0] / slope, radius * normal[1] / slope, 0.0]
                    }
                }
            };
            let [a, b, c] = triangle.map(|vertex| {
                let p = vertices[vertex];
                Point3D::new(p[0] + offset[0], p[1] + offset[1], p[2] + offset[2])
            });
            let Some(face) = Triangle3D::new(a, b, c) else {
                continue;
            };
            bounds.push(xy_bounds(&[a, b, c].map(|p| [p.x(), p.y()]), 0.0));
            elements.push(Element::Face(faces.len()));
            faces.push(face);
        }

        let mut vertex_elements = Vec::new();
        for (vertex, &point) in vertices.iter().enumerate() {
            let edges = adjacency.vertex_edges(vertex);
            if edges.is_empty() {
                continue;
            }
            // 縁でなく、つながる辺がすべて凹（または平ら）の頂点は外す
            let interior = edges
                .iter()
                .all(|&edge| adjacency.edges()[edge].triangles.len() == 2);
            if interior && edges.iter().all(|&edge| !kept_edges[edge]) {
                continue;
            }
            let neighbors = edges
                .iter()
                .map(|&edge| {
                    let [first, second] = adjacency.edges()[edge].vertices;
                    vertices[if first == vertex { second } else { first }]
                })
                .collect();
            let sphere = match shape {
                CutterShape::Ball { radius } => Some(
                    SphericalSurface3D::new_standard(point_of(point), radius)
                        .expect("the tool radius is positive"),
                ),
                CutterShape::Flat { .. } => None,
            };
            bounds.push(xy_bounds(&[[point[0], point[1]]], radius));
            elements.push(Element::Vertex(vertex_elements.len()));
            vertex_elements.push(VertexElement {
                point,
                sphere,
                neighbors,
            });
        }

        let mut edge_elements = Vec::new();
        for ((edge, kept), into_faces) in adjacency.edges().iter().zip(&kept_edges).zip(into_faces)
        {
            if !kept {
                continue;
            }
            let ends = edge.vertices.map(|vertex| vertices[vertex]);
            let direction = sub(ends[1], ends[0]);
            let cylinder = match shape {
                CutterShape::Ball { radius } => {
                    // 長さのない辺は、端点の球で決まる
                    let Some(axis) = Direction3D::new(direction[0], direction[1], direction[2])
                    else {
                        continue;
                    };
                    Some(
                        CylindricalSurface3D::new(
                            point_of(ends[0]),
                            axis,
                            perpendicular(axis),
                            radius,
                        )
                        .expect("the tool radius is positive"),
                    )
                }
                CutterShape::Flat { .. } => None,
            };
            bounds.push(xy_bounds(&ends.map(|p| [p[0], p[1]]), radius));
            elements.push(Element::Edge(edge_elements.len()));
            edge_elements.push(EdgeElement {
                ends,
                cylinder,
                into_faces,
            });
        }

        let lowest = vertices.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min);
        Ok(Self {
            shape,
            faces,
            vertices: vertex_elements,
            edges: edge_elements,
            index: XyBucketIndex::from_bounds(&bounds, radius, xy_min, xy_max),
            elements,
            xy_min,
            xy_max,
            ray_start_z: lowest - 2.0 * radius - 1.0,
        })
    }

    pub fn shape(&self) -> CutterShape {
        self.shape
    }

    /// 製品形状の XY 範囲（min, max）
    pub fn xy_bounds(&self) -> ([f64; 2], [f64; 2]) {
        (self.xy_min, self.xy_max)
    }

    /// (x, y) における工具先端の高さ（逆オフセット形状の上側の面の高さから求める）。工具が形状に接触しない場合は
    /// `None`
    pub fn tip_height_at(&self, x: f64, y: f64) -> Option<f64> {
        let ray = Ray3D::new(
            Point3D::new(x, y, self.ray_start_z),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .expect("the vertical direction is not zero");
        let highest = self
            .index
            .candidates(x, y)
            .iter()
            .filter_map(|&element| match self.elements[element] {
                Element::Face(face) => self.face_height(face, &ray),
                Element::Vertex(vertex) => self.vertex_height(vertex, x, y, &ray),
                Element::Edge(edge) => self.edge_height(edge, x, y, &ray),
            })
            .reduce(f64::max)?;
        // 基準点（ボールは球の中心、フラットは底面の中心）から工具先端へ
        Some(highest - self.shape.reference_offset())
    }

    /// (x, y) における工具先端位置（ToolPath 座標）
    pub fn tip_point_at(&self, x: f64, y: f64) -> Option<Point3D<f64>> {
        self.tip_height_at(x, y).map(|z| Point3D::new(x, y, z))
    }

    fn face_height(&self, face: usize, ray: &Ray3D<f64>) -> Option<f64> {
        highest(self.faces[face].intersection(ray, tolerance()), |_| true)
    }

    fn vertex_height(&self, vertex: usize, x: f64, y: f64, ray: &Ray3D<f64>) -> Option<f64> {
        let VertexElement {
            point,
            sphere,
            neighbors,
        } = &self.vertices[vertex];
        match sphere {
            // 球面のうち、つながる辺に垂直な平面（頂点を通る）の、辺と反対の側の部分
            Some(sphere) => highest(sphere.intersection(ray, tolerance()), |center| {
                let outward = sub(center, *point);
                neighbors.iter().all(|&neighbor| {
                    let along = sub(neighbor, *point);
                    dot(outward, along) <= range_slack(outward, along)
                })
            }),
            None => {
                let radius = self.shape.radius();
                ((x - point[0]).hypot(y - point[1]) <= radius).then_some(point[2])
            }
        }
    }

    fn edge_height(&self, edge: usize, x: f64, y: f64, ray: &Ray3D<f64>) -> Option<f64> {
        let EdgeElement {
            ends: [start, end],
            cylinder,
            into_faces,
        } = &self.edges[edge];
        let direction = sub(*end, *start);
        let length_squared = dot(direction, direction);
        match cylinder {
            // 円柱面のうち、軸への垂線の足が辺の中にあり、軸から離れる向きが隣り合うどの面の側にも向かない部分（辺が
            // 最も近い点になる範囲）
            Some(_) if length_squared == 0.0 => None,
            Some(cylinder) => highest(cylinder.intersection(ray, tolerance()), |center| {
                let along = dot(sub(center, *start), direction) / length_squared;
                if !(0.0..=1.0).contains(&along) {
                    return false;
                }
                let foot = [0, 1, 2].map(|k| start[k] + along * direction[k]);
                let outward = sub(center, foot);
                into_faces
                    .iter()
                    .all(|&into_face| dot(outward, into_face) <= range_slack(outward, into_face))
            }),
            // 円板を辺に沿って動かした形: XY に投影した辺と、(x, y) を中心とする半径 r1 の円の交点の高さ。辺の高さは
            // 辺に沿って線形に変わるため、円の内側の部分の最も高い点は交点か端点（端点は頂点の円板で決まる）
            None => {
                let projected = LineSegment2D::new(
                    Point2D::new(start[0], start[1]),
                    Point2D::new(end[0], end[1]),
                )?;
                let circle = Circle2D::new(Point2D::new(x, y), self.shape.radius())
                    .expect("the tool radius is positive");
                let horizontal_squared = direction[0] * direction[0] + direction[1] * direction[1];
                let height = |point: Point2D<f64>| {
                    let along = ((point.x() - start[0]) * direction[0]
                        + (point.y() - start[1]) * direction[1])
                        / horizontal_squared;
                    start[2] + along * direction[2]
                };
                match projected.intersection(&circle, tolerance()).geometry {
                    IntersectionGeometry::Point2D(point) => Some(height(point)),
                    IntersectionGeometry::Points2D(points) => {
                        points.into_iter().map(height).reduce(f64::max)
                    }
                    _ => None,
                }
            }
        }
    }
}

/// 交点のうち、`in_range` を満たす最も高い点の高さ
fn highest(
    result: geo_algorithms::IntersectionResult<f64>,
    in_range: impl Fn([f64; 3]) -> bool,
) -> Option<f64> {
    let coordinates = |point: &Point3D<f64>| [point.x(), point.y(), point.z()];
    let points: Vec<[f64; 3]> = match result.geometry {
        IntersectionGeometry::Point(point) => vec![coordinates(&point)],
        IntersectionGeometry::Points(points) => points.iter().map(coordinates).collect(),
        _ => Vec::new(),
    };
    points
        .into_iter()
        .filter(|&point| in_range(point))
        .map(|point| point[2])
        .reduce(f64::max)
}

/// 交差演算の許容誤差
fn tolerance() -> f64 {
    default_kernel_numerical_zero_tolerance::<f64>()
}

/// 範囲の判定（内積が 0 以下）の、向きの長さに比べた余裕（境目では、隣の要素が同じ高さを決める）
fn range_slack(first: [f64; 3], second: [f64; 3]) -> f64 {
    default_orthogonality_dot_error_tolerance::<f64>()
        * dot(first, first).sqrt()
        * dot(second, second).sqrt()
}

/// 上向き（z >= 0 側）の単位法線。鉛直な面は `None`
fn upward_unit_normal([a, b, c]: [[f64; 3]; 3]) -> Option<[f64; 3]> {
    let n = cross(sub(b, a), sub(c, a));
    let norm = dot(n, n).sqrt();
    let sign = if n[2] < 0.0 { -1.0 } else { 1.0 };
    let n = n.map(|component| sign * component / norm);
    (n[2] >= default_orthogonality_dot_error_tolerance::<f64>()).then_some(n)
}

/// 向き `direction` の辺が、上から見て谷（凹）か平らか（隣り合う面がちょうど 2 つの場合だけ判定し、それ以外は偽）
///
/// 辺に垂直な断面で、2 つの面の向き `into_faces`（辺から面の内側へ）を真上の向きから測った角が、真上の両側にあり、
/// 和が 180° 以下なら谷か平ら。わずかでも 180° を超える辺（尾根）は残す（2 つの面の要素は辺から離れる向きにずれ、
/// 間を辺の要素が埋めるため。フラットは面ごとに工具の半径だけずれるため、ほぼ平らな尾根でも間が大きく開く）。
/// 鉛直な辺と、真上・真下を向く面がある辺は判定せず偽とする。
fn is_valley_or_flat(direction: [f64; 3], into_faces: &[[f64; 3]]) -> bool {
    let [first, second] = into_faces else {
        return false;
    };
    let up = across([0.0, 0.0, 1.0], direction);
    let angle_tolerance = ToleranceSettings::<f64>::standard().angle_tolerance;
    if dot(up, up).sqrt() < angle_tolerance {
        return false;
    }
    // 真上からの角（0 から 180°）と、真上のどちらの側か
    let measure = |into_face: [f64; 3]| {
        let side = cross(up, into_face);
        let angle = dot(side, side).sqrt().atan2(dot(up, into_face));
        (angle, dot(side, direction))
    };
    let ((first_angle, first_side), (second_angle, second_side)) =
        (measure(*first), measure(*second));
    let straight =
        |angle: f64| angle < angle_tolerance || angle > std::f64::consts::PI - angle_tolerance;
    if straight(first_angle) || straight(second_angle) {
        return false;
    }
    first_side * second_side < 0.0 && first_angle + second_angle <= std::f64::consts::PI
}

/// `vector` の、向き `direction` に垂直な成分
fn across(vector: [f64; 3], direction: [f64; 3]) -> [f64; 3] {
    let scale = dot(vector, direction) / dot(direction, direction);
    [0, 1, 2].map(|k| vector[k] - scale * direction[k])
}

/// 軸 `axis` に垂直な向き
fn perpendicular(axis: Direction3D<f64>) -> Direction3D<f64> {
    let a = [axis.x(), axis.y(), axis.z()];
    let helper = if a[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let p = cross(a, helper);
    Direction3D::new(p[0], p[1], p[2]).expect("the helper axis is not parallel to the edge")
}

/// 点の列の XY 範囲を `margin` だけ広げたもの
fn xy_bounds(points: &[[f64; 2]], margin: f64) -> ([f64; 2], [f64; 2]) {
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for point in points {
        for k in 0..2 {
            min[k] = min[k].min(point[k] - margin);
            max[k] = max[k].max(point[k] + margin);
        }
    }
    (min, max)
}

fn point_of(p: [f64; 3]) -> Point3D<f64> {
    Point3D::new(p[0], p[1], p[2])
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests;
