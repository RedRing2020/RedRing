//! Arc変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 全幾何プリミティブで共通利用可能な統一インターフェース

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D, Vector2D};
use geo_foundation::{
    traits::{AdvancedTransform, BasicTransform, TransformHelpers},
    Angle, Scalar,
};

// TODO: Matrix3型の定義が必要（将来の拡張で追加予定）
// use crate::Matrix3;

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Arc2D<T> {
    type Transformed = Arc2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        // Circle2D の translate メソッドを使用（Circle2D側にて実装済みと仮定）
        let new_circle = Circle2D::new(
            Point2D::new(
                self.circle().center().x() + translation.x(),
                self.circle().center().y() + translation.y(),
            ),
            self.circle().radius(),
        )
        .expect("Translation should preserve circle validity");

        Self::new(new_circle, self.start_angle(), self.end_angle())
            .expect("Translation should preserve arc validity")
    }

    /// 指定中心での回転
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 中心点の回転
        let center_to_arc_center = Vector2D::new(
            self.circle().center().x() - center.x(),
            self.circle().center().y() - center.y(),
        );

        // 回転行列適用（簡易実装、将来的にはMatrix2D使用）
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let rotated_x = center_to_arc_center.x() * cos_angle - center_to_arc_center.y() * sin_angle;
        let rotated_y = center_to_arc_center.x() * sin_angle + center_to_arc_center.y() * cos_angle;

        let new_center = Point2D::new(center.x() + rotated_x, center.y() + rotated_y);
        let new_circle = Circle2D::new(new_center, self.circle().radius())
            .expect("Rotation should preserve circle validity");

        // 円弧の角度も回転分だけ調整
        let new_start = self.start_angle() + angle;
        let new_end = self.end_angle() + angle;

        Self::new(new_circle, new_start, new_end).expect("Rotation should preserve arc validity")
    }

    /// 指定中心でのスケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        if factor <= T::ZERO {
            panic!("Scale factor must be positive");
        }

        // 中心点のスケール
        let center_to_arc_center = Vector2D::new(
            self.circle().center().x() - center.x(),
            self.circle().center().y() - center.y(),
        );
        let scaled_offset = Vector2D::new(
            center_to_arc_center.x() * factor,
            center_to_arc_center.y() * factor,
        );
        let new_center = Point2D::new(
            center.x() + scaled_offset.x(),
            center.y() + scaled_offset.y(),
        );

        // 半径もスケール
        let new_radius = self.circle().radius() * factor;
        let new_circle =
            Circle2D::new(new_center, new_radius).expect("Scaling should preserve circle validity");

        Self::new(new_circle, self.start_angle(), self.end_angle())
            .expect("Scaling should preserve arc validity")
    }
}

// TransformHelpers trait は自動実装される
// （BasicTransform を実装しているため）

// ============================================================================
// Extension Methods (旧実装からの移行)
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// ベクトルによる平行移動
    pub fn translate(&self, vector: &Vector2D<T>) -> Self {
        self.translate_xy(vector.x(), vector.y())
    }

    /// 指定点を中心とした回転
    pub fn rotate_around_point(&self, center: &Point2D<T>, angle: Angle<T>) -> Self {
        // 1. 回転中心に移動
        let to_origin = Vector2D::new(-center.x(), -center.y());
        let translated = self.translate(&to_origin);

        // 2. 原点中心で回転
        let rotated = translated.rotate(Point2D::origin(), angle);

        // 3. 元の位置に戻す
        let back_translation = Vector2D::new(center.x(), center.y());
        rotated.translate(&back_translation)
    }

    /// 指定点を中心とした拡大縮小
    pub fn scale_around_point(&self, center: &Point2D<T>, factor: T) -> Self {
        if factor <= T::ZERO {
            panic!("Scale factor must be positive");
        }

        // 1. スケール中心に移動
        let to_origin = Vector2D::new(-center.x(), -center.y());
        let translated = self.translate(&to_origin);

        // 2. 原点中心でスケール
        let scaled = translated.scale(Point2D::origin(), factor);

        // 3. 元の位置に戻す
        let back_translation = Vector2D::new(center.x(), center.y());
        scaled.translate(&back_translation)
    }

    /// 円弧の反転（開始と終了を入れ替え）
    pub fn reverse(&self) -> Self {
        Self::new(*self.circle(), self.end_angle(), self.start_angle())
            .expect("Reversing should preserve arc validity")
    }
}

