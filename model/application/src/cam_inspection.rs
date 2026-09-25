//! CAM solver 検査（逆オフセット包絡面）向け orchestration 境界。
//!
//! 形状入力を離散化し、逆オフセット法の包絡面を格子で評価して返す。
//! ToolPath 生成（Job 経由の本番導線）とは独立した、検査・可視化用の入口。

use cam_algorithms::{
    CamSolverError, CutterShape, DropCutter, TessellationLimits, sample_cl_grid,
    tessellate_surfaces,
};
use geo_algorithms::NurbsSurface3D;

/// 検査対象の工具種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionToolKind {
    BallEndMill,
    FlatEndMill,
}

impl InspectionToolKind {
    /// 表示・ログ用ラベル
    pub fn label(self) -> &'static str {
        match self {
            Self::BallEndMill => "ball end mill",
            Self::FlatEndMill => "flat end mill",
        }
    }

    /// もう一方の工具種別
    pub fn toggled(self) -> Self {
        match self {
            Self::BallEndMill => Self::FlatEndMill,
            Self::FlatEndMill => Self::BallEndMill,
        }
    }
}

/// 逆オフセット包絡面検査の入力
#[derive(Debug, Clone)]
pub struct InverseOffsetInspectionRequest {
    pub surfaces: Vec<NurbsSurface3D<f64>>,
    pub tool_kind: InspectionToolKind,
    pub tool_radius: f64,
    /// 形状離散化の許容弦誤差
    pub chord_tolerance: f64,
    /// 包絡面を評価する格子間隔
    pub grid_pitch: f64,
}

/// 包絡面の格子（行 = Y、列 = X の行優先）
///
/// 高さは逆オフセットの基準点（ボール: 球中心、フラット: 底面中心）で表す。
#[derive(Debug, Clone, PartialEq)]
pub struct InverseOffsetEnvelopeGrid {
    pub origin: [f64; 2],
    pub pitch: f64,
    pub columns: usize,
    pub rows: usize,
    pub heights: Vec<Option<f64>>,
}

impl InverseOffsetEnvelopeGrid {
    pub fn xy(&self, column: usize, row: usize) -> [f64; 2] {
        [
            self.origin[0] + self.pitch * column as f64,
            self.origin[1] + self.pitch * row as f64,
        ]
    }

    pub fn height(&self, column: usize, row: usize) -> Option<f64> {
        self.heights[row * self.columns + column]
    }
}

/// 逆オフセット包絡面検査の結果
#[derive(Debug, Clone, PartialEq)]
pub struct InverseOffsetInspection {
    /// 離散化した元形状の頂点
    pub source_vertices: Vec<[f64; 3]>,
    /// 離散化した元形状の三角形
    pub source_triangles: Vec<[usize; 3]>,
    /// 逆オフセット包絡面
    pub envelope: InverseOffsetEnvelopeGrid,
    /// 工具先端から包絡面基準点までの高さ
    pub reference_offset: f64,
}

/// 検査の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InverseOffsetInspectionError {
    /// solver の失敗分類（`invalid_input` / `no_solution` / `convergence_failure`）
    Solver { code: &'static str, message: String },
}

impl std::fmt::Display for InverseOffsetInspectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Solver { code, message } => write!(f, "{}: {}", code, message),
        }
    }
}

impl std::error::Error for InverseOffsetInspectionError {}

impl From<CamSolverError> for InverseOffsetInspectionError {
    fn from(value: CamSolverError) -> Self {
        Self::Solver {
            code: value.code(),
            message: value.reason().to_string(),
        }
    }
}

