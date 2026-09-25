//! CAM アルゴリズム層（ToolPath 生成）
//!
//! 形状入力（NURBS 曲面集合 / 三角形メッシュ）から ToolPath を生成する純粋計算クレート。
//! Job 実行制御や参照解決は持たず、`cam_sim` の Job アダプタから呼び出される。
//!
//! 3D 経路生成は逆オフセット法を基盤とする。形状を三角形へ離散化し、
//! 工具種別ごとの要素オフセット（ボールエンドミル: 頂点→球、辺→円筒、面→工具半径オフセット面、
//! フラットエンドミル: 頂点→円板、辺→円板掃引、面→底面円周接触）の上側包絡から工具位置（CL）を求める。

pub mod cl_grid;
pub mod inverse_offset;
pub mod process_template;
pub mod scanline;
pub mod solver;
pub mod tessellation;

pub use cl_grid::{ClGrid, MAX_CL_GRID_SAMPLES, sample_cl_grid};
pub use inverse_offset::{CutterShape, DropCutter};
pub use process_template::{
    MachiningBoundary, MachiningStage, OperationDefinition, OperationType, ProcessTemplate,
    RectangleBoundary, ResolvedOperation, SolverDefaults, TemplateFailure, build_solver_input,
    resolve_operation,
};
pub use scanline::generate_scanline_toolpath;
pub use solver::{
    CamSolverError, CamSolverInput, GeometryKind, OperationSpec, ScanlineParams, SolverGeometry,
    solve_toolpath,
};
pub use tessellation::{TessellationLimits, tessellate_surfaces};

#[cfg(test)]
mod tests;
