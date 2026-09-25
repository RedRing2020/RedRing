//! CAM アルゴリズム層（ToolPath 生成）
//!
//! 形状入力（NURBS 曲面集合 / 三角形メッシュ）から ToolPath を生成する純粋計算クレート。
//! Job 実行制御や参照解決は持たず、`cam_sim` の Job アダプタから呼び出される。
//!
//! 3D 経路生成は逆オフセット法を基盤とする。形状を三角形へ離散化し、
//! 工具種別ごとの要素オフセット（ボールエンドミル: 頂点→球、辺→円筒、面→工具半径オフセット面）
//! の上側包絡から工具位置（CL）を求める。

pub mod inverse_offset;
pub mod scanline;
pub mod solver;
pub mod tessellation;

pub use inverse_offset::BallDropCutter;
pub use scanline::generate_scanline_toolpath;
pub use solver::{
    CamSolverError, CamSolverInput, GeometryKind, OperationSpec, ScanlineParams, SolverGeometry,
    solve_toolpath,
};
pub use tessellation::{TessellationLimits, tessellate_surfaces};

#[cfg(test)]
mod tests;
