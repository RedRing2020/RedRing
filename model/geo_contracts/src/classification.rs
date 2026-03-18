//! Classification contracts for geometry primitives.
// geo_foundation の分類定義を contracts 層へ段階移設するための先行コピー。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Point,
    LineSegment,
    PolyLine,
    BezierCurve,
    NurbsCurve,
    NurbsCurve2D,
    NurbsCurve3D,
    Arc,
    Ray,
    InfiniteLine,
    Circle,
    Ellipse,
    Rectangle,
    Polygon,
    Triangle,
    Sphere,
    SphericalSolid,
    SphericalSurface,
    EllipsoidalSolid,
    EllipsoidalSurface,
    Cylinder,
    CylindricalSolid,
    CylindricalSurface,
    Cone,
    ConicalSolid,
    ConicalSurface,
    TorusSolid,
    TorusSurface,
    Cube,
    Plane,
    TriangleMesh,
    NurbsSurface,
    NurbsSurface3D,
    Group,
    Assembly,
    BBox,
    Vector,
    Mesh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionClass {
    Zero,
    One,
    Two,
    Three,
    Complex,
}

impl PrimitiveKind {
    pub fn dimension(&self) -> DimensionClass {
        match self {
            PrimitiveKind::Point => DimensionClass::Zero,
            PrimitiveKind::LineSegment
            | PrimitiveKind::PolyLine
            | PrimitiveKind::BezierCurve
            | PrimitiveKind::NurbsCurve
            | PrimitiveKind::NurbsCurve2D
            | PrimitiveKind::NurbsCurve3D
            | PrimitiveKind::Arc
            | PrimitiveKind::Ray
            | PrimitiveKind::InfiniteLine => DimensionClass::One,
            PrimitiveKind::Circle
            | PrimitiveKind::Ellipse
            | PrimitiveKind::Rectangle
            | PrimitiveKind::Polygon
            | PrimitiveKind::Triangle
            | PrimitiveKind::Plane
            | PrimitiveKind::CylindricalSurface
            | PrimitiveKind::SphericalSurface
            | PrimitiveKind::EllipsoidalSurface
            | PrimitiveKind::ConicalSurface
            | PrimitiveKind::TorusSurface
            | PrimitiveKind::NurbsSurface
            | PrimitiveKind::NurbsSurface3D => DimensionClass::Two,
            PrimitiveKind::Sphere
            | PrimitiveKind::SphericalSolid
            | PrimitiveKind::EllipsoidalSolid
            | PrimitiveKind::Cylinder
            | PrimitiveKind::CylindricalSolid
            | PrimitiveKind::Cone
            | PrimitiveKind::ConicalSolid
            | PrimitiveKind::TorusSolid
            | PrimitiveKind::Cube
            | PrimitiveKind::TriangleMesh
            | PrimitiveKind::Mesh => DimensionClass::Three,
            PrimitiveKind::Group
            | PrimitiveKind::Assembly
            | PrimitiveKind::BBox
            | PrimitiveKind::Vector => DimensionClass::Complex,
        }
    }

    pub fn is_curve(&self) -> bool {
        matches!(self.dimension(), DimensionClass::One)
    }

    pub fn is_surface(&self) -> bool {
        matches!(self.dimension(), DimensionClass::Two)
    }

    pub fn is_solid(&self) -> bool {
        matches!(self.dimension(), DimensionClass::Three)
    }

    pub fn is_parametric(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::BezierCurve | PrimitiveKind::NurbsCurve | PrimitiveKind::NurbsSurface
        )
    }

    pub fn is_analytical(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Circle
                | PrimitiveKind::Ellipse
                | PrimitiveKind::Sphere
                | PrimitiveKind::EllipsoidalSolid
                | PrimitiveKind::EllipsoidalSurface
                | PrimitiveKind::SphericalSolid
                | PrimitiveKind::SphericalSurface
                | PrimitiveKind::Cylinder
                | PrimitiveKind::CylindricalSolid
                | PrimitiveKind::CylindricalSurface
                | PrimitiveKind::Cone
                | PrimitiveKind::ConicalSolid
                | PrimitiveKind::ConicalSurface
                | PrimitiveKind::TorusSolid
                | PrimitiveKind::TorusSurface
                | PrimitiveKind::Plane
        )
    }

    pub fn is_mesh(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Polygon | PrimitiveKind::Triangle | PrimitiveKind::TriangleMesh
        )
    }
}

pub trait GeometryPrimitive {
    fn kind(&self) -> PrimitiveKind;

    fn dimension(&self) -> DimensionClass {
        self.kind().dimension()
    }
}
