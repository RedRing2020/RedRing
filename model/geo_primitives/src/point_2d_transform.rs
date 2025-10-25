//! Point2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 2D点の基本的な変換機能を提供
//!
//! ## 設計方針
//!
//! このファイルは公開API層として機能し、内部的に安全な実装
//! (`point_2d_transform_safe.rs`) を呼び出します：
//!
//! - **BasicTransform trait**: パニックしない安全なインターフェース
//! - **エラー時の動作**: 元の点をcloneして返す（後方互換性維持）
//! - **詳細なエラーハンドリング**: `safe_*` メソッドを直接使用
//!
//! ## 使用例
//!
//! ```rust
//! use geo_primitives::{Point2D, Vector2D};
//! use geo_foundation::{extensions::BasicTransform, Angle};
//!
//! let point = Point2D::new(1.0, 2.0);
//! let center = Point2D::origin();
//! let angle = Angle::from_radians(std::f64::consts::PI / 4.0);
//!
//! // BasicTransformトレイトを使用した変換（エラー時は元の点を返す）
//! let rotated = BasicTransform::rotate(&point, center, angle);
//!
//! // エラーハンドリングが必要な場合
//! match point.safe_rotate(center, angle) {
//!     Ok(result) => println!("回転成功: {:?}", result),
//!     Err(e) => println!("回転失敗: {}", e),
//! }
//! ```

use crate::{Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Point2D<T> {
    type Transformed = Point2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい点
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        Point2D::new(self.x() + translation.x(), self.y() + translation.y())
    }

    /// 指定中心での回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい点
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 内部的に安全なメソッドを使用し、エラー時は元の点を返す
        self.safe_rotate(center, angle).unwrap_or(*self)
    }

    /// 指定中心でのスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい点
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // 内部的に安全なメソッドを使用し、エラー時は元の点を返す
        self.safe_scale(center, factor).unwrap_or(*self)
    }
}

// ============================================================================
// Required implementations for BasicTransform
// Note: Default trait is already implemented in point_2d.rs
// ============================================================================
