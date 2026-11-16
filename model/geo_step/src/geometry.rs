//! STEP の幾何の記録を、曲線・曲面の型にする（設計: BREP_AND_STEP_IMPORT_DESIGN.md 7 章）
//!
//! 配置を正規直交の座標系にし、直線・円・楕円・平面・円柱・円錐・球・トーラスを解析的な型（形を変えない）、
//! B-spline を `geo_nurbs` の型（両端を固定していないものは、同じ形の固定したものに直す）にする。放物線・
//! 双曲線は、稜線の区間が決まったときに NURBS にするため、円錐曲線として返す。押し出し・回転・オフセットの
//! 面は、7 章の範囲だけを読む。読めない幾何は、理由を記録して、ほかを読み続ける。

mod bspline;
mod conic;
#[cfg(test)]
mod tests;

pub use conic::Conic;

use crate::record::{Geometry as Record, Records};
use geo_contracts::{
    Angle, Circle3DConstructor, ConicalSurface3DConstructor, CylindricalSurface3DConstructor,
    Ellipse3DConstructor, InfiniteLine3DConstructor, Plane3DConstructor,
    SphericalSurface3DConstructor, ToleranceSettings, TorusSurface3DConstructor,
};
use geo_core::{Direction3D, Point3D, Vector3D};
use geo_topology::{
    CurveRef, SurfaceRef, TopoCircle3D, TopoConicalSurface3D, TopoCylindricalSurface3D,
    TopoEllipse3D, TopoInfiniteLine3D, TopoPlane3D, TopoSphericalSurface3D, TopoTorusSurface3D,
};
use std::collections::{BTreeMap, BTreeSet};

/// 型にした曲線
#[derive(Debug, Clone)]
pub enum Curve {
    /// 稜線の母曲線にそのまま使える曲線
    Base(CurveRef<f64>),
    /// 放物線・双曲線（稜線の区間ごとに NURBS にする）
    Conic(Conic),
}

/// 型にした幾何（番号はファイルの番号）
#[derive(Debug, Clone, Default)]
pub struct Geometries {
    /// 曲線（曲面の上の曲線の番号にも、その 3 次元の曲線を入れる）
    pub curves: BTreeMap<u64, Curve>,
    pub surfaces: BTreeMap<u64, Surface>,
    /// 型にできなかった曲線・曲面と、その理由
    pub skipped: Vec<GeometrySkipped>,
}

/// 型にした曲面
#[derive(Debug, Clone)]
pub struct Surface {
    pub surface: SurfaceRef<f64>,
    /// 型の曲面の法線（∂u × ∂v）と、STEP の曲面の法線の関係（面の向きを決めるときに使う）
    pub sense: Sense,
}

/// 型の曲面の法線と、STEP の曲面の法線の関係
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    /// 同じ向き
    Same,
    /// 逆の向き（押し出し・回転の面は、型の曲面を外向きにそろえるため、STEP の曲線と向きによって逆になる）
    Reversed,
    /// 面ごとに決まる（中心が軸の上の円を回した球は、円の半分ずつが球の全体を作り、面がどちらの半分から
    /// 作られたかで STEP の法線が逆になる。面の点だけからは決まらないため、面のループの回りで決める）
    PerFace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeometrySkipped {
    pub id: u64,
    pub reason: GeometryError,
}

/// 曲線・曲面を型にできない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    /// 参照の先の幾何の記録がない（または、型にできなかった）
    MissingReference,
    /// 方向の大きさが 0、または軸と基準の方向が平行で、座標系を作れない
    DegeneratePlacement,
    /// 半径などの値が、形を作れない値
    InvalidShape,
    /// B-spline のノット・重複・重みが、制御点の数と次数に合わない
    InvalidBSpline,
    /// 設計書 7 章の範囲の外の押し出し・回転・オフセットの面
    Unsupported,
    /// 曲面の上の曲線・オフセットの面が、たどると自分を指している
    CyclicReference,
}

/// 正規直交の座標系（原点と、X・Y・Z の軸）
#[derive(Debug, Clone, Copy)]
struct Frame {
    origin: Point3D<f64>,
    x: Direction3D<f64>,
    y: Direction3D<f64>,
    z: Direction3D<f64>,
}

