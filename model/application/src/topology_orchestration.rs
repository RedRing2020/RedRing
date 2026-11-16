//! Topology向け orchestration 境界。

use crate::primitives::{Arc3D, Circle3D, LineSegment3D, Triangle3D};
use geo_contracts::{
    Arc3DEndpoint, Arc3DProperties, Circle3DConstructor, Circle3DEvaluation, Circle3DProperties,
    InfiniteLine3DConstructor, InfiniteLine3DProperties, LineSegment3DProperties,
    LineSegment3DSupportLine, Plane3DConstructor, Triangle3DBoundaryAccess,
    default_distance_tolerance,
};
use geo_topology::{
    Brep, BrepValidationReport, CurveRef, EdgeIndex, EdgeUse, FaceIndex, SurfaceRef,
    TopoInfiniteLine3D, TopoPlane3D, TopologyToleranceSettings, TopologyValidator,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLineTopologyRequest {
    pub line: LineSegment3D<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateArcTopologyRequest {
    pub arc: Arc3D<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCircleTopologyRequest {
    pub circle: Circle3D<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTriangleTopologyRequest {
    pub triangle: Triangle3D<f64>,
}

/// 稜線 1 本の B-rep
#[derive(Debug, Clone)]
pub struct CreateCurveTopologyResult {
    pub brep: Brep<f64>,
    pub edge: EdgeIndex,
    pub validation: BrepValidationReport,
}

/// 面 1 つの B-rep
#[derive(Debug, Clone)]
pub struct CreateFaceTopologyResult {
    pub brep: Brep<f64>,
    pub face: FaceIndex,
    pub validation: BrepValidationReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyOrchestrationError {
    InvalidLineTopology,
    InvalidArcTopology,
    InvalidCircleTopology,
    InvalidTriangleTopology,
}

impl std::fmt::Display for TopologyOrchestrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLineTopology => write!(f, "invalid line topology"),
            Self::InvalidArcTopology => write!(f, "invalid arc topology"),
            Self::InvalidCircleTopology => write!(f, "invalid circle topology"),
            Self::InvalidTriangleTopology => write!(f, "invalid triangle topology"),
        }
    }
}

impl std::error::Error for TopologyOrchestrationError {}

/// 線・円弧・円のトポロジーを作る
pub(crate) trait CurveTopologyMutationPort {
    fn create_line_topology(
        &self,
        request: CreateLineTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError>;

    fn create_arc_topology(
        &self,
        request: CreateArcTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError>;

    fn create_circle_topology(
        &self,
        request: CreateCircleTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError>;
}

/// 三角形のトポロジーを作る
pub trait DebugShapeTopologyMutationPort {
    fn create_triangle_topology(
        &self,
        request: CreateTriangleTopologyRequest,
    ) -> Result<CreateFaceTopologyResult, TopologyOrchestrationError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TopologyOrchestrator;

impl CurveTopologyMutationPort for TopologyOrchestrator {
    fn create_line_topology(
        &self,
        request: CreateLineTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
        let mut brep = Brep::new();
        let start = brep.add_vertex(request.line.start_point());
        let end = brep.add_vertex(request.line.end_point());
        // 稜線は、線分の母線と、その区間（母線が終点から始点の向きなら、母線を逆にして区間を小さい順にする）
        let support_line = request.line.support_line();
        let (start_param, end_param) = (request.line.start_param(), request.line.end_param());
        let (line, range) = if start_param <= end_param {
            (*support_line, (start_param, end_param))
        } else {
            let reversed = TopoInfiniteLine3D::new(
                support_line.point(),
                -support_line.direction().as_vector(),
            )
            .ok_or(TopologyOrchestrationError::InvalidLineTopology)?;
            (reversed, (-start_param, -end_param))
        };
        let edge = brep
            .add_edge(start, end, CurveRef::Line(line), range)
            .map_err(|_| TopologyOrchestrationError::InvalidLineTopology)?;
        curve_result(brep, edge, TopologyOrchestrationError::InvalidLineTopology)
    }

    fn create_arc_topology(
        &self,
        request: CreateArcTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
        let error = TopologyOrchestrationError::InvalidArcTopology;
        // 稜線は、円弧の載る円と、始点の角度から円弧の角度の分の区間
        let arc = &request.arc;
        let circle = Circle3D::new(arc.center(), arc.normal(), arc.radius()).ok_or(error)?;
        let start_point = arc.start_point();
        let offset = start_point - circle.center();
        let x = circle.ref_direction().as_vector();
        let y = circle.axis().as_vector().cross(x);
        let start_angle = offset.dot(y).atan2(offset.dot(x));
        let end_angle = start_angle + arc.angle_span().to_radians();
        let mut brep = Brep::new();
        let start = brep.add_vertex(start_point);
        let end = brep.add_vertex(arc.end_point());
        let edge = brep
            .add_edge(
                start,
                end,
                CurveRef::Circle(circle),
                (start_angle, end_angle),
            )
            .map_err(|_| error)?;
        curve_result(brep, edge, TopologyOrchestrationError::InvalidArcTopology)
    }

    fn create_circle_topology(
        &self,
        request: CreateCircleTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
        // 1 周の稜線は、始点と終点が同じ頂点
        let mut brep = Brep::new();
        let vertex = brep.add_vertex(request.circle.point_at_parameter(0.0));
        let edge = brep
            .add_edge(
                vertex,
                vertex,
                CurveRef::Circle(request.circle),
                (0.0, std::f64::consts::TAU),
            )
            .map_err(|_| TopologyOrchestrationError::InvalidCircleTopology)?;
        curve_result(
            brep,
            edge,
            TopologyOrchestrationError::InvalidCircleTopology,
        )
    }
}

/// 稜線 1 本の B-rep を確かめ、確かめられなければ `error` を返す
fn curve_result(
    brep: Brep<f64>,
    edge: EdgeIndex,
    error: TopologyOrchestrationError,
) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
    let validation =
        TopologyValidator::new(TopologyToleranceSettings::new(default_distance_tolerance()))
            .validate(&brep);
    if !validation.is_valid() {
        return Err(error);
    }
    Ok(CreateCurveTopologyResult {
        brep,
        edge,
        validation,
    })
}

impl DebugShapeTopologyMutationPort for TopologyOrchestrator {
    fn create_triangle_topology(
        &self,
        request: CreateTriangleTopologyRequest,
    ) -> Result<CreateFaceTopologyResult, TopologyOrchestrationError> {
        let error = TopologyOrchestrationError::InvalidTriangleTopology;
        let points = [
            request.triangle.vertex_a(),
            request.triangle.vertex_b(),
            request.triangle.vertex_c(),
        ];
        let mut brep = Brep::new();
        let vertices = points.map(|point| brep.add_vertex(point));
        let mut uses = Vec::with_capacity(3);
        for side in 0..3 {
            let (start, end) = (side, (side + 1) % 3);
            let line =
                TopoInfiniteLine3D::from_two_points(points[start], points[end]).ok_or(error)?;
            let edge = brep
                .add_edge(
                    vertices[start],
                    vertices[end],
                    CurveRef::Line(line),
                    (0.0, points[start].distance_to(points[end])),
                )
                .map_err(|_| error)?;
            uses.push(EdgeUse {
                edge,
                forward: true,
            });
        }
        // 頂点 A・B・C の順に回る向きを表にする平面
        let plane = TopoPlane3D::from_three_points(points[0], points[1], points[2]).ok_or(error)?;
        let tolerances = TopologyToleranceSettings::new(default_distance_tolerance());
        let face = brep
            .add_face(SurfaceRef::Plane(plane), true, vec![uses], tolerances)
            .map_err(|_| error)?;

        let validation = TopologyValidator::new(tolerances).validate(&brep);
        if !validation.is_valid() {
            return Err(error);
        }
        Ok(CreateFaceTopologyResult {
            brep,
            face,
            validation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CreateArcTopologyRequest, CreateCircleTopologyRequest, CreateLineTopologyRequest,
        CreateTriangleTopologyRequest, CurveTopologyMutationPort, DebugShapeTopologyMutationPort,
        TopologyOrchestrator,
    };
    use crate::primitives::{Arc3D, Circle3D, LineSegment3D, Point3D, Triangle3D};
    use geo_contracts::Angle;
    use geo_contracts::Arc3DConstructor;
    use geo_contracts::Circle3DConstructor;
    use geo_contracts::LineSegment3DConstructor;
    use geo_contracts::Triangle3DConstructor;

    #[test]
    fn create_line_topology_returns_valid_edge() {
        let line = LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0))
            .expect("line should be constructible");

        let result = TopologyOrchestrator
            .create_line_topology(CreateLineTopologyRequest { line })
            .expect("topology orchestration should succeed");

        assert!(result.validation.is_valid());
        let edge = result.brep.edge(result.edge);
        assert_eq!(result.brep.vertex(edge.start()).point().x(), 0.0);
        assert_eq!(result.brep.vertex(edge.end()).point().x(), 2.0);
    }

    #[test]
    fn create_arc_topology_returns_valid_edge() {
        let arc = Arc3D::xy_arc(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::FRAC_PI_2),
        )
        .expect("arc should be constructible");

        let result = TopologyOrchestrator
            .create_arc_topology(CreateArcTopologyRequest { arc })
            .expect("arc topology orchestration should succeed");

        assert!(result.validation.is_valid());
    }

    #[test]
    fn create_circle_topology_returns_valid_closed_edge() {
        let circle = Circle3D::new_xy_plane(Point3D::new(0.0, 0.0, 0.0), 3.0)
            .expect("circle should be constructible");

        let result = TopologyOrchestrator
            .create_circle_topology(CreateCircleTopologyRequest { circle })
            .expect("circle topology orchestration should succeed");

        assert!(result.validation.is_valid());
        let edge = result.brep.edge(result.edge);
        assert_eq!(edge.start(), edge.end());
    }

    #[test]
    fn create_triangle_topology_returns_a_valid_face() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .expect("triangle should be constructible");

        let result = TopologyOrchestrator
            .create_triangle_topology(CreateTriangleTopologyRequest { triangle })
            .expect("triangle topology orchestration should succeed");

        assert!(result.validation.is_valid());
        let face = result.brep.face(result.face);
        assert_eq!(face.loops().len(), 1);
        assert_eq!(result.brep.face_loop(face.loops()[0]).uses().len(), 3);
    }
}
