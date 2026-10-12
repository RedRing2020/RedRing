//! AppState のデバッグ表示ロード（octree/toolpath/svg/nurbs）を扱うモジュール群。

mod camera_fit;
mod inverse_offset;
mod nurbs;
mod octree;
mod shapes;
mod toolpath;

pub(crate) use inverse_offset::InverseOffsetDebugState;