/// 中間データの幾何を、曲線・曲面の型にする
///
/// `tolerances` は、平行か・軸の上かなどの形の判定に使う（距離と角度）。
pub fn convert(records: &Records, tolerances: &ToleranceSettings<f64>) -> Geometries {
    let mut converter = Converter {
        records,
        tolerances,
        geometries: Geometries::default(),
        failed: BTreeMap::new(),
        visiting: BTreeSet::new(),
    };
    for (&id, record) in &records.geometry {
        // 結果は、型にする処理が覚え、型にできなければ理由を記録する
        match record {
            Record::Line { .. }
            | Record::Circle { .. }
            | Record::Ellipse { .. }
            | Record::Parabola { .. }
            | Record::Hyperbola { .. }
            | Record::BSplineCurve(_)
            | Record::SurfaceCurve { .. } => {
                let _ = converter.curve(id);
            }
            Record::Plane { .. }
            | Record::CylindricalSurface { .. }
            | Record::ConicalSurface { .. }
            | Record::SphericalSurface { .. }
            | Record::ToroidalSurface { .. }
            | Record::BSplineSurface(_)
            | Record::SurfaceOfLinearExtrusion { .. }
            | Record::SurfaceOfRevolution { .. }
            | Record::OffsetSurface { .. } => {
                let _ = converter.surface(id);
            }
            // 点・方向・配置などは、曲線・曲面の一部として読む
            _ => {}
        }
    }
    converter.geometries
}

struct Converter<'a> {
    records: &'a Records,
    tolerances: &'a ToleranceSettings<f64>,
    geometries: Geometries,
    /// 型にできなかった曲線・曲面（同じ理由を何度も記録しないため）
    failed: BTreeMap<u64, GeometryError>,
    /// 型にしている途中の曲線・曲面（たどると自分を指す参照を見つけるため）
    visiting: BTreeSet<u64>,
}

