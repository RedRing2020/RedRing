//! 加工の条件の値（加工の種類と条件・加工ステージ・加工範囲）
//!
//! 値と加工ステージの token の読み取りだけを持ち、条件の検証と経路の計算は持たない。

/// スキャン加工（`operation_type = scanline`）パラメータ
///
/// 走査ラインを、ラインに垂直な向きへ `stepover` ずつ送る。ラインの向きは X 軸から `angle_degrees` 回した向きで、
/// 0° では X 方向のラインを Y 方向へ送る。各ラインの送る向きは `cut_direction` で決める。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScanlineParams {
    /// 走査ライン間隔（mm）。工具径以下であること
    pub stepover: f64,
    /// 切削送り速度（mm/min）
    pub feed_rate: f64,
    /// 加工範囲（境界）を指定しない場合の、工具の中心の移動範囲
    pub tool_range: ScanlineToolRange,
    /// 各走査ラインの送り方
    pub cut_direction: ScanlineCutDirection,
    /// 走査ラインの角度（度。X 軸から、上から見て反時計回り）
    pub angle_degrees: f64,
}

/// スキャン加工の送り方
///
/// 主軸は右回り（上から見て時計回り）とする。走査ラインは Y 正方向へ進むため、まだ削っていない素材は +Y 側にあり、
/// X 負方向に送ると素材が送りの右側になってダウンカット、X 正方向に送るとアップカットになる（角度を付けたラインでは、
/// X・Y をラインの向きとそれに垂直な向きに読み替える）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanlineCutDirection {
    /// すべてのラインを X 負方向に送る一方向の加工
    #[default]
    DownCut,
    /// すべてのラインを X 正方向に送る一方向の加工
    UpCut,
    /// 最初のラインをダウンカット（X 負方向）で送り、ラインごとに送る向きを入れ替える往復の加工
    ZigZag,
}

/// 加工範囲（境界）を指定しないスキャン加工の、工具の中心の移動範囲
///
/// 形状の範囲の外へ出た工具は、形状の端の稜線の上を転がって高さが下がり、端の外側の素材を丸く削る（角がだれる）。
/// 角をだれさせない場合は、見切り面を加工の対象に含める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanlineToolRange {
    /// 形状の XY の範囲を、各辺で工具の半径だけ外へ広げた範囲（平坦面の端の外側の素材を残さない）
    #[default]
    ExtendByToolRadius,
    /// 形状の XY の範囲
    GeometryExtent,
}

/// オペレーション定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationSpec {
    Scanline(ScanlineParams),
}

/// 加工ステージ（`machining_stage`）
///
/// 実行順序を強制する状態ではなくタグ分類として扱い、`operation_type` とは直交する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachiningStage {
    /// `rough`: 荒加工
    Rough,
    /// `semi_finish`: 中加工
    SemiFinish,
    /// `finish`: 仕上げ
    Finish,
}

impl MachiningStage {
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "rough" => Some(Self::Rough),
            "semi_finish" => Some(Self::SemiFinish),
            "finish" => Some(Self::Finish),
            _ => None,
        }
    }
}

/// 矩形の加工範囲（`boundary_mode = rectangle`、ワーク局所座標・mm）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectangleBoundary {
    pub rect_min: [f64; 2],
    pub rect_max: [f64; 2],
}

/// 加工範囲（`boundary_mode`）
///
/// `edge_projected_2d` は後続 Step で追加する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MachiningBoundary {
    Rectangle(RectangleBoundary),
}
