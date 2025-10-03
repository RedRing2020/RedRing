/// 幾何プリミティブの分類システム
/// 
/// すべてのプリミティブ形状を階層的に分類し、
/// 型安全な識別と処理を可能にする

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    // 0次元: 点
    Point,
    
    // 1次元: 線形要素
    LineSegment,
    PolyLine,
    BezierCurve,
    NurbsCurve,
    
    // 2次元: 面要素
    Circle,
    Ellipse,
    Rectangle,
    Polygon,
    Triangle,
    
    // 3次元: 立体要素
    Sphere,
    Cylinder,
    Cone,
    Cube,
    Plane,
    TriangleMesh,
    
    // 複合要素
    Composite,
    Unknown,
}

impl PrimitiveKind {
    /// プリミティブの次元を返す
    pub fn dimension(&self) -> u8 {
        match self {
            PrimitiveKind::Point => 0,
            
            PrimitiveKind::LineSegment 
            | PrimitiveKind::PolyLine 
            | PrimitiveKind::BezierCurve 
            | PrimitiveKind::NurbsCurve => 1,
            
            PrimitiveKind::Circle 
            | PrimitiveKind::Ellipse 
            | PrimitiveKind::Rectangle 
            | PrimitiveKind::Polygon 
            | PrimitiveKind::Triangle 
            | PrimitiveKind::Plane => 2,
            
            PrimitiveKind::Sphere 
            | PrimitiveKind::Cylinder 
            | PrimitiveKind::Cone 
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
            | PrimitiveKind::Cube => true,
            
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
            | PrimitiveKind::Cone => true,
            
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
            PrimitiveKind::Point 
            | PrimitiveKind::LineSegment 
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
            | PrimitiveKind::Cone => ComplexityLevel::Intermediate,
            
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
        assert_eq!(PrimitiveKind::Point.dimension(), 0);
        assert_eq!(PrimitiveKind::LineSegment.dimension(), 1);
        assert_eq!(PrimitiveKind::Circle.dimension(), 2);
        assert_eq!(PrimitiveKind::Sphere.dimension(), 3);
    }

    #[test]
    fn test_primitive_kind_properties() {
        assert!(PrimitiveKind::Circle.is_closed());
        assert!(!PrimitiveKind::LineSegment.is_closed());
        assert!(PrimitiveKind::NurbsCurve.is_parametric());
        assert!(!PrimitiveKind::Triangle.is_parametric());
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
}