impl<'a> Converter<'a> {
    fn record(&self, id: u64) -> Result<&'a Record, GeometryError> {
        self.records
            .geometry
            .get(&id)
            .ok_or(GeometryError::MissingReference)
    }

    /// B-spline の制御点の点（mm の座標のまま）
    fn raw_point(&self, id: u64) -> Option<crate::record::Point> {
        match self.records.geometry.get(&id) {
            Some(Record::Point(point)) => Some(*point),
            _ => None,
        }
    }

    /// 型にした結果を覚え、型にできなければ理由を記録する
    fn remember<T>(
        &mut self,
        id: u64,
        result: Result<T, GeometryError>,
        store: impl FnOnce(&mut Geometries, T),
    ) -> Result<(), GeometryError> {
        match result {
            Ok(value) => {
                store(&mut self.geometries, value);
                Ok(())
            }
            // 自分を指す参照は、たどり始めた要素で記録する
            Err(GeometryError::CyclicReference) if self.visiting.contains(&id) => {
                Err(GeometryError::CyclicReference)
            }
            Err(reason) => {
                self.failed.insert(id, reason);
                self.geometries.skipped.push(GeometrySkipped { id, reason });
                Err(reason)
            }
        }
    }

    /// 曲線 `id` を型にする（一度型にしたものは覚えておき、同じ曲線を何度も型にしない）
    fn curve(&mut self, id: u64) -> Result<Curve, GeometryError> {
        if let Some(curve) = self.geometries.curves.get(&id) {
            return Ok(curve.clone());
        }
        if let Some(&reason) = self.failed.get(&id) {
            return Err(reason);
        }
        if !self.visiting.insert(id) {
            return Err(GeometryError::CyclicReference);
        }
        let result = self.build_curve(id);
        self.visiting.remove(&id);
        self.remember(id, result.clone(), |geometries, curve| {
            geometries.curves.insert(id, curve);
        })?;
        result
    }

    /// 曲面 `id` を型にする（曲線と同じく覚えておく）
    fn surface(&mut self, id: u64) -> Result<Surface, GeometryError> {
        if let Some(surface) = self.geometries.surfaces.get(&id) {
            return Ok(surface.clone());
        }
        if let Some(&reason) = self.failed.get(&id) {
            return Err(reason);
        }
        if !self.visiting.insert(id) {
            return Err(GeometryError::CyclicReference);
        }
        let result = self.build_surface(id);
        self.visiting.remove(&id);
        self.remember(id, result.clone(), |geometries, surface| {
            geometries.surfaces.insert(id, surface);
        })?;
        result
    }

    fn point(&self, id: u64) -> Result<Point3D<f64>, GeometryError> {
        match self.record(id)? {
            Record::Point([x, y, z]) => Ok(Point3D::new(*x, *y, *z)),
            _ => Err(GeometryError::MissingReference),
        }
    }

    fn direction(&self, id: u64) -> Result<Direction3D<f64>, GeometryError> {
        match self.record(id)? {
            Record::Direction([x, y, z]) => Direction3D::from_vector(Vector3D::new(*x, *y, *z))
                .ok_or(GeometryError::DegeneratePlacement),
            _ => Err(GeometryError::MissingReference),
        }
    }

    /// 方向と大きさ（大きさは使わず、向きだけを返す）
    fn vector(&self, id: u64) -> Result<Direction3D<f64>, GeometryError> {
        match self.record(id)? {
            Record::Vector {
                direction,
                magnitude,
            } => {
                let direction = self.direction(*direction)?;
                if *magnitude < 0.0 {
                    Direction3D::from_vector(-direction.as_vector())
                        .ok_or(GeometryError::DegeneratePlacement)
                } else {
                    Ok(direction)
                }
            }
            _ => Err(GeometryError::MissingReference),
        }
    }

    /// 配置（`AXIS2_PLACEMENT_3D`）の正規直交の座標系（ISO 10303-42 の `build_axes` のとおり、軸がなければ
    /// Z、基準の方向がなければ X（軸が X なら Y）とし、基準の方向から軸の成分を除く）
    fn frame(&self, id: u64) -> Result<Frame, GeometryError> {
        let Record::Placement {
            location,
            axis,
            ref_direction,
        } = self.record(id)?
        else {
            return Err(GeometryError::MissingReference);
        };
        let origin = self.point(*location)?;
        let z = match axis {
            Some(axis) => self.direction(*axis)?,
            None => Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0))
                .ok_or(GeometryError::DegeneratePlacement)?,
        };
        let reference = match ref_direction {
            Some(reference) => self.direction(*reference)?.as_vector(),
            None => {
                let x = Vector3D::new(1.0, 0.0, 0.0);
                if z.as_vector().cross(x).length() <= self.tolerances.angle_tolerance {
                    Vector3D::new(0.0, 1.0, 0.0)
                } else {
                    x
                }
            }
        };
        let projected = reference - z.as_vector() * reference.dot(z.as_vector());
        if projected.length() <= self.tolerances.angle_tolerance {
            return Err(GeometryError::DegeneratePlacement);
        }
        let x = Direction3D::from_vector(projected).ok_or(GeometryError::DegeneratePlacement)?;
        let y = Direction3D::from_vector(z.as_vector().cross(x.as_vector()))
            .ok_or(GeometryError::DegeneratePlacement)?;
        Ok(Frame { origin, x, y, z })
    }

    fn build_curve(&mut self, id: u64) -> Result<Curve, GeometryError> {
        let invalid = || GeometryError::InvalidShape;
        Ok(match self.record(id)? {
            Record::Line { point, vector } => Curve::Base(CurveRef::Line(
                TopoInfiniteLine3D::new(self.point(*point)?, self.vector(*vector)?.as_vector())
                    .ok_or(GeometryError::DegeneratePlacement)?,
            )),
            Record::Circle { placement, radius } => {
                let frame = self.frame(*placement)?;
                Curve::Base(CurveRef::Circle(
                    circle(&frame, *radius).ok_or_else(invalid)?,
                ))
            }
            Record::Ellipse {
                placement,
                semi_axis_1,
                semi_axis_2,
            } => {
                let frame = self.frame(*placement)?;
                // 長い半軸の方向を長軸にする
                let (major, minor, major_direction) = if semi_axis_1 >= semi_axis_2 {
                    (*semi_axis_1, *semi_axis_2, frame.x)
                } else {
                    (*semi_axis_2, *semi_axis_1, frame.y)
                };
                Curve::Base(CurveRef::Ellipse(
                    TopoEllipse3D::new(frame.origin, major, minor, frame.z, major_direction)
                        .ok_or_else(invalid)?,
                ))
            }
            Record::Parabola {
                placement,
                focal_distance,
            } => {
                let frame = self.frame(*placement)?;
                if *focal_distance == 0.0 {
                    return Err(invalid());
                }
                Curve::Conic(Conic::Parabola {
                    center: frame.origin,
                    x: frame.x.as_vector(),
                    y: frame.y.as_vector(),
                    focal_distance: *focal_distance,
                })
            }
            Record::Hyperbola {
                placement,
                semi_axis,
                semi_imaginary_axis,
            } => {
                let frame = self.frame(*placement)?;
                if *semi_axis <= 0.0 || *semi_imaginary_axis <= 0.0 {
                    return Err(invalid());
                }
                Curve::Conic(Conic::Hyperbola {
                    center: frame.origin,
                    x: frame.x.as_vector(),
                    y: frame.y.as_vector(),
                    semi_axis: *semi_axis,
                    semi_imaginary_axis: *semi_imaginary_axis,
                })
            }
            Record::BSplineCurve(record) => Curve::Base(CurveRef::Nurbs(
                bspline::curve(record, |id| self.raw_point(id)).map_err(bspline_error)?,
            )),
            Record::SurfaceCurve { curve } => self.curve(*curve)?,
            _ => return Err(GeometryError::MissingReference),
        })
    }

    fn build_surface(&mut self, id: u64) -> Result<Surface, GeometryError> {
        let invalid = || GeometryError::InvalidShape;
        // 解析的な曲面と B-spline 曲面は、STEP と同じパラメータの取り方なので、法線の向きも同じ
        let same = |surface| Surface {
            surface,
            sense: Sense::Same,
        };
        Ok(match self.record(id)? {
            Record::Plane { placement } => {
                let frame = self.frame(*placement)?;
                same(SurfaceRef::Plane(
                    TopoPlane3D::from_origin_and_axes(frame.origin, frame.z, frame.x)
                        .ok_or_else(invalid)?,
                ))
            }
            Record::CylindricalSurface { placement, radius } => same(SurfaceRef::Cylinder(
                cylinder(&self.frame(*placement)?, *radius).ok_or_else(invalid)?,
            )),
            Record::ConicalSurface {
                placement,
                radius,
                semi_angle,
            } => same(SurfaceRef::Cone(
                cone(&self.frame(*placement)?, *radius, *semi_angle).ok_or_else(invalid)?,
            )),
            Record::SphericalSurface { placement, radius } => same(SurfaceRef::Sphere(
                sphere(&self.frame(*placement)?, *radius).ok_or_else(invalid)?,
            )),
            Record::ToroidalSurface {
                placement,
                major_radius,
                minor_radius,
            } => same(SurfaceRef::Torus(
                torus(&self.frame(*placement)?, *major_radius, *minor_radius)
                    .ok_or_else(invalid)?,
            )),
            Record::BSplineSurface(record) => same(SurfaceRef::Nurbs(
                bspline::surface(record, |id| self.raw_point(id)).map_err(bspline_error)?,
            )),
            Record::SurfaceOfLinearExtrusion { curve, vector } => {
                let direction = self.vector(*vector)?;
                self.extrusion(*curve, direction)?
            }
            Record::SurfaceOfRevolution { curve, axis } => self.revolution(*curve, *axis)?,
            Record::OffsetSurface { surface, distance } => {
                let basis = self.surface(*surface)?;
                // STEP のオフセットは STEP の曲面の法線の側へ。型の曲面の法線が逆なら、逆の側へ動かす
                let distance = match basis.sense {
                    Sense::Same => *distance,
                    Sense::Reversed => -distance,
                    // 面ごとに向きの決まる曲面のオフセットは、動かす側が決まらない
                    Sense::PerFace => return Err(GeometryError::Unsupported),
                };
                Surface {
                    surface: offset(basis.surface, distance)?,
                    sense: basis.sense,
                }
            }
            _ => return Err(GeometryError::MissingReference),
        })
    }

    /// 押し出しの面: 直線は平面、押し出しの向きが軸と平行な円は円柱（ほかは範囲の外）
    ///
    /// STEP の押し出しの面の法線は、曲線の接線 × 押し出しの向き。平面はこの法線で作り、円柱（外向き）は、
    /// 押し出しの向きが円の軸と逆なら、法線が逆になる。
    fn extrusion(
        &mut self,
        curve: u64,
        direction: Direction3D<f64>,
    ) -> Result<Surface, GeometryError> {
        use geo_contracts::{Circle3DProperties, InfiniteLine3DProperties};
        let angle = self.tolerances.angle_tolerance;
        match self.curve(curve)? {
            Curve::Base(CurveRef::Line(line)) => {
                let normal = line.direction().as_vector().cross(direction.as_vector());
                if normal.length() <= angle {
                    return Err(GeometryError::DegeneratePlacement);
                }
                let normal =
                    Direction3D::from_vector(normal).ok_or(GeometryError::DegeneratePlacement)?;
                Ok(Surface {
                    surface: SurfaceRef::Plane(
                        TopoPlane3D::from_origin_and_axes(line.point(), normal, line.direction())
                            .ok_or(GeometryError::InvalidShape)?,
                    ),
                    sense: Sense::Same,
                })
            }
            Curve::Base(CurveRef::Circle(circle)) => {
                let along = circle.axis().as_vector().dot(direction.as_vector());
                if circle
                    .axis()
                    .as_vector()
                    .cross(direction.as_vector())
                    .length()
                    > angle
                {
                    return Err(GeometryError::Unsupported);
                }
                Ok(Surface {
                    surface: SurfaceRef::Cylinder(
                        TopoCylindricalSurface3D::new(
                            circle.center(),
                            circle.axis(),
                            circle.ref_direction(),
                            circle.radius(),
                        )
                        .ok_or(GeometryError::InvalidShape)?,
                    ),
                    sense: sense_of(along < 0.0),
                })
            }
            _ => Err(GeometryError::Unsupported),
        }
    }

    /// 回転の面: 軸に平行な直線は円柱、軸と交わる直線は円錐、軸を含む平面の上で中心が軸の上の円は球、
    /// 軸を含む平面の上の円はトーラス（ほかは範囲の外）
    ///
    /// 型の曲面は外向きの法線で作る。STEP の回転の面の法線（回る向き × 曲線の接線）と、回した曲線の点で比べ、
    /// 逆なら `Sense::Reversed` にする。中心が軸の上の円を回した球は、面ごとに決まる（`Sense::PerFace`）。
    fn revolution(&mut self, curve: u64, axis: u64) -> Result<Surface, GeometryError> {
        use geo_contracts::{
            Circle3DEvaluation, Circle3DProperties, InfiniteLine3DEvaluation,
            InfiniteLine3DProperties,
        };
        let Record::Axis { location, axis } = self.record(axis)? else {
            return Err(GeometryError::MissingReference);
        };
        let origin = self.point(*location)?;
        let k = match axis {
            Some(axis) => self.direction(*axis)?,
            None => Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0))
                .ok_or(GeometryError::DegeneratePlacement)?,
        };
        let (distance, angle) = (
            self.tolerances.distance_tolerance,
            self.tolerances.angle_tolerance,
        );
        // 点 `point` の、軸の上の足と、軸から離れる向きのベクトル
        let foot = |point: Point3D<f64>| {
            let along = (point - origin).dot(k.as_vector());
            let foot = origin + k.as_vector() * along;
            (foot, point - foot)
        };
        // 回した曲線の点 `point`（接線 `tangent`）での、STEP の回転の面の法線
        let step_normal = |point: Point3D<f64>, tangent: Vector3D<f64>| {
            k.as_vector().cross(foot(point).1).cross(tangent)
        };
        match self.curve(curve)? {
            Curve::Base(CurveRef::Line(line)) => {
                let d = line.direction();
                let (foot_point, radial) = foot(line.point());
                if d.as_vector().cross(k.as_vector()).length() <= angle {
                    // 軸に平行: 円柱（外向きの法線は、軸から離れる向き）
                    let reference =
                        Direction3D::from_vector(radial).ok_or(GeometryError::Unsupported)?;
                    return Ok(Surface {
                        surface: SurfaceRef::Cylinder(
                            TopoCylindricalSurface3D::new(
                                foot_point,
                                k,
                                reference,
                                radial.length(),
                            )
                            .ok_or(GeometryError::InvalidShape)?,
                        ),
                        sense: sense_of(step_normal(line.point(), d.as_vector()).dot(radial) < 0.0),
                    });
                }
                // 軸と交わるか（2 つの直線の間の距離）
                let normal = d.as_vector().cross(k.as_vector());
                let gap = (line.point() - origin).dot(normal) / normal.length();
                if gap.abs() > distance {
                    return Err(GeometryError::Unsupported);
                }
                // 頂点（軸との交点）から直線に沿って 1 進んだ点で、円錐を作る
                let along = d.as_vector().dot(k.as_vector());
                let perpendicular = d.as_vector() - k.as_vector() * along;
                let t_apex = -radial.dot(perpendicular) / perpendicular.length_squared();
                let apex = line.point_at_parameter(t_apex);
                let sample = line.point_at_parameter(t_apex + 1.0);
                let (sample_foot, sample_radial) = foot(sample);
                let height = (sample_foot - apex).dot(k.as_vector());
                if height.abs() <= distance {
                    return Err(GeometryError::Unsupported);
                }
                let axis_direction = if height > 0.0 {
                    k
                } else {
                    Direction3D::from_vector(-k.as_vector())
                        .ok_or(GeometryError::DegeneratePlacement)?
                };
                let reference =
                    Direction3D::from_vector(sample_radial).ok_or(GeometryError::Unsupported)?;
                let semi_angle = (sample_radial.length() / height.abs()).atan();
                // 円錐の外向きの法線: 軸から離れる向き × cos(半頂角) − 軸の向き × sin(半頂角)
                let outward = reference.as_vector() * semi_angle.cos()
                    - axis_direction.as_vector() * semi_angle.sin();
                Ok(Surface {
                    surface: SurfaceRef::Cone(
                        TopoConicalSurface3D::new(
                            sample_foot,
                            axis_direction,
                            reference,
                            sample_radial.length(),
                            Angle::from_radians(semi_angle),
                        )
                        .ok_or(GeometryError::InvalidShape)?,
                    ),
                    sense: sense_of(step_normal(sample, d.as_vector()).dot(outward) < 0.0),
                })
            }
            Curve::Base(CurveRef::Circle(circle)) => {
                // 円の平面が軸を含む（円の軸が回転の軸に垂直で、回転の軸が円の平面の上）
                if circle.axis().as_vector().dot(k.as_vector()).abs() > angle
                    || (origin - circle.center())
                        .dot(circle.axis().as_vector())
                        .abs()
                        > distance
                {
                    return Err(GeometryError::Unsupported);
                }
                // 法線を比べる、軸から最も離れた円の点（円の中心から、軸から離れる向き、または円の基準の方向）
                let (center_foot, radial) = foot(circle.center());
                let major = radial.length();
                let sample_angle = if major > distance {
                    let reference = circle.ref_direction().as_vector();
                    let side = circle.axis().as_vector().cross(reference);
                    radial.dot(side).atan2(radial.dot(reference))
                } else {
                    // 中心が軸の上なら、軸に垂直な向き（円の基準の方向と、その直角の向きのうち、軸から離れるほう）
                    let reference = circle.ref_direction().as_vector();
                    if reference.cross(k.as_vector()).length()
                        >= circle
                            .axis()
                            .as_vector()
                            .cross(reference)
                            .cross(k.as_vector())
                            .length()
                    {
                        0.0
                    } else {
                        std::f64::consts::FRAC_PI_2
                    }
                };
                let sample = circle.point_at_parameter(sample_angle);
                let tangent = circle.axis().as_vector().cross(sample - circle.center());
                let step = step_normal(sample, tangent);
                if major <= distance {
                    // 中心が軸の上: 球（外向きの法線は、中心から離れる向き）
                    let reference = perpendicular(k).ok_or(GeometryError::DegeneratePlacement)?;
                    return Ok(Surface {
                        surface: SurfaceRef::Sphere(
                            TopoSphericalSurface3D::new(center_foot, k, reference, circle.radius())
                                .ok_or(GeometryError::InvalidShape)?,
                        ),
                        sense: Sense::PerFace,
                    });
                }
                if major <= circle.radius() {
                    // 管が軸と交わる（縮退したトーラス）は範囲の外
                    return Err(GeometryError::Unsupported);
                }
                let reference =
                    Direction3D::from_vector(radial).ok_or(GeometryError::Unsupported)?;
                // トーラスの外向きの法線は、管の中心（回した円の中心）から離れる向き
                Ok(Surface {
                    surface: SurfaceRef::Torus(
                        TopoTorusSurface3D::new(center_foot, k, reference, major, circle.radius())
                            .ok_or(GeometryError::InvalidShape)?,
                    ),
                    sense: sense_of(step.dot(sample - circle.center()) < 0.0),
                })
            }
            _ => Err(GeometryError::Unsupported),
        }
    }
}

