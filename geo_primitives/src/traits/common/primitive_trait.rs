//! 幾何プリミティブの共通トレイト
//!
//! 全ての幾何プリミティブが実装すべき基本的なインターフェース

use super::classification::PrimitiveKind;
use crate::geometry3d::BBox3D;
use geo_foundation::abstract_types::Scalar;

/// 全ての幾何プリミティブが実装する共通トレイト（ジェネリック版）
pub trait GeometricPrimitive<T: Scalar = f64> {
    /// プリミティブの種類を返す
    fn primitive_kind(&self) -> PrimitiveKind;

    /// バウンディングボックスを返す（ジェネリック版）
    fn bounding_box(&self) -> BBox3D<T>;

    /// プリミティブの測定値（長さ、面積、体積など）を返す（ジェネリック版）
    fn measure(&self) -> Option<T>;
}

/// 変形可能な幾何プリミティブのトレイト（ジェネリック版）
pub trait TransformablePrimitive<T: Scalar = f64>: GeometricPrimitive<T> {
    /// 平行移動（ジェネリック版）
    fn translate(&mut self, offset: (T, T, T));

    /// スケール変換（ジェネリック版）
    fn scale(&mut self, factor: T);

    /// 回転（オイラー角、ラジアン、ジェネリック版）
    fn rotate(&mut self, angles: (T, T, T));
}

/// 測定可能な幾何プリミティブのトレイト（ジェネリック版）
pub trait MeasurablePrimitive<T: Scalar = f64>: GeometricPrimitive<T> {
    /// 表面積を計算（適用可能な場合、ジェネリック版）
    fn surface_area(&self) -> Option<T>;

    /// 体積を計算（適用可能な場合、ジェネリック版）
    fn volume(&self) -> Option<T>;

    /// 周囲長/周長を計算（適用可能な場合、ジェネリック版）
    fn perimeter(&self) -> Option<T>;
}

/// 幾何プリミティブのコレクション操作（ジェネリック版）
pub trait PrimitiveCollection<T: Scalar = f64> {
    type Item: GeometricPrimitive<T>;

    /// 全プリミティブの結合境界ボックス（ジェネリック版）
    fn combined_bounding_box(&self) -> Option<BBox3D<T>>;

    /// 指定した点に最も近いプリミティブを取得（ジェネリック版）
    fn nearest_to_point(&self, point: (T, T, T)) -> Option<&Self::Item>;

    /// 指定した境界ボックスと交差するプリミティブを取得（ジェネリック版）
    fn intersecting_with_bbox(&self, bbox: &BBox3D<T>) -> Vec<&Self::Item>;
}