/// 形状を離散化し、逆オフセット包絡面を格子で評価する。
///
/// 評価範囲は形状の XY 範囲を工具半径だけ広げた領域とする。
pub fn inspect_inverse_offset(
    request: &InverseOffsetInspectionRequest,
) -> Result<InverseOffsetInspection, InverseOffsetInspectionError> {
    if !(request.tool_radius.is_finite() && request.tool_radius > 0.0) {
        return Err(CamSolverError::InvalidInput(
            "tool radius must be positive and finite".to_string(),
        )
        .into());
    }
    if !(request.chord_tolerance.is_finite() && request.chord_tolerance > 0.0) {
        return Err(CamSolverError::InvalidInput(
            "chord_tolerance must be positive and finite".to_string(),
        )
        .into());
    }

    let mesh = tessellate_surfaces(
        &request.surfaces,
        request.chord_tolerance,
        TessellationLimits::default(),
    )?;
    let shape = match request.tool_kind {
        InspectionToolKind::BallEndMill => CutterShape::Ball {
            radius: request.tool_radius,
        },
        InspectionToolKind::FlatEndMill => CutterShape::Flat {
            radius: request.tool_radius,
        },
    };
    let cutter = DropCutter::new(&mesh, shape)?;
    let grid = sample_cl_grid(&cutter, request.grid_pitch, request.tool_radius)?;

    let reference_offset = shape.reference_offset();
    Ok(InverseOffsetInspection {
        source_vertices: mesh
            .vertices()
            .iter()
            .map(|p| [p.x(), p.y(), p.z()])
            .collect(),
        source_triangles: mesh.indices().to_vec(),
        envelope: InverseOffsetEnvelopeGrid {
            origin: grid.origin,
            pitch: grid.pitch,
            columns: grid.columns,
            rows: grid.rows,
            heights: grid
                .tip_heights
                .iter()
                .map(|height| height.map(|z| z + reference_offset))
                .collect(),
        },
        reference_offset,
    })
}

#[cfg(test)]
mod tests {
    use geo_algorithms::nurbs_fixtures::create_sample_nurbs_surface_with_adaptive_params;

    use super::*;

    fn request(tool_kind: InspectionToolKind) -> InverseOffsetInspectionRequest {
        let (surface, _) = create_sample_nurbs_surface_with_adaptive_params(0.01).unwrap();
        InverseOffsetInspectionRequest {
            surfaces: vec![surface],
            tool_kind,
            tool_radius: 0.1,
            chord_tolerance: 0.001,
            grid_pitch: 0.05,
        }
    }

    #[test]
    fn ball_envelope_is_tool_center_one_radius_above_tip() {
        let inspection = inspect_inverse_offset(&request(InspectionToolKind::BallEndMill)).unwrap();
        assert!(!inspection.source_triangles.is_empty());
        assert_eq!(inspection.reference_offset, 0.1);

        let envelope = &inspection.envelope;
        // サンプル曲面 (0..1) を工具半径 0.1 だけ拡張 → -0.1..1.1 を 0.05 間隔で 25 点
        assert_eq!((envelope.columns, envelope.rows), (25, 25));
        // 曲面頂点 (0.5, 0.5) は z=0.125、曲率半径 1 > r のため球は頂点で接し中心は r 上
        let chord_tolerance = request(InspectionToolKind::BallEndMill).chord_tolerance;
        let center = envelope.height(12, 12).unwrap();
        let [x, y] = envelope.xy(12, 12);
        let distance_tolerance = geo_contracts::default_distance_tolerance::<f64>();
        assert!((x - 0.5).abs() <= distance_tolerance && (y - 0.5).abs() <= distance_tolerance);
        assert!(
            (center - (0.125 + 0.1)).abs() <= 2.0 * chord_tolerance,
            "center height = {center}"
        );
        // 形状の XY 範囲内では直下の曲面点（z >= 0）から球中心が r 以上離れる
        for row in 0..envelope.rows {
            for column in 0..envelope.columns {
                let [x, y] = envelope.xy(column, row);
                if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
                    let z = envelope.height(column, row).unwrap();
                    assert!(z >= 0.1 - chord_tolerance, "center at ({x}, {y}) = {z}");
                }
            }
        }
    }

    #[test]
    fn flat_envelope_is_tool_bottom() {
        let inspection = inspect_inverse_offset(&request(InspectionToolKind::FlatEndMill)).unwrap();
        assert_eq!(inspection.reference_offset, 0.0);
        // 外周角は形状から r 以上離れて非接触
        assert!(inspection.envelope.height(0, 0).is_none());
    }

    #[test]
    fn invalid_pitch_is_reported_as_invalid_input() {
        let mut invalid = request(InspectionToolKind::BallEndMill);
        invalid.grid_pitch = 0.0;
        let error = inspect_inverse_offset(&invalid).unwrap_err();
        assert!(matches!(
            error,
            InverseOffsetInspectionError::Solver {
                code: "invalid_input",
                ..
            }
        ));
    }
}
