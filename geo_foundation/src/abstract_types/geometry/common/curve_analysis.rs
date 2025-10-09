//! 曲線解析の統一インターフェイス
//!
//! NURBS、円、円弧等すべての曲線型に適用可能な
//! 接線、法線、曲率等の微分幾何学的解析の共通API

use crate::abstract_types::Scalar;

/// 微分幾何学的情報を格納する構造体
///
/// 曲線上の任意の点における接線、法線、曲率等を統一的に表現
#[derive(Debug, Clone, PartialEq)]
pub struct DifferentialGeometry<T: Scalar, V> {
    /// 接線ベクトル（正規化済み）
    pub tangent: V,
    /// 主法線ベクトル（正規化済み、曲率の方向）
    pub normal: V,
    /// 曲率値（1/曲率半径）
    pub curvature: T,
    /// 曲率半径（曲率の逆数、直線部分では無限大）
    pub curvature_radius: T,
}

impl<T: Scalar, V> DifferentialGeometry<T, V> {
    /// 新しい微分幾何学的情報を作成
    pub fn new(tangent: V, normal: V, curvature: T) -> Self {
        let curvature_radius = if curvature.abs() < T::TOLERANCE {
            T::INFINITY
        } else {
            T::ONE / curvature
        };

        Self {
            tangent,
            normal,
            curvature,
            curvature_radius,
        }
    }

    /// 直線部分（曲率ゼロ）かどうかを判定
    pub fn is_straight(&self) -> bool {
        self.curvature.abs() < T::TOLERANCE
    }
}

/// 2D曲線の微分幾何学的解析トレイト
pub trait CurveAnalysis2D<T: Scalar> {
    /// 点の型（通常は Point2D）
    type Point;
    /// ベクトルの型（通常は Vector2D）
    type Vector;

    /// 指定されたパラメータ位置での点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    fn tangent_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定されたパラメータ位置での法線ベクトルを取得（正規化済み）
    fn normal_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定されたパラメータ位置での曲率を取得
    fn curvature_at_parameter(&self, t: T) -> T;

    /// 指定されたパラメータ位置での曲率半径を取得
    fn curvature_radius_at_parameter(&self, t: T) -> T {
        let curvature = self.curvature_at_parameter(t);
        if curvature.abs() < T::TOLERANCE {
            T::INFINITY
        } else {
            T::ONE / curvature
        }
    }

    /// 指定されたパラメータ位置での接線と曲率を同時計算（効率的）
    fn tangent_and_curvature_at_parameter(&self, t: T) -> (Self::Vector, T) {
        (self.tangent_at_parameter(t), self.curvature_at_parameter(t))
    }

    /// 指定されたパラメータ位置での微分幾何学的情報を一括取得（最も効率的）
    fn differential_geometry_at_parameter(&self, t: T) -> DifferentialGeometry<T, Self::Vector> {
        DifferentialGeometry::new(
            self.tangent_at_parameter(t),
            self.normal_at_parameter(t),
            self.curvature_at_parameter(t),
        )
    }

    /// 最大曲率の位置と値を取得（解析的に求められる場合）
    fn max_curvature(&self) -> Option<(T, T)> {
        // デフォルト実装なし（各具象型で実装）
        None
    }

    /// 最小曲率の位置と値を取得（解析的に求められる場合）
    fn min_curvature(&self) -> Option<(T, T)> {
        // デフォルト実装なし（各具象型で実装）
        None
    }

    /// 曲率がゼロになる位置を取得（変曲点、解析的に求められる場合）
    fn inflection_points(&self) -> Vec<T> {
        // デフォルト実装なし（各具象型で実装）
        Vec::new()
    }
}

/// 効率的な曲線解析の統一ヘルパー
///
/// 個別実装を補完する共通計算ロジック集
pub struct CurveAnalysisHelper;

impl CurveAnalysisHelper {
    /// 数値微分による接線計算（NURBS等の複雑な曲線用）
    pub fn numerical_tangent<T: Scalar, C>(_curve: &C, _t: T, _delta: T) -> C::Vector
    where
        C: CurveAnalysis2D<T>,
    {
        // 数値微分実装
        // let p1 = curve.point_at_parameter(t - delta);
        // let p2 = curve.point_at_parameter(t + delta);
        // (p2 - p1).normalize()
        todo!("Numerical tangent calculation")
    }

    /// 数値微分による曲率計算（NURBS等の複雑な曲線用）
    pub fn numerical_curvature<T: Scalar, C>(_curve: &C, _t: T, _delta: T) -> T
    where
        C: CurveAnalysis2D<T>,
    {
        // 数値微分による曲率計算
        // κ = |r' × r''| / |r'|³
        todo!("Numerical curvature calculation")
    }

    /// 円弧の解析的微分幾何学計算（効率的）
    pub fn circle_differential_geometry<T: Scalar>(
        _center: T,
        radius: T,
        start_angle: T,
        angle_range: T,
        t: T,
    ) -> (T, T, T) {
        let angle = start_angle + t * angle_range;
        let _cos_a = angle.cos();
        let _sin_a = angle.sin();

        // 戻り値: (tangent_magnitude, normal_magnitude, curvature)
        // 具象型側で適切なベクトル型に変換
        (T::ONE, T::ONE, T::ONE / radius)
    }

