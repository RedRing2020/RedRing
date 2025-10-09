/// 幾何プリミティブの分類システム
///
/// すべてのプリミティブ形状を階層的に分類し、
/// 型安全な識別と処理を可能にする
/// 
/// 注意: 基本的な分類システムは geo_foundation に移動されました。
/// 以下のトレイト・列挙型は geo_foundation::abstract_types::geometry::classification で利用可能です:
/// - PrimitiveKind
/// - DimensionClass  
/// - GeometryPrimitive

// geo_foundation の分類システムを再エクスポート
pub use geo_foundation::abstract_types::geometry::classification::{
    DimensionClass, GeometryPrimitive, PrimitiveKind,
};

/// 幾何プリミティブのタグ付きユニオン（geo_primitives 固有の具体実装）
#[derive(Debug, Clone)]
pub enum GeometryUnion {
    Point(crate::geometry3d::point::Point3DF64),
    Vector(crate::geometry3d::Vector3D),
    //Circle(crate::CadCircle),
    //Ellipse(crate::CadEllipse),
    // 他の型は必要に応じて追加
}

impl GeometryPrimitive for GeometryUnion {
    fn kind(&self) -> PrimitiveKind {
        match self {
            GeometryUnion::Point(_) => PrimitiveKind::Point,
            GeometryUnion::Vector(_) => PrimitiveKind::LineSegment, // ベクトルは線分として扱う
                                                                    //GeometryUnion::Circle(_) => PrimitiveKind::Circle,
                                                                    //GeometryUnion::Ellipse(_) => PrimitiveKind::Ellipse,
        }
    }
}
