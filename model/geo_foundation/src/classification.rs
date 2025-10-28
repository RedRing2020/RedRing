//! 幾何プリミティブの分類システム
//!
//! すべてのプリミティブ形状を階層的に分類し、
//! 型安全な識別と処理を可能にする

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    // 0次元: 点
    Point,

    // 1次元: 線形要素
    LineSegment,
    PolyLine,
    BezierCurve,
    NurbsCurve,
    Arc,
    Ray,

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
    NurbsSurface,

    // 複合要素
    Group,
    Assembly,

    // 補助要素
    BBox,
    Vector,
    Mesh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionClass {
    Zero,    // Point
    One,     // Curve
    Two,     // Surface
    Three,   // Solid
    Complex, // Group/Assembly
}

impl PrimitiveKind {
    /// 次元を取得
    pub fn dimension(&self) -> DimensionClass {
        match self {
            PrimitiveKind::Point => DimensionClass::Zero,

            PrimitiveKind::LineSegment
            | PrimitiveKind::PolyLine
            | PrimitiveKind::BezierCurve
            | PrimitiveKind::NurbsCurve
            | PrimitiveKind::Arc
            | PrimitiveKind::Ray => DimensionClass::One,

            PrimitiveKind::Circle
            | PrimitiveKind::Ellipse
            | PrimitiveKind::Rectangle
            | PrimitiveKind::Polygon
            | PrimitiveKind::Triangle
            | PrimitiveKind::Plane
            | PrimitiveKind::NurbsSurface => DimensionClass::Two,

            PrimitiveKind::Sphere
            | PrimitiveKind::Cylinder
            | PrimitiveKind::Cone
            | PrimitiveKind::Cube
            | PrimitiveKind::TriangleMesh
            | PrimitiveKind::Mesh => DimensionClass::Three,

            PrimitiveKind::Group | PrimitiveKind::Assembly => DimensionClass::Complex,

            PrimitiveKind::BBox | PrimitiveKind::Vector => DimensionClass::Complex,
        }
    }

    /// 曲線系かどうか
    pub fn is_curve(&self) -> bool {
        matches!(self.dimension(), DimensionClass::One)
    }

    /// 面系かどうか
    pub fn is_surface(&self) -> bool {
        matches!(self.dimension(), DimensionClass::Two)
    }

    /// 立体系かどうか
    pub fn is_solid(&self) -> bool {
        matches!(self.dimension(), DimensionClass::Three)
    }

    /// パラメトリック形状かどうか
    pub fn is_parametric(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::BezierCurve | PrimitiveKind::NurbsCurve | PrimitiveKind::NurbsSurface
        )
    }

    /// 解析的形状かどうか
    pub fn is_analytical(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Circle
                | PrimitiveKind::Ellipse
                | PrimitiveKind::Sphere
                | PrimitiveKind::Cylinder
                | PrimitiveKind::Cone
                | PrimitiveKind::Plane
        )
    }

    /// 多角形/メッシュ系かどうか
    pub fn is_mesh(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Polygon | PrimitiveKind::Triangle | PrimitiveKind::TriangleMesh
        )
    }
}

/// 幾何プリミティブの基本トレイト
pub trait GeometryPrimitive {
    /// プリミティブの種類を返す
    fn kind(&self) -> PrimitiveKind;

    /// 次元を返す
    fn dimension(&self) -> DimensionClass {
        self.kind().dimension()
    }
}
