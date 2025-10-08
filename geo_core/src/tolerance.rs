//! トレラントモデリングにおける許容誤差管理
//!
//! CAD/CAMシステムにおける数値的堅牢性を保証するため、
//! 様々な種類の許容誤差を構造化して管理する。
use std::fmt;

/// 幾何学的計算での実用的な許容誤差
/// 機械精度よりも大きく、CAD/CAM用途に適した値
///
/// この値は以下の用途で使用される：
/// - 点の一致判定
/// - 線分の交差判定
/// - 面の共面判定
/// - 幾何学的な近似計算
pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;

/// 許容誤差コンテキスト - 各種許容誤差の統合管理
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToleranceContext {
    /// 幾何的長さの許容誤差（メートル単位）
    pub linear: f64,

    /// 角度の許容誤差（ラジアン単位）
    pub angular: f64,

    /// パラメータ空間での許容誤差
    pub parametric: f64,

    /// 曲率の許容誤差（1/m単位）
    pub curvature: f64,

    /// 面積の許容誤差（m²単位）
    pub area: f64,

    /// 体積の許容誤差（m³単位）
    pub volume: f64,
}

impl ToleranceContext {
    /// 標準的な許容誤差コンテキストを作成
    pub const fn standard() -> Self {
        Self {
            linear: 1e-6,      // 1マイクロメートル
            angular: 1e-8,     // 約0.0057度
            parametric: 1e-10, // パラメータ空間
            curvature: 1e-3,   // 曲率許容誤差
            area: 1e-12,       // 面積許容誤差
            volume: 1e-18,     // 体積許容誤差
        }
    }

    /// 高精度許容誤差コンテキストを作成
    pub const fn high_precision() -> Self {
        Self {
            linear: 1e-9,      // 1ナノメートル
            angular: 1e-10,    // より高精度な角度
            parametric: 1e-12, // より高精度なパラメータ
            curvature: 1e-6,   // 高精度曲率
            area: 1e-18,       // 高精度面積
            volume: 1e-27,     // 高精度体積
        }
    }

    /// 低精度許容誤差コンテキストを作成（プロトタイピング用）
    pub const fn low_precision() -> Self {
        Self {
            linear: 1e-3,     // 1ミリメートル
            angular: 1e-6,    // 低精度角度
            parametric: 1e-8, // 低精度パラメータ
            curvature: 1e-1,  // 低精度曲率
            area: 1e-6,       // 低精度面積
            volume: 1e-9,     // 低精度体積
        }
    }

    /// スケールファクターを適用
    pub fn scaled(&self, scale: f64) -> Self {
        Self {
            linear: self.linear * scale,
            angular: self.angular, // 角度は無次元なのでスケールしない
            parametric: self.parametric,
            curvature: self.curvature / scale, // 曲率は1/長さなので逆比例
            area: self.area * scale * scale,
            volume: self.volume * scale * scale * scale,
        }
    }

    /// 許容誤差の厳しさを調整
    pub fn tightened(&self, factor: f64) -> Self {
        debug_assert!(
            factor > 0.0 && factor <= 1.0,
            "Tightening factor must be in (0, 1]"
        );
        Self {
            linear: self.linear * factor,
            angular: self.angular * factor,
            parametric: self.parametric * factor,
            curvature: self.curvature * factor,
            area: self.area * factor,
            volume: self.volume * factor,
        }
    }
}

impl Default for ToleranceContext {
    fn default() -> Self {
        Self::standard()
    }
}

impl fmt::Display for ToleranceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ToleranceContext {{ linear: {:.2e}, angular: {:.2e}, parametric: {:.2e} }}",
            self.linear, self.angular, self.parametric
        )
    }
}

/// 許容誤差プロバイダー - 幾何オブジェクトが許容誤差を提供する能力
pub trait ToleranceProvider {
    /// 使用する許容誤差コンテキストを取得
    fn tolerance_context(&self) -> &ToleranceContext;

    /// 許容誤差コンテキストを設定
    fn set_tolerance_context(&mut self, context: ToleranceContext);
}

/// トレラント比較 - 許容誤差を考慮した等価性判定
pub trait TolerantEq<Rhs = Self> {
    /// 許容誤差を考慮した等価判定
    fn tolerant_eq(&self, other: &Rhs, context: &ToleranceContext) -> bool;

