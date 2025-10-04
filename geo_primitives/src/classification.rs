/// 幾何プリミティブの分類システム
///
/// すべてのプリミティブ形状を階層的に分類し、
/// 型安全な識別と処理を可能にする

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    // 2次元空間要素 (geometry2d)
    Point2D,
    LineSegment,
    InfiniteLine,
    Ray,
    Arc,
    PolyLine,
    BezierCurve,
    NurbsCurve,
    Circle,
    Ellipse,
    Rectangle,
    Polygon,
    Triangle,

    // 3次元空間要素 (geometry3d + surface)
    Point3D,
    Sphere,
    Cylinder,
    Cone,
    Ellipsoid,
    Torus,
    Cube,
    Plane,
    TriangleMesh,

    // 複合要素
    Composite,
    Unknown,
}

impl PrimitiveKind {
    /// プリミティブの空間次元を返す
    /// (geometry2d=2次元, geometry3d/surface=3次元)
    pub fn dimension(&self) -> u8 {
        match self {
            // 2次元空間要素 (geometry2d)
            PrimitiveKind::Point2D
            | PrimitiveKind::LineSegment
            | PrimitiveKind::InfiniteLine
            | PrimitiveKind::Ray
            | PrimitiveKind::Arc
            | PrimitiveKind::PolyLine
            | PrimitiveKind::BezierCurve
            | PrimitiveKind::NurbsCurve
            | PrimitiveKind::Circle
            | PrimitiveKind::Ellipse
            | PrimitiveKind::Rectangle
            | PrimitiveKind::Polygon
            | PrimitiveKind::Triangle => 2,

            // 3次元空間要素 (geometry3d + surface)
            PrimitiveKind::Point3D
            | PrimitiveKind::Plane
            | PrimitiveKind::Sphere
            | PrimitiveKind::Cylinder
            | PrimitiveKind::Cone
            | PrimitiveKind::Ellipsoid
            | PrimitiveKind::Torus
            | PrimitiveKind::Cube
            | PrimitiveKind::TriangleMesh => 3,

            PrimitiveKind::Composite
            | PrimitiveKind::Unknown => 255, // 特殊値
        }
    }

    /// プリミティブが閉じた形状かどうか
    pub fn is_closed(&self) -> bool {
        match self {
            PrimitiveKind::Circle
            | PrimitiveKind::Ellipse
            | PrimitiveKind::Rectangle
            | PrimitiveKind::Polygon
            | PrimitiveKind::Triangle
            | PrimitiveKind::Sphere
            | PrimitiveKind::Cylinder
            | PrimitiveKind::Cone
            | PrimitiveKind::Ellipsoid
            | PrimitiveKind::Torus
            | PrimitiveKind::Cube => true,

            // Point は開閉の概念がない
            PrimitiveKind::Point2D
            | PrimitiveKind::Point3D => false,

            _ => false,
        }
    }

    /// プリミティブがパラメトリック曲線/曲面かどうか
    pub fn is_parametric(&self) -> bool {
        match self {
            PrimitiveKind::BezierCurve
            | PrimitiveKind::NurbsCurve
            | PrimitiveKind::Circle
            | PrimitiveKind::Ellipse
            | PrimitiveKind::Sphere
            | PrimitiveKind::Cylinder
            | PrimitiveKind::Cone
            | PrimitiveKind::Ellipsoid
            | PrimitiveKind::Torus => true,

            _ => false,
        }
    }
}

/// 幾何分類のメタデータ
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryClassification {
    pub kind: PrimitiveKind,
    pub dimension: u8,
    pub is_closed: bool,
    pub is_parametric: bool,
    pub complexity: ComplexityLevel,
}

impl GeometryClassification {
    pub fn new(kind: PrimitiveKind) -> Self {
        Self {
            dimension: kind.dimension(),
            is_closed: kind.is_closed(),
            is_parametric: kind.is_parametric(),
            complexity: ComplexityLevel::from_kind(&kind),
            kind,
        }
    }
}

