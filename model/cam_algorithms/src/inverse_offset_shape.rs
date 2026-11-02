//! 逆オフセット形状（工具を逆さまにして製品形状の上で掃引した形。その上側の面が CL の面）
//!
//! 製品形状の三角形メッシュの面・頂点・辺を工具の形でオフセットした要素の集まりとして持つ。工具は 3 次元の向きの
//! オフセット r2 と径の向きのオフセット r1 に分けて扱う（ボールは r1 = 0、フラットは r2 = 0）。
//!
//! | 要素 | ボール（半径 r2） | フラット（半径 r1） |
//! |---|---|---|
//! | 面 | 外向きの法線の向きに r2 ずらした三角形 | 外向きの法線を水平面に射影した向きに r1 ずらした三角形 |
//! | 頂点 | 球面。つながる辺に垂直な平面で切り取った部分 | 頂点の高さの水平な円板 |
//! | 辺 | 円柱面。軸から離れる向きが、隣り合うどの面の側にも向かない部分 | 水平な円板を辺に沿って動かした形 |
//!
//! 形状ポリゴンの位相化（隣接）で、三角形の向きを塊ごとにそろえ、外（材料のない側）を決める。外は、塊の中の上から
//! 届く面（上側の包絡に出ている面）が上を向く向きとする（STL の頂点の順は外向きとは限らないため、形から決める）。
//! 外向きの法線で凹か平らな辺と、縁でなくつながる辺がすべて凹か平らな頂点を外す。外した要素は、隣り合う面の要素に
//! 覆われ、逆オフセット形状の面に出ない。向きをそろえられない塊・上から届く面がない塊は外を決められないため、面ごとに
//! 上向きにそろえ、辺・頂点を外さない。下を向く面は、閉じた塊（縁のない塊）では真上に塊の材料と上を向く面があり
//! 上から届かないため、高さを求める索引に入れない。開いた塊では面の両側から工具が届くことがあるため、上側に
//! ずらした要素も索引に入れ、下を向く面に隣り合う辺は外さない（隠れている面の上側の要素は、上から先に当たる面より
//! 低いため、高さを変えない）。辺の範囲は、辺から面の内側へ向かう向き（面の 3 つ目の頂点で決まる）で決める
//! （凸の辺では、外向きの法線の間と同じ範囲）。
//!
//! 要素と鉛直線の交点は geo_algorithms の交差演算で、要素の式から求める（要素を三角形に分けない）。

use geo_algorithms::{
    Circle2D, CrossIntersection, CylindricalSurface3D, Direction3D, IntersectionGeometry,
    LineSegment2D, Point2D, Point3D, Ray3D, SphericalSurface3D, Triangle3D, TriangleMesh3D,
    TriangleMeshAdjacency, Vector3D,
};
use geo_contracts::{
    Circle2DConstructor, CylindricalSurface3DConstructor, LineSegment2DConstructor,
    Ray3DConstructor, SphericalSurface3DConstructor, ToleranceSettings, Triangle3DBoundaryAccess,
    Triangle3DConstructor, default_kernel_numerical_zero_tolerance,
    default_orthogonality_dot_error_tolerance,
};

use crate::inverse_offset::{CutterShape, MeshTriangles, XyBucketIndex, cross, sub};
use crate::product_surface::ProductSurface;
use crate::solver::CamSolverError;

pub use display::{InverseOffsetPart, InverseOffsetPartKind, MIN_DISPLAY_ANGLE_STEP};

/// 逆オフセット形状
#[derive(Debug, Clone)]
pub struct InverseOffsetShape {
    shape: CutterShape,
    faces: Vec<Triangle3D<f64>>,
    /// 面ごとの役割（高さを求める索引に入るか、外を決められなかった塊の面か）
    face_kinds: Vec<InverseOffsetPartKind>,
    /// 開いた塊の下を向く面を、上側（外向きと逆の向き）にずらした三角形
    upper_sides: Vec<Triangle3D<f64>>,
    vertices: Vec<VertexElement>,
    edges: Vec<EdgeElement>,
    /// 要素の XY 範囲の索引（`elements` の添字を返す）
    index: XyBucketIndex,
    elements: Vec<Element>,
    /// 要素ごとの、最も低い点の高さ（鉛直線を表す半直線の始点を、要素ごとにそのすぐ下に置くため）
    lowest: Vec<f64>,
    xy_min: [f64; 2],
    xy_max: [f64; 2],
}

