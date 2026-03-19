//! Ray 形状の契約定義
//!
//! geo_primitives が実装すべき Ray 系の公開 trait。
//! analysis との結合を避けるため origin / direction はタプルで返す。

use crate::Scalar;

/// Ray3D 基本プロパティ取得 trait
pub trait Ray3DProperties<T: Scalar> {
    /// 起点を取得
    fn origin(&self) -> (T, T, T);

    /// 方向ベクトルを取得（正規化済み）
    fn direction(&self) -> (T, T, T);

    /// 起点のX座標
    fn origin_x(&self) -> T;

    /// 起点のY座標
    fn origin_y(&self) -> T;

    /// 起点のZ座標
    fn origin_z(&self) -> T;

    /// 方向ベクトルのX成分
    fn direction_x(&self) -> T;

    /// 方向ベクトルのY成分
    fn direction_y(&self) -> T;

    /// 方向ベクトルのZ成分
    fn direction_z(&self) -> T;

    /// Rayが有効かどうか（方向ベクトルが非ゼロ）
    fn is_valid(&self) -> bool;

    /// 方位角（azimuth）を取得（XY平面での角度）
    fn azimuth(&self) -> T;

    /// 仰角（elevation）を取得（Z軸からの角度）
    fn elevation(&self) -> T;

    /// RayがXY平面上にあるかどうか
    fn is_on_xy_plane(&self) -> bool;
}