// ============================================================================
// AdvancedTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> AdvancedTransform<T> for Arc2D<T> {
    type Line2D = LineSegment2D<T>;
    type Matrix3 = (); // TODO: Matrix3型が定義されたら置き換え

    /// 鏡像反転
    fn mirror(&self, _axis: Self::Line2D) -> Self::Transformed {
        // TODO: Line2D での鏡像反転実装
        // 現在は簡易実装として水平軸での反転を提供
        self.flip_horizontal()
    }

    /// 非等方スケール
    fn scale_non_uniform(
        &self,
        center: Self::Point2D,
        scale_x: T,
        scale_y: T,
    ) -> Self::Transformed {
        if scale_x <= T::ZERO || scale_y <= T::ZERO {
            panic!("Scale factors must be positive");
        }

        // TODO: 非等方スケールは楕円弧への変換が必要
        // 現在は平均値での等方スケールで近似
        let average_factor = (scale_x + scale_y) / (T::ONE + T::ONE);
        <Self as BasicTransform<T>>::scale(self, center, average_factor)
    }

    /// アフィン変換行列による変換
    fn transform_matrix(&self, _matrix: &Self::Matrix3) -> Self::Transformed {
        // TODO: Matrix3が実装されたら変換処理を追加
        panic!("Matrix transformation not yet implemented");
    }

    /// 反転（向きの逆転）
    fn reverse(&self) -> Self::Transformed {
        Self::new(*self.circle(), self.end_angle(), self.start_angle())
            .expect("Reversing should preserve arc validity")
    }
}

// ============================================================================
// 互換性維持のための Extension Methods
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// 水平反転（Y軸に対する鏡像）（互換性維持）
    pub fn flip_horizontal(&self) -> Self {
        let center = self.circle().center();
        let flipped_center = Point2D::new(-center.x(), center.y());
        let flipped_circle = Circle2D::new(flipped_center, self.circle().radius())
            .expect("Flip should preserve circle validity");

        // 角度も反転
        let pi = Angle::from_radians(T::PI);
        let new_start = Angle::from_radians(pi.to_radians() - self.start_angle().to_radians());
        let new_end = Angle::from_radians(pi.to_radians() - self.end_angle().to_radians());

        Self::new(flipped_circle, new_end, new_start)
            .expect("Horizontal flip should preserve arc validity")
    }

    /// 垂直反転（X軸に対する鏡像）（互換性維持）
    pub fn flip_vertical(&self) -> Self {
        let center = self.circle().center();
        let flipped_center = Point2D::new(center.x(), -center.y());
        let flipped_circle = Circle2D::new(flipped_center, self.circle().radius())
            .expect("Flip should preserve circle validity");

        // 角度も反転
        let new_start = Angle::from_radians(-self.start_angle().to_radians());
        let new_end = Angle::from_radians(-self.end_angle().to_radians());

        Self::new(flipped_circle, new_end, new_start)
            .expect("Vertical flip should preserve arc validity")
    }

    /// 均等拡大縮小（互換性維持）
    pub fn uniform_scale(&self, factor: T) -> Self {
        self.scale_origin(factor)
    }

    /// 非均等拡大縮小（互換性維持）
    pub fn non_uniform_scale(&self, x_factor: T, y_factor: T) -> Self {
        if x_factor <= T::ZERO || y_factor <= T::ZERO {
            panic!("Scale factors must be positive");
        }

        // 非均等スケールの場合、円は楕円になるため、
        // 正確な実装には楕円弧への変換が必要
        // 現在は近似として平均倍率を使用
        let average_factor = (x_factor + y_factor) / (T::ONE + T::ONE);
        self.scale_origin(average_factor)
    }
}