/// 計算複雑度レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    /// 基本形状（点、線分、円など）
    Basic,
    /// 中程度（多角形、楕円など）
    Intermediate,
    /// 複雑（NURBS、メッシュなど）
    Complex,
    /// 複合形状
    Composite,
}

impl ComplexityLevel {
    fn from_kind(kind: &PrimitiveKind) -> Self {
        match kind {
            PrimitiveKind::Point2D
            | PrimitiveKind::Point3D
            | PrimitiveKind::LineSegment
            | PrimitiveKind::InfiniteLine
            | PrimitiveKind::Ray
            | PrimitiveKind::Arc
            | PrimitiveKind::Circle
            | PrimitiveKind::Rectangle
            | PrimitiveKind::Triangle
            | PrimitiveKind::Sphere
            | PrimitiveKind::Cube
            | PrimitiveKind::Plane => ComplexityLevel::Basic,

            PrimitiveKind::PolyLine
            | PrimitiveKind::Ellipse
            | PrimitiveKind::Polygon
            | PrimitiveKind::Cylinder
            | PrimitiveKind::Cone
            | PrimitiveKind::Ellipsoid
            | PrimitiveKind::Torus => ComplexityLevel::Intermediate,

            PrimitiveKind::BezierCurve
            | PrimitiveKind::NurbsCurve
            | PrimitiveKind::TriangleMesh => ComplexityLevel::Complex,

            PrimitiveKind::Composite => ComplexityLevel::Composite,

            PrimitiveKind::Unknown => ComplexityLevel::Complex,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_kind_dimension() {
        // 2次元空間要素
        assert_eq!(PrimitiveKind::Point2D.dimension(), 2);
        assert_eq!(PrimitiveKind::LineSegment.dimension(), 2);
        assert_eq!(PrimitiveKind::Circle.dimension(), 2);

        // 3次元空間要素
        assert_eq!(PrimitiveKind::Point3D.dimension(), 3);
        assert_eq!(PrimitiveKind::Sphere.dimension(), 3);
        assert_eq!(PrimitiveKind::Ellipsoid.dimension(), 3);
        assert_eq!(PrimitiveKind::Torus.dimension(), 3);
    }

    #[test]
    fn test_primitive_kind_properties() {
        assert!(PrimitiveKind::Circle.is_closed());
        assert!(!PrimitiveKind::LineSegment.is_closed());
        assert!(PrimitiveKind::NurbsCurve.is_parametric());
        assert!(!PrimitiveKind::Triangle.is_parametric());
        assert!(PrimitiveKind::Ellipsoid.is_closed());
        assert!(PrimitiveKind::Torus.is_parametric());
    }

    #[test]
    fn test_geometry_classification() {
        let classification = GeometryClassification::new(PrimitiveKind::Circle);
        assert_eq!(classification.kind, PrimitiveKind::Circle);
        assert_eq!(classification.dimension, 2);
        assert!(classification.is_closed);
        assert!(classification.is_parametric);
        assert_eq!(classification.complexity, ComplexityLevel::Basic);
    }

    #[test]
    fn test_surface_primitives_classification() {
        let ellipsoid_class = GeometryClassification::new(PrimitiveKind::Ellipsoid);
        assert_eq!(ellipsoid_class.dimension, 3);
        assert!(ellipsoid_class.is_closed);
        assert!(ellipsoid_class.is_parametric);
        assert_eq!(ellipsoid_class.complexity, ComplexityLevel::Intermediate);

        let torus_class = GeometryClassification::new(PrimitiveKind::Torus);
        assert_eq!(torus_class.dimension, 3);
        assert!(torus_class.is_closed);
        assert!(torus_class.is_parametric);
        assert_eq!(torus_class.complexity, ComplexityLevel::Intermediate);
    }
}
