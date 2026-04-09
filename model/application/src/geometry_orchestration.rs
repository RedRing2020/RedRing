//! Geometry向け orchestration 境界。

use crate::primitives::{Angle, Arc3D, Circle3D, LineSegment3D, Point3D, Triangle3D};

/// 将来の geometry orchestration ユースケース向けマーカー入力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryOrchestrationRequest {
    pub operation_name: String,
}

/// line geometry 生成の入力境界。
#[derive(Debug, Clone, PartialEq)]
pub struct CreateLineGeometryRequest {
    pub start: [f64; 3],
    pub end: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLineGeometryResult {
    pub line: LineSegment3D<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCircleGeometryRequest {
    pub center: [f64; 3],
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCircleGeometryResult {
    pub circle: Circle3D<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateArcGeometryRequest {
    pub center: [f64; 3],
    pub radius: f64,
    pub start_angle_radians: f64,
    pub end_angle_radians: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateArcGeometryResult {
    pub arc: Arc3D<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTriangleGeometryRequest {
    pub vertex_a: [f64; 3],
    pub vertex_b: [f64; 3],
    pub vertex_c: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTriangleGeometryResult {
    pub triangle: Triangle3D<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryOrchestrationError {
    InvalidLineEndpoints,
    InvalidCircleParameters,
    InvalidArcParameters,
    InvalidTriangleVertices,
}

impl std::fmt::Display for GeometryOrchestrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLineEndpoints => write!(f, "invalid line endpoints"),
            Self::InvalidCircleParameters => write!(f, "invalid circle parameters"),
            Self::InvalidArcParameters => write!(f, "invalid arc parameters"),
            Self::InvalidTriangleVertices => write!(f, "invalid triangle vertices"),
        }
    }
}

impl std::error::Error for GeometryOrchestrationError {}

pub trait LineGeometryMutationPort {
    fn create_line_geometry(
        &self,
        request: CreateLineGeometryRequest,
    ) -> Result<CreateLineGeometryResult, GeometryOrchestrationError>;
}

pub trait DebugShapeGeometryMutationPort {
    fn create_circle_geometry(
        &self,
        request: CreateCircleGeometryRequest,
    ) -> Result<CreateCircleGeometryResult, GeometryOrchestrationError>;

    fn create_arc_geometry(
        &self,
        request: CreateArcGeometryRequest,
    ) -> Result<CreateArcGeometryResult, GeometryOrchestrationError>;

    fn create_triangle_geometry(
        &self,
        request: CreateTriangleGeometryRequest,
    ) -> Result<CreateTriangleGeometryResult, GeometryOrchestrationError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GeometryOrchestrator;

impl LineGeometryMutationPort for GeometryOrchestrator {
    fn create_line_geometry(
        &self,
        request: CreateLineGeometryRequest,
    ) -> Result<CreateLineGeometryResult, GeometryOrchestrationError> {
        let start = Point3D::new(request.start[0], request.start[1], request.start[2]);
        let end = Point3D::new(request.end[0], request.end[1], request.end[2]);
        let line = LineSegment3D::new(start, end)
            .ok_or(GeometryOrchestrationError::InvalidLineEndpoints)?;

        Ok(CreateLineGeometryResult { line })
    }
}

impl DebugShapeGeometryMutationPort for GeometryOrchestrator {
    fn create_circle_geometry(
        &self,
        request: CreateCircleGeometryRequest,
    ) -> Result<CreateCircleGeometryResult, GeometryOrchestrationError> {
        let center = Point3D::new(request.center[0], request.center[1], request.center[2]);
        let circle = Circle3D::new_xy_plane(center, request.radius)
            .ok_or(GeometryOrchestrationError::InvalidCircleParameters)?;

        Ok(CreateCircleGeometryResult { circle })
    }

    fn create_arc_geometry(
        &self,
        request: CreateArcGeometryRequest,
    ) -> Result<CreateArcGeometryResult, GeometryOrchestrationError> {
        let center = Point3D::new(request.center[0], request.center[1], request.center[2]);
        let arc = Arc3D::xy_arc(
            center,
            request.radius,
            Angle::from_radians(request.start_angle_radians),
            Angle::from_radians(request.end_angle_radians),
        )
        .ok_or(GeometryOrchestrationError::InvalidArcParameters)?;

        Ok(CreateArcGeometryResult { arc })
    }

    fn create_triangle_geometry(
        &self,
        request: CreateTriangleGeometryRequest,
    ) -> Result<CreateTriangleGeometryResult, GeometryOrchestrationError> {
        let vertex_a = Point3D::new(
            request.vertex_a[0],
            request.vertex_a[1],
            request.vertex_a[2],
        );
        let vertex_b = Point3D::new(
            request.vertex_b[0],
            request.vertex_b[1],
            request.vertex_b[2],
        );
        let vertex_c = Point3D::new(
            request.vertex_c[0],
            request.vertex_c[1],
            request.vertex_c[2],
        );

        let triangle = Triangle3D::new(vertex_a, vertex_b, vertex_c)
            .ok_or(GeometryOrchestrationError::InvalidTriangleVertices)?;

        Ok(CreateTriangleGeometryResult { triangle })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CreateArcGeometryRequest, CreateCircleGeometryRequest, CreateLineGeometryRequest,
        CreateTriangleGeometryRequest, DebugShapeGeometryMutationPort, GeometryOrchestrationError,
        GeometryOrchestrator, LineGeometryMutationPort,
    };
    use geo_contracts::{Arc3DProperties, Circle3DProperties, Triangle3DBoundaryAccess};

    #[test]
    fn create_line_geometry_returns_line_segment() {
        let result = GeometryOrchestrator
            .create_line_geometry(CreateLineGeometryRequest {
                start: [0.0, 0.0, 0.0],
                end: [10.0, 0.0, 0.0],
            })
            .expect("line geometry creation should succeed");

        assert_eq!(result.line.start().x(), 0.0);
        assert_eq!(result.line.end().x(), 10.0);
    }

    #[test]
    fn create_line_geometry_rejects_identical_endpoints() {
        let result = GeometryOrchestrator.create_line_geometry(CreateLineGeometryRequest {
            start: [1.0, 1.0, 1.0],
            end: [1.0, 1.0, 1.0],
        });

        assert_eq!(
            result,
            Err(GeometryOrchestrationError::InvalidLineEndpoints)
        );
    }

    #[test]
    fn create_circle_geometry_returns_circle() {
        let result = GeometryOrchestrator
            .create_circle_geometry(CreateCircleGeometryRequest {
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
            })
            .expect("circle geometry creation should succeed");

        assert_eq!(result.circle.radius(), 5.0);
    }

    #[test]
    fn create_arc_geometry_returns_arc() {
        let result = GeometryOrchestrator
            .create_arc_geometry(CreateArcGeometryRequest {
                center: [0.0, 0.0, 0.0],
                radius: 1.5,
                start_angle_radians: 0.0,
                end_angle_radians: std::f64::consts::FRAC_PI_2,
            })
            .expect("arc geometry creation should succeed");

        assert_eq!(result.arc.radius(), 1.5);
    }

    #[test]
    fn create_triangle_geometry_returns_triangle() {
        let result = GeometryOrchestrator
            .create_triangle_geometry(CreateTriangleGeometryRequest {
                vertex_a: [0.0, 0.0, 0.0],
                vertex_b: [1.0, 0.0, 0.0],
                vertex_c: [0.5, 1.0, 0.0],
            })
            .expect("triangle geometry creation should succeed");

        assert_eq!(result.triangle.vertex_a(), (0.0, 0.0, 0.0));
    }
}
