/// RedRing トレラント幾何モデリング基盤
///
/// 数値誤差に堅牢な幾何計算を提供し、CAD/CAMアプリケーションの
/// 基礎となる許容誤差ベースの演算を実装する。

pub mod scalar;
pub mod vector;
pub mod tolerance;
pub mod robust;
pub mod primitives;
pub mod primitives2d;
pub mod primitives3d;
pub mod angle; // 角度型抽象

// テストモジュール
#[cfg(test)]
mod unit_tests;

// 主要な型の再エクスポート
pub use tolerance::{ToleranceContext, TolerantEq, TolerantOrd};
pub use scalar::Scalar;
pub use angle::Angle;
pub use vector::{Vector, Vector2D, Vector3D, Direction3D};
pub use primitives::{
    Point2D, Point3D, LineSegment2D, LineSegment3D,
    Arc2D, Polygon2D, Plane, Sphere,
    ParametricCurve2D, ParametricCurve3D, ParametricSurface
};
pub use robust::{Orientation, RobustSolver};

/// 標準的な許容誤差コンテキスト
pub const DEFAULT_TOLERANCE: ToleranceContext = ToleranceContext {
    linear: 1e-6,      // 1マイクロメートル
    angular: 1e-8,     // 約0.0057度
    parametric: 1e-10, // パラメータ空間
    curvature: 1e-3,   // 曲率許容誤差
    area: 1e-12,       // 面積許容誤差
    volume: 1e-18,     // 体積許容誤差
};

/// プリファクトリ：よく使用される値の作成
pub mod prelude {
    pub use crate::{
        Point2D, Point3D, Vector2D, Vector3D, Direction3D,
        ToleranceContext, TolerantEq, TolerantOrd,
        DEFAULT_TOLERANCE,
    };
}


