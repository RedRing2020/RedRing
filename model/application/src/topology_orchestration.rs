//! Topology向け orchestration 境界。

use geo_contracts::{Arc3DConstructor, Circle3DProperties, Triangle3DBoundaryAccess};
use geo_primitives::{Arc3D, Circle3D, LineSegment3D, Point3D, Triangle3D};
use geo_topology::{
    CurveRef, Edge, EdgeValidationReport, TopologyToleranceSettings, TopologyValidator, Vertex,
    Wire, WireValidationReport,
};
use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub struct CreateCurveTopologyResult {
    pub edge: Edge<f64>,
    pub validation: EdgeValidationReport<f64>,
}

#[derive(Debug, Clone)]
pub struct CreateWireTopologyResult {
    pub wire: Wire<f64>,
    pub validation: WireValidationReport<f64>,
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

pub trait LineTopologyMutationPort {
    fn create_line_topology(
        &self,
        request: CreateLineTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError>;
}

pub trait DebugShapeTopologyMutationPort {
    fn create_arc_topology(
        &self,
        request: CreateArcTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError>;

    fn create_circle_topology(
        &self,
        request: CreateCircleTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError>;

    fn create_triangle_topology(
        &self,
        request: CreateTriangleTopologyRequest,
    ) -> Result<CreateWireTopologyResult, TopologyOrchestrationError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TopologyOrchestrator;

impl LineTopologyMutationPort for TopologyOrchestrator {
    fn create_line_topology(
        &self,
        request: CreateLineTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
        let start_vertex = Arc::new(Vertex::new(request.line.start()));
        let end_vertex = Arc::new(Vertex::new(request.line.end()));
        let edge = Edge::new(
            start_vertex,
            end_vertex,
            CurveRef::Line(request.line),
            (request.line.start_param(), request.line.end_param()),
        )
        .ok_or(TopologyOrchestrationError::InvalidLineTopology)?;

        let validation = TopologyValidator::new(TopologyToleranceSettings::new(1.0e-9, 1.0e-9))
            .validate_edge(&edge);

        if !validation.is_valid() {
            return Err(TopologyOrchestrationError::InvalidLineTopology);
        }

        Ok(CreateCurveTopologyResult { edge, validation })
    }
}

impl DebugShapeTopologyMutationPort for TopologyOrchestrator {
    fn create_arc_topology(
        &self,
        request: CreateArcTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
        let start = request.arc.start_point();
        let end = request.arc.end_point();
        let start_vertex = Arc::new(Vertex::new(geo_topology::Point3D::new(
            start.x(),
            start.y(),
            start.z(),
        )));
        let end_vertex = Arc::new(Vertex::new(geo_topology::Point3D::new(
            end.x(),
            end.y(),
            end.z(),
        )));
        let edge = Edge::new(
            start_vertex,
            end_vertex,
            CurveRef::Arc(request.arc),
            (0.0, 1.0),
        )
        .ok_or(TopologyOrchestrationError::InvalidArcTopology)?;

        let validation = TopologyValidator::new(TopologyToleranceSettings::new(1.0e-9, 1.0e-9))
            .validate_edge(&edge);

        if !validation.is_valid() {
            return Err(TopologyOrchestrationError::InvalidArcTopology);
        }

        Ok(CreateCurveTopologyResult { edge, validation })
    }

    fn create_circle_topology(
        &self,
        request: CreateCircleTopologyRequest,
    ) -> Result<CreateCurveTopologyResult, TopologyOrchestrationError> {
        let center = request.circle.center();
        let axis = request.circle.axis();
        let full_arc = Arc3D::full_circle(center, axis, request.circle.radius())
            .ok_or(TopologyOrchestrationError::InvalidCircleTopology)?;
        let start = full_arc.start_point();
        let shared_vertex = Arc::new(Vertex::new(geo_topology::Point3D::new(
            start.x(),
            start.y(),
            start.z(),
        )));
        let edge = Edge::new(
            shared_vertex.clone(),
            shared_vertex,
            CurveRef::Arc(full_arc),
            (0.0, 1.0),
        )
        .ok_or(TopologyOrchestrationError::InvalidCircleTopology)?;

        let validation = TopologyValidator::new(TopologyToleranceSettings::new(1.0e-9, 1.0e-9))
            .validate_edge(&edge);

        if !validation.is_valid() {
            return Err(TopologyOrchestrationError::InvalidCircleTopology);
        }

        Ok(CreateCurveTopologyResult { edge, validation })
    }

    fn create_triangle_topology(
        &self,
        request: CreateTriangleTopologyRequest,
    ) -> Result<CreateWireTopologyResult, TopologyOrchestrationError> {
        let (ax, ay, az) = request.triangle.vertex_a();
        let (bx, by, bz) = request.triangle.vertex_b();
        let (cx, cy, cz) = request.triangle.vertex_c();

        let point_a = Point3D::new(ax, ay, az);
        let point_b = Point3D::new(bx, by, bz);
        let point_c = Point3D::new(cx, cy, cz);
        let vertex_a = Arc::new(Vertex::new(point_a));
        let vertex_b = Arc::new(Vertex::new(point_b));
        let vertex_c = Arc::new(Vertex::new(point_c));

        let edge_ab_curve = LineSegment3D::new(point_a, point_b)
            .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;
        let edge_bc_curve = LineSegment3D::new(point_b, point_c)
            .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;
        let edge_ca_curve = LineSegment3D::new(point_c, point_a)
            .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;

        let edge_ab = Edge::new(
            vertex_a.clone(),
            vertex_b.clone(),
            CurveRef::Line(edge_ab_curve),
            (edge_ab_curve.start_param(), edge_ab_curve.end_param()),
        )
        .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;
        let edge_bc = Edge::new(
            vertex_b,
            vertex_c.clone(),
            CurveRef::Line(edge_bc_curve),
            (edge_bc_curve.start_param(), edge_bc_curve.end_param()),
        )
        .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;
        let edge_ca = Edge::new(
            vertex_c,
            vertex_a,
            CurveRef::Line(edge_ca_curve),
            (edge_ca_curve.start_param(), edge_ca_curve.end_param()),
        )
        .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;

        let tolerances = TopologyToleranceSettings::new(1.0e-9, 1.0e-9);
        let wire = Wire::new(vec![edge_ab, edge_bc, edge_ca], tolerances)
            .ok_or(TopologyOrchestrationError::InvalidTriangleTopology)?;
        let validation = TopologyValidator::new(tolerances).validate_wire(&wire);

        if !validation.is_valid() || !wire.is_closed(tolerances) {
            return Err(TopologyOrchestrationError::InvalidTriangleTopology);
        }

        Ok(CreateWireTopologyResult { wire, validation })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CreateArcTopologyRequest, CreateCircleTopologyRequest, CreateLineTopologyRequest,
        CreateTriangleTopologyRequest, DebugShapeTopologyMutationPort, LineTopologyMutationPort,
        TopologyOrchestrator,
    };
    use geo_contracts::Angle;
    use geo_primitives::{Arc3D, Circle3D, LineSegment3D, Point3D, Triangle3D};
    use geo_topology::TopologyToleranceSettings;

    #[test]
    fn create_line_topology_returns_valid_edge() {
        let line = LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0))
            .expect("line should be constructible");

        let result = TopologyOrchestrator
            .create_line_topology(CreateLineTopologyRequest { line })
            .expect("topology orchestration should succeed");

        assert!(result.validation.is_valid());
        assert_eq!(result.edge.start_vertex().point().x(), 0.0);
        assert_eq!(result.edge.end_vertex().point().x(), 2.0);
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
        assert_eq!(
            result.edge.start_vertex().id().value(),
            result.edge.end_vertex().id().value()
        );
    }

    #[test]
    fn create_triangle_topology_returns_valid_closed_wire() {
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
        assert_eq!(result.wire.edges().len(), 3);
        assert!(
            result
                .wire
                .is_closed(TopologyToleranceSettings::new(1.0e-9, 1.0e-9))
        );
    }
}