/// 解析的な曲面のオフセット（型の曲面の外向きの法線の側へ `distance`）。ほかは範囲の外
fn offset(surface: SurfaceRef<f64>, distance: f64) -> Result<SurfaceRef<f64>, GeometryError> {
    use geo_contracts::{
        ConicalSurface3DProperties, CylindricalSurface3DProperties, Plane3DProperties,
        SphericalSurface3DProperties, TorusSurface3DProperties,
    };
    let invalid = || GeometryError::InvalidShape;
    Ok(match surface {
        SurfaceRef::Plane(plane) => SurfaceRef::Plane(
            TopoPlane3D::from_origin_and_axes(
                plane.origin() + plane.normal().as_vector() * distance,
                plane.normal(),
                plane.u_axis(),
            )
            .ok_or_else(invalid)?,
        ),
        SurfaceRef::Cylinder(cylinder) => SurfaceRef::Cylinder(
            TopoCylindricalSurface3D::new(
                cylinder.center(),
                cylinder.axis(),
                cylinder.ref_direction(),
                cylinder.radius() + distance,
            )
            .ok_or_else(invalid)?,
        ),
        // 円錐の外向きの法線の側へ動かすと、同じ半頂角で、同じ高さの半径が distance / cos(半頂角) 増える
        SurfaceRef::Cone(cone) => SurfaceRef::Cone(
            TopoConicalSurface3D::new(
                cone.center(),
                cone.axis(),
                cone.ref_direction(),
                cone.radius() + distance / cone.semi_angle().to_radians().cos(),
                cone.semi_angle(),
            )
            .ok_or_else(invalid)?,
        ),
        SurfaceRef::Sphere(sphere) => SurfaceRef::Sphere(
            TopoSphericalSurface3D::new(
                sphere.center(),
                sphere.axis(),
                sphere.ref_direction(),
                sphere.radius() + distance,
            )
            .ok_or_else(invalid)?,
        ),
        SurfaceRef::Torus(torus) => SurfaceRef::Torus(
            TopoTorusSurface3D::new(
                torus.center(),
                torus.axis(),
                torus.ref_direction(),
                torus.major_radius(),
                torus.minor_radius() + distance,
            )
            .ok_or_else(invalid)?,
        ),
        SurfaceRef::Nurbs(_) => return Err(GeometryError::Unsupported),
    })
}

