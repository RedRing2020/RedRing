/// 幾何プリミティブの共通トレイト
/// 
/// 全ての幾何プリミティブが実装すべき基本的なインターフェース

use crate::geometry3d::{BBox3D, Point3D};
use super::classification::PrimitiveKind;

/// 全ての幾何プリミティブが実装する共通トレイト
pub trait GeometricPrimitive {
    /// プリミティブの種類を返す
    fn primitive_kind(&self) -> PrimitiveKind;

    /// バウンディングボックスを返す
    fn bounding_box(&self) -> BBox3D;

    /// プリミティブの測定値（長さ、面積、体積など）を返す
    fn measure(&self) -> Option<f64>;
}

/// 変形可能な幾何プリミティブのトレイト
pub trait TransformablePrimitive: GeometricPrimitive {
    /// 平行移動
    fn translate(&mut self, offset: (f64, f64, f64));
    
    /// スケール変換
    fn scale(&mut self, factor: f64);
    
    /// 回転（オイラー角、ラジアン）
    fn rotate(&mut self, angles: (f64, f64, f64));
}

/// 測定可能な幾何プリミティブのトレイト
pub trait MeasurablePrimitive: GeometricPrimitive {
    /// 表面積を計算（適用可能な場合）
    fn surface_area(&self) -> Option<f64>;
    
    /// 体積を計算（適用可能な場合）
    fn volume(&self) -> Option<f64>;
    
    /// 周囲長/周長を計算（適用可能な場合）
    fn perimeter(&self) -> Option<f64>;
}

/// 幾何プリミティブのコレクション操作
pub trait PrimitiveCollection {
    type Item: GeometricPrimitive;
    
    /// 全プリミティブの結合境界ボックス
    fn combined_bounding_box(&self) -> Option<BBox3D>;
    
    /// 指定した点に最も近いプリミティブを取得
    fn nearest_to_point(&self, point: (f64, f64, f64)) -> Option<&Self::Item>;
    
    /// 指定した境界ボックスと交差するプリミティブを取得
    fn intersecting_with_bbox(&self, bbox: &BBox3D) -> Vec<&Self::Item>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::common::classification::PrimitiveKind;

    // テスト用のモックプリミティブ
    struct MockPrimitive {
        kind: PrimitiveKind,
        bbox: BBox3D,
        measure: Option<f64>,
    }

    impl GeometricPrimitive for MockPrimitive {
        fn primitive_kind(&self) -> PrimitiveKind {
            self.kind
        }

        fn bounding_box(&self) -> BBox3D {
            self.bbox.clone()
        }

        fn measure(&self) -> Option<f64> {
            self.measure
        }
    }

    #[test]
    fn test_geometric_primitive_interface() {
        let primitive = MockPrimitive {
            kind: PrimitiveKind::Point,
            bbox: BBox3D::new((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)),
            measure: Some(5.0),
        };

        assert_eq!(primitive.primitive_kind(), PrimitiveKind::Point);
        assert_eq!(primitive.measure(), Some(5.0));
        
        let bbox = primitive.bounding_box();
        assert_eq!(bbox.min, Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max, Point3D::new(1.0, 1.0, 1.0));
    }
}