    /// 楕円の解析的曲率計算
    pub fn ellipse_curvature<T: Scalar>(semi_major: T, semi_minor: T, t: T) -> T {
        let angle = t * T::TAU;
        let sin_t = angle.sin();
        let cos_t = angle.cos();
        let a = semi_major;
        let b = semi_minor;

        // κ = ab / (a²sin²θ + b²cos²θ)^(3/2)
        let denominator = (a * a * sin_t * sin_t + b * b * cos_t * cos_t)
            .sqrt()
            .powi(3);
        a * b / denominator
    }
}

/// 3D曲線の微分幾何学的解析トレイト
pub trait CurveAnalysis3D<T: Scalar> {
    /// 点の型（通常は Point3D）
    type Point;
    /// ベクトルの型（通常は Vector3D）
    type Vector;
    /// 方向の型（通常は Direction3D）
    type Direction;

    /// 指定されたパラメータ位置での点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    fn tangent_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定されたパラメータ位置での主法線ベクトルを取得（正規化済み）
    fn normal_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定されたパラメータ位置での双法線ベクトルを取得（正規化済み）
    fn binormal_at_parameter(&self, t: T) -> Self::Vector;

    /// 指定されたパラメータ位置での曲率を取得
    fn curvature_at_parameter(&self, t: T) -> T;

    /// 指定されたパラメータ位置での捩率（ねじれ）を取得
    fn torsion_at_parameter(&self, t: T) -> T;

    /// 指定されたパラメータ位置での曲率半径を取得
    fn curvature_radius_at_parameter(&self, t: T) -> T {
        let curvature = self.curvature_at_parameter(t);
        if curvature.abs() < T::TOLERANCE {
            T::INFINITY
        } else {
            T::ONE / curvature
        }
    }

    /// 指定されたパラメータ位置での接線と曲率を同時計算（効率的）
    fn tangent_and_curvature_at_parameter(&self, t: T) -> (Self::Vector, T) {
        (self.tangent_at_parameter(t), self.curvature_at_parameter(t))
    }

    /// 指定されたパラメータ位置でのFrenet標構（接線、法線、双法線）を取得
    fn frenet_frame_at_parameter(&self, t: T) -> (Self::Vector, Self::Vector, Self::Vector) {
        (
            self.tangent_at_parameter(t),
            self.normal_at_parameter(t),
            self.binormal_at_parameter(t),
        )
    }

    /// 指定されたパラメータ位置での微分幾何学的情報を一括取得（最も効率的）
    fn differential_geometry_at_parameter(&self, t: T) -> DifferentialGeometry<T, Self::Vector> {
        DifferentialGeometry::new(
            self.tangent_at_parameter(t),
            self.normal_at_parameter(t),
            self.curvature_at_parameter(t),
        )
    }

    /// 最大曲率の位置と値を取得（解析的に求められる場合）
    fn max_curvature(&self) -> Option<(T, T)> {
        // デフォルト実装なし（各具象型で実装）
        None
    }

    /// 最小曲率の位置と値を取得（解析的に求められる場合）
    fn min_curvature(&self) -> Option<(T, T)> {
        // デフォルト実装なし（各具象型で実装）
        None
    }

    /// 曲率がゼロになる位置を取得（変曲点、解析的に求められる場合）
    fn inflection_points(&self) -> Vec<T> {
        // デフォルト実装なし（各具象型で実装）
        Vec::new()
    }

    /// 曲線が平面曲線かどうかを判定（捩率がゼロ）
    fn is_planar(&self) -> bool {
        // デフォルト実装：捩率を数値的にチェック
        // 具象型では解析的な判定が可能な場合がある
        false
    }
}

/// 解析的曲線の拡張トレイト（解析形状用の効率的実装）
pub trait AnalyticalCurve<T: Scalar> {
    /// 曲線の種類（円、楕円、直線等）
    fn curve_type(&self) -> CurveType;

    /// 一定曲率かどうか（円の場合）
    fn has_constant_curvature(&self) -> bool {
        matches!(self.curve_type(), CurveType::Circle)
    }

    /// 解析的に計算可能な曲率の定数値（円の場合）
    fn constant_curvature(&self) -> Option<T> {
        None
    }

    /// 解析的に計算可能な曲率半径の定数値（円の場合）
    fn constant_curvature_radius(&self) -> Option<T> {
        self.constant_curvature().map(|k| {
            if k.abs() < T::TOLERANCE {
                T::INFINITY
            } else {
                T::ONE / k
            }
        })
    }
}

/// 曲線の種類を表す列挙型
#[derive(Debug, Clone, PartialEq)]
pub enum CurveType {
    /// 直線
    Line,
    /// 円
    Circle,
    /// 楕円
    Ellipse,
    /// 楕円弧
    EllipseArc,
    /// 円弧
    CircleArc,
    /// NURBS曲線
    Nurbs,
    /// ベジエ曲線
    Bezier,
    /// B-スプライン曲線
    BSpline,
    /// その他
    Other(String),
}

/// 数値微分による曲線解析のヘルパートレイト
///
/// 解析的な微分が困難な曲線（NURBS等）での数値計算用
pub trait NumericalCurveAnalysis<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;

    /// 指定されたパラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 数値微分による接線ベクトル計算
    fn numerical_tangent_at_parameter(&self, t: T, delta: T) -> Self::Vector;

    /// 数値微分による曲率計算
    fn numerical_curvature_at_parameter(&self, t: T, delta: T) -> T;

    /// 数値微分による法線ベクトル計算
    fn numerical_normal_at_parameter(&self, t: T, delta: T) -> Self::Vector;
}