fn circle(frame: &Frame, radius: f64) -> Option<TopoCircle3D<f64>> {
    TopoCircle3D::new(frame.origin, frame.z, radius)
}

fn cylinder(frame: &Frame, radius: f64) -> Option<TopoCylindricalSurface3D<f64>> {
    TopoCylindricalSurface3D::new(frame.origin, frame.z, frame.x, radius)
}

fn cone(frame: &Frame, radius: f64, semi_angle: Angle<f64>) -> Option<TopoConicalSurface3D<f64>> {
    TopoConicalSurface3D::new(frame.origin, frame.z, frame.x, radius, semi_angle)
}

fn sphere(frame: &Frame, radius: f64) -> Option<TopoSphericalSurface3D<f64>> {
    TopoSphericalSurface3D::new(frame.origin, frame.z, frame.x, radius)
}

fn torus(frame: &Frame, major: f64, minor: f64) -> Option<TopoTorusSurface3D<f64>> {
    TopoTorusSurface3D::new(frame.origin, frame.z, frame.x, major, minor)
}

/// 方向 `axis` に垂直な方向の 1 つ
fn perpendicular(axis: Direction3D<f64>) -> Option<Direction3D<f64>> {
    let k = axis.as_vector();
    let seed = if k.x().abs() < 0.9 {
        Vector3D::new(1.0, 0.0, 0.0)
    } else {
        Vector3D::new(0.0, 1.0, 0.0)
    };
    Direction3D::from_vector(seed - k * seed.dot(k))
}

fn bspline_error(error: bspline::BSplineError) -> GeometryError {
    match error {
        bspline::BSplineError::MissingPoint => GeometryError::MissingReference,
        bspline::BSplineError::Invalid => GeometryError::InvalidBSpline,
    }
}

/// 逆か（`reversed`）から、法線の関係
fn sense_of(reversed: bool) -> Sense {
    if reversed {
        Sense::Reversed
    } else {
        Sense::Same
    }
}
