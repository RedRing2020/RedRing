//! トリムした面の (u, v) の範囲を、境界の点をそのまま使って三角形に分ける（設計:
//! TRIMMED_FACE_TESSELLATION_DESIGN.md 5.3・6 章）
//!
//! 1. [`FaceRegion::new`] で、面のループ（(u, v) の点と頂点の番号の並び）から、切れ目を入れた範囲（閉じた多角形の
//!    並び）を作る
//! 2. [`FaceRegion::crossing_edges`] で、多角形の辺どうしが交わらないかを、三角形分割と同じ座標で確かめる（交われば、
//!    呼び出し元が稜線を分け直す）
//! 3. [`FaceRegion::triangulate`] で、多角形の辺を必ず辺に持つ制約つきのドロネー分割をし、許容弦誤差と刻みで内側に
//!    点を足す
//!
//! 境界の点は、そのまま三角形の頂点にする（境界に点を足さない）。足すのは、切れ目と極の点と、内側の点だけ。

mod region;
mod triangulate;

#[cfg(test)]
mod tests;

pub use region::FaceRegion;

use analysis::consts::tessellation::GRID_DIAGONAL_TO_STEP_RATIO;

use crate::{Angle, Point2D, Point3D, Vector3D};

/// 三角形に分ける面が載る曲面
pub trait TriangulationSurface {
    /// パラメータ (u, v) の点（評価できなければ `None`）
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>>;

    /// パラメータ (u, v) の法線（∂u × ∂v の向き。極などで定まらなければ `None`）
    fn normal_at(&self, u: f64, v: f64) -> Option<Vector3D<f64>>;

    /// u・v の周期（周期のない方向は `None`）
    fn periods(&self) -> (Option<f64>, Option<f64>);

    /// u によらず 1 点になる v（球の極・円錐の頂点）。小さい側と大きい側（ない側は `None`）
    fn poles(&self) -> (Option<f64>, Option<f64>);

    /// 許容弦誤差 `chord_tolerance` と角度の上限 `max_angle` から決まる u・v の刻み（刻みで点を足さない方向は
    /// `None`）
    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (Option<f64>, Option<f64>);

    /// 三角形分割の座標（曲面の上の長さに近い座標）にするために、u・v に掛ける値（正）
    fn scale(&self) -> (f64, f64);
}

/// 面を三角形に分ける設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceTriangulationSettings {
    /// 許容弦誤差（三角形と曲面の離れの上限。正）
    pub chord_tolerance: f64,
    /// 刻みを決める角度の上限（0 より大きく 180 度より小さい）
    pub max_angle: Angle<f64>,
    /// 1 つの面の頂点の数の上限
    pub max_points: usize,
    /// 内側に点を足す繰り返しの回数の上限
    pub max_passes: usize,
    /// 細かくする三角形の幅の下限（3 次元。0 以上）。幅（最も長い辺に、向かいの頂点から下ろした高さ）がこれより小さい
    /// 三角形は、曲面からの離れをこの値まで許し（許容弦誤差より大きければ）、法線のなす角では点を足さない。0 なら下限を
    /// 置かない
    pub min_width: f64,
}

impl FaceTriangulationSettings {
    fn check(&self) -> Result<(), FaceTriangulationError> {
        let angle = self.max_angle.to_radians();
        if self.chord_tolerance.is_finite()
            && self.chord_tolerance > 0.0
            && angle > 0.0
            && angle < std::f64::consts::PI
            && self.max_points >= 3
            && self.min_width.is_finite()
            && self.min_width >= 0.0
        {
            Ok(())
        } else {
            Err(FaceTriangulationError::InvalidSettings)
        }
    }
}

/// 格子と切れ目・極の線に置く点の、u・v の間隔（刻みのある方向は、正の有限の値でなければ、曲面の値が正しくない
/// とする）
///
/// 間隔は、曲面の刻みに `GRID_DIAGONAL_TO_STEP_RATIO / √2` を掛けた値とする。刻みのままでは、マスの対角線と、
/// 切れ目の辺に接する三角形の内側の離れが許容弦誤差を超え、細分化で点を 1 つずつ足し続けるため
fn checked_steps(
    surface: &impl TriangulationSurface,
    settings: &FaceTriangulationSettings,
) -> Result<(Option<f64>, Option<f64>), FaceTriangulationError> {
    let steps = surface.steps(settings.chord_tolerance, settings.max_angle);
    let valid = |step: Option<f64>| step.is_none_or(|step| step.is_finite() && step > 0.0);
    if valid(steps.0) && valid(steps.1) {
        let ratio = GRID_DIAGONAL_TO_STEP_RATIO * std::f64::consts::FRAC_1_SQRT_2;
        Ok((
            steps.0.map(|step| step * ratio),
            steps.1.map(|step| step * ratio),
        ))
    } else {
        Err(FaceTriangulationError::Evaluation)
    }
}

/// ループの点（境界の頂点の番号と (u, v)）
///
/// 極（`TriangulationSurface::poles`）の上の点は、v を極の値に合わせて渡す（差が
/// `analysis::consts::tessellation::POLE_PARAMETER_TOLERANCE` より大きいと、極の点として扱わない）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoopPoint {
    pub vertex: usize,
    pub uv: Point2D<f64>,
}

/// 三角形の頂点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FaceVertex {
    /// 境界の点（ループの点の頂点の番号）
    Boundary(usize),
    /// 足した点（[`FaceTriangulation::added`] の位置）
    Added(usize),
}

/// 足した点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AddedPoint {
    pub uv: Point2D<f64>,
    pub point: Point3D<f64>,
}

/// 面を三角形に分けた結果
#[derive(Debug, Clone, PartialEq)]
pub struct FaceTriangulation {
    /// 足した点（切れ目・極・内側の点）
    pub added: Vec<AddedPoint>,
    /// 三角形（(u, v) で反時計回り）。極でつぶれる三角形は含めない
    pub triangles: Vec<[FaceVertex; 3]>,
}

/// 面を三角形に分けられない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceTriangulationError {
    /// 設定が正しくない（許容弦誤差が正の有限の値でない、角度の上限が範囲の外、頂点の数の上限が 3 より小さい）
    InvalidSettings,
    /// ループが正しくない（有限の値でない、周期をまわらないループの点が 3 つより少ない）
    InvalidLoops,
    /// 扱えないループの組（周期を 2 回以上まわる、u と v の両方にまわる、外側のループが決まらないなど）
    UnsupportedLoops,
    /// 穴を避けて切れ目を置けない
    NoCut,
    /// 多角形の辺どうしが交わる
    Crossing,
    /// 曲面を評価できない（点が有限の値でない、刻みや倍率が正の有限の値でない）
    Evaluation,
    /// 頂点の数が上限を超える
    TooManyPoints,
    /// 繰り返しの回数の上限までに、許容弦誤差と刻みを守れない
    NotConverged,
}

impl std::fmt::Display for FaceTriangulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidSettings => "invalid face triangulation settings",
            Self::InvalidLoops => "invalid face loops",
            Self::UnsupportedLoops => "unsupported face loops",
            Self::NoCut => "no cut avoids the holes",
            Self::Crossing => "the face loops cross",
            Self::Evaluation => "the surface cannot be evaluated",
            Self::TooManyPoints => "too many points",
            Self::NotConverged => "the refinement does not converge",
        })
    }
}

impl std::error::Error for FaceTriangulationError {}