#[derive(Debug, Clone, Copy)]
enum Element {
    Face(usize),
    UpperSide(usize),
    Vertex(usize),
    Edge(usize),
}

/// 三角形の外の決まり方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outward {
    /// 外を決められない（面ごとに上向きにそろえた）
    Unknown,
    /// 閉じた塊（どの辺にも面がちょうど 2 つある塊）で外を決めた
    Closed,
    /// 開いた塊で外を決めた
    Open,
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
    /// 工具の半径が正の有限値でない場合と、面積のある三角形がない場合。
    pub fn new(mesh: &TriangleMesh3D<f64>, shape: CutterShape) -> Result<Self, CamSolverError> {
        Self::build(mesh, shape, true)
    }

    /// `exclude_concave` が偽の場合は、凹の辺・凹の頂点も外さない（外しても上側の面が変わらないことの確かめに使う）
    fn build(
        mesh: &TriangleMesh3D<f64>,
        shape: CutterShape,
        exclude_concave: bool,
    ) -> Result<Self, CamSolverError> {
        let radius = shape.radius();
        if !(radius.is_finite() && radius > 0.0) {
            return Err(CamSolverError::InvalidInput(
                "inverse offset shape requires a positive finite cutter radius".to_string(),
            ));
        }
        let MeshTriangles {
            vertices,
            triangles,
            xy_min,
            xy_max,
        } = MeshTriangles::new(mesh)?;
        let adjacency = TriangleMeshAdjacency::from_triangles(vertices.len(), &triangles);
        let (triangles, outward) = orient_outward(mesh, &adjacency, triangles, &vertices)?;
        let normals: Vec<[f64; 3]> = triangles
            .iter()
            .map(|triangle| unit_normal(triangle.map(|vertex| vertices[vertex])))
            .collect();
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
            .map(|edge| {
                !exclude_concave
                    || !is_concave_or_flat(edge, &triangles, &vertices, &normals, &outward)
            })
            .collect();

        let mut bounds = Vec::new();
        let mut elements = Vec::new();
        let mut lowest = Vec::new();
        let mut faces = Vec::new();
        let mut face_kinds = Vec::new();
        let mut upper_sides = Vec::new();
        let flat = default_orthogonality_dot_error_tolerance::<f64>();
        for ((triangle, normal), &outward) in triangles.iter().zip(&normals).zip(&outward) {
            let corners = triangle.map(|vertex| vertices[vertex]);
            let Some(face) = offset_triangle(corners, *normal, shape) else {
                continue;
            };
            // 上を向かない面（鉛直な面・アンダーカットの面）の要素は、高さを求める索引に入れない
            if normal[2] >= flat {
                let element = Element::Face(faces.len());
                push_face(&face, element, &mut bounds, &mut elements, &mut lowest);
            }
            faces.push(face);
            face_kinds.push(match outward {
                Outward::Unknown => InverseOffsetPartKind::UnorientedFace,
                _ if normal[2] >= flat => InverseOffsetPartKind::IndexedFace,
                _ => InverseOffsetPartKind::UnindexedFace,
            });
            // 開いた塊の下を向く面は、上から届くことがあるため、上側にずらした要素を索引に入れる
            if outward == Outward::Open && normal[2] <= -flat {
                let reversed = normal.map(|component| -component);
                if let Some(upper) = offset_triangle(corners, reversed, shape) {
                    let element = Element::UpperSide(upper_sides.len());
                    push_face(&upper, element, &mut bounds, &mut elements, &mut lowest);
                    upper_sides.push(upper);
                }
            }
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
            lowest.push(point[2] - radius);
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
            lowest.push(ends[0][2].min(ends[1][2]) - radius);
            edge_elements.push(EdgeElement {
                ends,
                cylinder,
                into_faces,
            });
        }

        Ok(Self {
            shape,
            faces,
            face_kinds,
            upper_sides,
            vertices: vertex_elements,
            edges: edge_elements,
            index: XyBucketIndex::from_bounds(&bounds, radius, xy_min, xy_max),
            elements,
            lowest,
            xy_min,
            xy_max,
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
        let highest = self
            .index
            .candidates(x, y)
            .iter()
            .filter_map(|&element| {
                // 半直線の始点は要素ごとに、その要素のすぐ下に置く（ほかの要素の高さで、交点の高さが桁落ちしない）
                let ray = Ray3D::new(
                    Point3D::new(x, y, self.lowest[element] - 1.0),
                    Vector3D::new(0.0, 0.0, 1.0),
                )
                .expect("the vertical direction is not zero");
                match self.elements[element] {
                    Element::Face(face) => face_height(&self.faces[face], &ray),
                    Element::UpperSide(face) => face_height(&self.upper_sides[face], &ray),
                    Element::Vertex(vertex) => self.vertex_height(vertex, x, y, &ray),
                    Element::Edge(edge) => self.edge_height(edge, x, y, &ray),
                }
            })
            .reduce(f64::max)?;
        // 基準点（ボールは球の中心、フラットは底面の中心）から工具先端へ
        Some(highest - self.shape.reference_offset())
    }

    /// (x, y) における工具先端位置（ToolPath 座標）
    pub fn tip_point_at(&self, x: f64, y: f64) -> Option<Point3D<f64>> {
        self.tip_height_at(x, y).map(|z| Point3D::new(x, y, z))
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
                // 交点は端点から許容誤差まで辺の外を含むため、辺の中に収めて高さを求める（辺の外へ延ばさない）
                let height = |point: Point2D<f64>| {
                    let along = ((point.x() - start[0]) * direction[0]
                        + (point.y() - start[1]) * direction[1])
                        / horizontal_squared;
                    start[2] + along.clamp(0.0, 1.0) * direction[2]
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

fn face_height(face: &Triangle3D<f64>, ray: &Ray3D<f64>) -> Option<f64> {
    highest(face.intersection(ray, tolerance()), |_| true)
}

/// 三角形 `corners` を、単位法線 `normal` の向きに工具の形でずらした三角形（ボールは法線の向きに半径、フラットは
/// 法線を水平面に射影した向きに半径）。面積がなくなる場合は `None`
fn offset_triangle(
    corners: [[f64; 3]; 3],
    normal: [f64; 3],
    shape: CutterShape,
) -> Option<Triangle3D<f64>> {
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
    let [a, b, c] = corners.map(|p| point_of([0, 1, 2].map(|k| p[k] + offset[k])));
    Triangle3D::new(a, b, c)
}

/// 面の要素を索引に入れる
fn push_face(
    face: &Triangle3D<f64>,
    element: Element,
    bounds: &mut Vec<([f64; 2], [f64; 2])>,
    elements: &mut Vec<Element>,
    lowest: &mut Vec<f64>,
) {
    let corners = [face.vertex_a(), face.vertex_b(), face.vertex_c()];
    bounds.push(xy_bounds(&corners.map(|p| [p.x(), p.y()]), 0.0));
    elements.push(element);
    lowest.push(corners.iter().map(|p| p.z()).fold(f64::INFINITY, f64::min));
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

/// 三角形 `[a, b, c]` の、頂点の順（右ねじ）の単位法線
fn unit_normal([a, b, c]: [[f64; 3]; 3]) -> [f64; 3] {
    let n = cross(sub(b, a), sub(c, a));
    let norm = dot(n, n).sqrt();
    n.map(|component| component / norm)
}

/// 三角形の向きを塊ごとにそろえ、法線が外（材料のない側）を向くように頂点の順を決めた三角形と、三角形ごとの、外の
/// 決まり方
///
/// 外は、塊の中の、上から届く面（上側の包絡に出ている面。3 軸では工具が上から近づく）が上を向く向きとする。向きを
/// そろえられない塊と、上から届く面がない塊は外を決められないため、面ごとに上向きにそろえる。
fn orient_outward(
    mesh: &TriangleMesh3D<f64>,
    adjacency: &TriangleMeshAdjacency,
    mut triangles: Vec<[usize; 3]>,
    vertices: &[[f64; 3]],
) -> Result<(Vec<[usize; 3]>, Vec<Outward>), CamSolverError> {
    let orientation = adjacency.orient(&triangles);
    for (triangle, &reversed) in triangles.iter_mut().zip(&orientation.reversed) {
        if reversed {
            triangle.swap(1, 2);
        }
    }
    let corners = |triangle: &[usize; 3]| triangle.map(|vertex| vertices[vertex]);
    let centroid = |triangle: &[usize; 3]| {
        let [a, b, c] = corners(triangle);
        [0, 1, 2].map(|k| (a[k] + b[k] + c[k]) / 3.0)
    };
    // 高い面から順に、上から届くかを確かめる（塊ごとに、最初に見つかった面で外を決める）
    let surface = ProductSurface::new(mesh)?;
    let mut order: Vec<usize> = (0..triangles.len()).collect();
    order.sort_by(|&first, &second| {
        centroid(&triangles[second])[2].total_cmp(&centroid(&triangles[first])[2])
    });
    let mut flip: Vec<Option<bool>> = vec![None; orientation.orientable.len()];
    let mut undecided = orientation
        .orientable
        .iter()
        .filter(|&&orientable| orientable)
        .count();
    let tolerance = ToleranceSettings::<f64>::standard().distance_tolerance;
    for triangle in order {
        if undecided == 0 {
            break;
        }
        let component = orientation.components[triangle];
        if !orientation.orientable[component] || flip[component].is_some() {
            continue;
        }
        let normal = unit_normal(corners(&triangles[triangle]));
        if normal[2].abs() < default_orthogonality_dot_error_tolerance::<f64>() {
            continue;
        }
        let [x, y, z] = centroid(&triangles[triangle]);
        if surface
            .height_at(x, y)
            .is_some_and(|height| z >= height - tolerance)
        {
            flip[component] = Some(normal[2] < 0.0);
            undecided -= 1;
        }
    }
    // 縁の辺・3 つ以上の面の辺がある塊は開いている
    let mut open = vec![false; orientation.orientable.len()];
    for edge in adjacency.edges() {
        if edge.triangles.len() != 2 {
            for &triangle in &edge.triangles {
                open[orientation.components[triangle]] = true;
            }
        }
    }
    let mut outward = Vec::with_capacity(triangles.len());
    for (triangle, &component) in triangles.iter_mut().zip(&orientation.components) {
        let reverse = match flip[component] {
            Some(flip) => flip,
            None => unit_normal(corners(triangle))[2] < 0.0,
        };
        if reverse {
            triangle.swap(1, 2);
        }
        outward.push(match (flip[component], open[component]) {
            (None, _) => Outward::Unknown,
            (Some(_), false) => Outward::Closed,
            (Some(_), true) => Outward::Open,
        });
    }
    Ok((triangles, outward))
}

/// 辺が凹か平らか（隣り合う面がちょうど 2 つで、ともに外を決められた場合だけ判定し、それ以外は偽。開いた塊で下を
/// 向く面に隣り合う辺は、面の上側から工具が届くことがあり、上側から見ると凹凸が逆になるため偽）
///
/// 一方の面の外向きの法線で、もう一方の面の辺にない頂点が、面を含む平面の外の側にあれば凸。わずかでも凸の辺は残す
/// （面の要素は辺から離れる向きにずれ、間を辺の要素が埋めるため。フラットは面ごとに工具の半径だけずれるため、ほぼ
/// 平らな凸の辺でも間が大きく開く）。外向きの法線で決めるため、工具の向きによらない。
fn is_concave_or_flat(
    edge: &geo_algorithms::TriangleMeshEdge,
    triangles: &[[usize; 3]],
    vertices: &[[f64; 3]],
    normals: &[[f64; 3]],
    outward: &[Outward],
) -> bool {
    let [first, second] = edge.triangles[..] else {
        return false;
    };
    if outward[first] == Outward::Unknown || outward[second] == Outward::Unknown {
        return false;
    }
    let downward = |triangle: usize| {
        normals[triangle][2] <= -default_orthogonality_dot_error_tolerance::<f64>()
    };
    if (outward[first] == Outward::Open || outward[second] == Outward::Open)
        && (downward(first) || downward(second))
    {
        return false;
    }
    let Some(&other) = triangles[second]
        .iter()
        .find(|vertex| !edge.vertices.contains(vertex))
    else {
        return false;
    };
    let side = dot(
        sub(vertices[other], vertices[edge.vertices[0]]),
        normals[first],
    );
    // 法線が逆を向く（面が 90 度より深く折り返す）辺で、もう一方の面が同じ平面にある場合は、裏表の面が重なる厚みの
    // ない縁のため、凸として残す
    if dot(normals[first], normals[second]) < 0.0 {
        return side > tolerance();
    }
    side >= 0.0
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

mod display;
#[cfg(test)]
mod tests;
