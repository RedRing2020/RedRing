//! 変換操作の統一インターフェース
//!
//! 全幾何プリミティブで共通利用可能な変換操作Foundation システム
//! メンテナンス効率向上のため、統一インターフェースを提供

use crate::Scalar;

/// 基本変換操作の統一インターフェース
///
/// 全ての幾何プリミティブが実装すべき基本的な変換操作
/// 平行移動、回転、スケールの3つの基本変換を提供
pub trait BasicTransform<T: Scalar> {
    /// 変換後の型（通常は Self と同じ）
    type Transformed;

    /// 2D ベクトル型
    type Vector2D;

    /// 2D 点型
    type Point2D;

    /// 角度型
    type Angle;

    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しいオブジェクト
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed;

    /// 指定中心での回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいオブジェクト
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed;

    /// 指定中心でのスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しいオブジェクト
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed;
}

/// 高度変換操作の拡張インターフェース
///
/// より高度な変換操作を提供する拡張トレイト
/// 鏡像反転、非等方スケール、行列変換等を含む
pub trait AdvancedTransform<T: Scalar>: BasicTransform<T> {
    /// 2D 直線型
    type Line2D;

    /// 3x3 変換行列型
    type Matrix3;

    /// 鏡像反転
    ///
    /// # 引数
    /// * `axis` - 反転軸となる直線
    ///
    /// # 戻り値
    /// 鏡像反転された新しいオブジェクト
    fn mirror(&self, axis: Self::Line2D) -> Self::Transformed;

    /// 非等方スケール（X軸・Y軸で異なる倍率）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    ///
    /// # 戻り値
    /// 非等方スケールされた新しいオブジェクト
    fn scale_non_uniform(&self, center: Self::Point2D, scale_x: T, scale_y: T)
        -> Self::Transformed;

    /// アフィン変換行列による変換
    ///
    /// # 引数
    /// * `matrix` - 3x3 アフィン変換行列
    ///
    /// # 戻り値
    /// 行列変換された新しいオブジェクト
    fn transform_matrix(&self, matrix: &Self::Matrix3) -> Self::Transformed;

    /// 反転（向きの逆転）
    ///
    /// Arc、LineSegment等の向きを持つ幾何要素に適用
    /// Circle等の向きを持たない要素では self を返す
    ///
    /// # 戻り値
    /// 向きが反転された新しいオブジェクト
    fn reverse(&self) -> Self::Transformed;
}

/// 便利メソッドを提供するヘルパートレイト
///
/// よく使用される特定パラメータでの変換操作のデフォルト実装を提供
/// `BasicTransform` を実装した型に対して自動的に実装される
pub trait TransformHelpers<T: Scalar>: BasicTransform<T> {
    /// 原点中心での回転
    ///
    /// # 引数
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 原点中心で回転された新しいオブジェクト
    fn rotate_origin(&self, angle: Self::Angle) -> Self::Transformed;

    /// 原点中心でのスケール
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// 原点中心でスケールされた新しいオブジェクト
    fn scale_origin(&self, factor: T) -> Self::Transformed;

    /// X軸方向への平行移動
    ///
    /// # 引数
    /// * `dx` - X軸方向の移動量
    ///
    /// # 戻り値
    /// X軸方向に移動された新しいオブジェクト
    fn translate_x(&self, dx: T) -> Self::Transformed;

    /// Y軸方向への平行移動
    ///
    /// # 引数
    /// * `dy` - Y軸方向の移動量
    ///
    /// # 戻り値
    /// Y軸方向に移動された新しいオブジェクト
    fn translate_y(&self, dy: T) -> Self::Transformed;

    /// XY両方向への平行移動
    ///
    /// # 引数
    /// * `dx` - X軸方向の移動量
    /// * `dy` - Y軸方向の移動量
    ///
    /// # 戻り値
    /// XY方向に移動された新しいオブジェクト
    fn translate_xy(&self, dx: T, dy: T) -> Self::Transformed;
}

// TransformHelpers の自動実装
// BasicTransform を実装している型に対して自動的に便利メソッドを提供
impl<T: Scalar, U> TransformHelpers<T> for U
where
    U: BasicTransform<T>,
    // 型制約: 必要な型が構築可能であること
    U::Vector2D: From<(T, T)>,
    U::Point2D: Default,
{
    fn rotate_origin(&self, angle: Self::Angle) -> Self::Transformed {
        self.rotate(Self::Point2D::default(), angle)
    }

    fn scale_origin(&self, factor: T) -> Self::Transformed {
        self.scale(Self::Point2D::default(), factor)
    }

    fn translate_x(&self, dx: T) -> Self::Transformed {
        self.translate(U::Vector2D::from((dx, T::ZERO)))
    }

    fn translate_y(&self, dy: T) -> Self::Transformed {
        self.translate(U::Vector2D::from((T::ZERO, dy)))
    }

    fn translate_xy(&self, dx: T, dy: T) -> Self::Transformed {
        self.translate(U::Vector2D::from((dx, dy)))
    }
}

/// 3D 変換操作の基本インターフェース
///
/// 3D 幾何プリミティブ用の基本変換操作
/// 将来の3D対応時に使用予定
pub trait BasicTransform3D<T: Scalar> {
    /// 変換後の型
    type Transformed;

    /// 3D ベクトル型
    type Vector3D;

    /// 3D 点型
    type Point3D;

    /// 3D 回転型（クォータニオンまたはオイラー角）
    type Rotation3D;

    /// 3D 平行移動
    fn translate_3d(&self, translation: Self::Vector3D) -> Self::Transformed;

    /// 3D 回転
    fn rotate_3d(&self, center: Self::Point3D, rotation: Self::Rotation3D) -> Self::Transformed;

    /// 3D 等方スケール
    fn scale_3d(&self, center: Self::Point3D, factor: T) -> Self::Transformed;
}