    /// 許容誤差を考慮した非等価判定
    fn tolerant_ne(&self, other: &Rhs, context: &ToleranceContext) -> bool {
        !self.tolerant_eq(other, context)
    }
}

/// トレラント順序比較
pub trait TolerantOrd<Rhs = Self>: TolerantEq<Rhs> {
    /// 許容誤差を考慮した順序比較
    fn tolerant_cmp(&self, other: &Rhs, context: &ToleranceContext) -> Option<std::cmp::Ordering>;

    fn tolerant_lt(&self, other: &Rhs, context: &ToleranceContext) -> bool {
        matches!(
            self.tolerant_cmp(other, context),
            Some(std::cmp::Ordering::Less)
        )
    }

    fn tolerant_le(&self, other: &Rhs, context: &ToleranceContext) -> bool {
        !matches!(
            self.tolerant_cmp(other, context),
            Some(std::cmp::Ordering::Greater)
        )
    }

    fn tolerant_gt(&self, other: &Rhs, context: &ToleranceContext) -> bool {
        matches!(
            self.tolerant_cmp(other, context),
            Some(std::cmp::Ordering::Greater)
        )
    }

    fn tolerant_ge(&self, other: &Rhs, context: &ToleranceContext) -> bool {
        !matches!(
            self.tolerant_cmp(other, context),
            Some(std::cmp::Ordering::Less)
        )
    }
}

/// 基本データ型への TolerantEq 実装
impl TolerantEq for f64 {
    fn tolerant_eq(&self, other: &f64, context: &ToleranceContext) -> bool {
        (self - other).abs() < context.linear
    }
}

impl TolerantOrd for f64 {
    fn tolerant_cmp(&self, other: &f64, context: &ToleranceContext) -> Option<std::cmp::Ordering> {
        let diff = self - other;
        if diff.abs() < context.linear {
            Some(std::cmp::Ordering::Equal)
        } else if diff < 0.0 {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

/// 位相処理のための基盤トレイト（将来拡張用）
pub trait TopologicalEntity {
    /// 位相的次元（0:点, 1:線, 2:面, 3:立体）
    fn topological_dimension(&self) -> u8;

    /// 境界要素の取得
    fn boundary_entities(&self) -> Vec<Box<dyn TopologicalEntity>>;

    /// 隣接関係の判定
    fn is_adjacent_to(&self, other: &dyn TopologicalEntity, context: &ToleranceContext) -> bool;
}

/// トレラント幾何 - 許容誤差を考慮した幾何操作
pub trait TolerantGeometry: ToleranceProvider {
    /// 点が幾何要素に含まれるかの判定
    fn contains_point(&self, point: &dyn std::any::Any, context: &ToleranceContext) -> bool;

    /// 他の幾何要素との距離計算
    fn distance_to(&self, other: &dyn TolerantGeometry, context: &ToleranceContext) -> f64;

    /// 他の幾何要素との交差判定
    fn intersects_with(&self, other: &dyn TolerantGeometry, context: &ToleranceContext) -> bool {
        self.distance_to(other, context) < context.linear
    }

    /// 幾何要素の有効性検証
    fn is_valid(&self, context: &ToleranceContext) -> bool;

    /// 幾何要素の退化判定
    fn is_degenerate(&self, context: &ToleranceContext) -> bool;
}

/// 位相的一貫性チェッカー（将来の B-rep モデリング用）
pub struct TopologyChecker {
    #[allow(dead_code)] // 将来のB-repモデリング実装で使用予定
    context: ToleranceContext,
}

impl TopologyChecker {
    pub fn new(context: ToleranceContext) -> Self {
        Self { context }
    }

    /// オイラー特性数の検証 (V - E + F = 2 for closed solids)
    pub fn verify_euler_characteristic(&self, vertices: usize, edges: usize, faces: usize) -> bool {
        // 简单的封闭立体检查
        (vertices as i32) - (edges as i32) + (faces as i32) == 2
    }

    /// 境界の向き一貫性チェック
    pub fn verify_boundary_orientation(&self, _boundaries: &[Box<dyn TopologicalEntity>]) -> bool {
        // TODO: 将来の実装で境界の向き一貫性をチェック
        true
    }

    /// 多様体性の検証
    pub fn verify_manifold_property(&self, _entity: &dyn TopologicalEntity) -> bool {
        // TODO: 将来の実装で多様体性をチェック
        true
    }
}
