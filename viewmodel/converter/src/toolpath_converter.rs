//! CAM工具経路のViewModel変換
//!
//! `cam_core::ToolPath` を GPU描画用の頂点データに変換します。
//!
//! # 設計方針
//!
//! - **色設定の注入**: App層から `ToolPathColorScheme` を受け取る
//! - **3フェーズ構造対応**: approach → cutting → retract の分離
//! - **セグメント種別の識別**: Cutting, Rapid, Approach, Retract, PassRetract
//! - **円弧テッセレーション**: 円弧を線分に分割（設定可能な分割数）

use cam_core::{
    ArcDirection, ContourLevelPath, CuttingDirection, PathGeometry, PathSegment, SegmentType,
    ToolPath,
};
use geo_algorithms::Point3D;

/// 工具経路の色設定（App層から注入）
#[derive(Debug, Clone, Copy)]
pub struct ToolPathColorScheme {
    /// 切削セグメントの色（デフォルト: 白色）
    pub cutting: [f32; 4],

    /// 早送りセグメントの色（デフォルト: 青色）
    pub rapid: [f32; 4],

    /// アプローチセグメントの色（デフォルト: 緑色）
    pub approach: [f32; 4],

    /// リトラクトセグメントの色（デフォルト: 黄色）
    pub retract: [f32; 4],

    /// 周回間リトラクトセグメントの色（デフォルト: オレンジ色）
    pub pass_retract: [f32; 4],

    /// ダウンカットの色（オプション使用、デフォルト: 白色）
    pub down_cut: [f32; 4],

    /// アップカットの色（オプション使用、デフォルト: オレンジ色）
    pub up_cut: [f32; 4],
}

impl Default for ToolPathColorScheme {
    fn default() -> Self {
        Self {
            cutting: [0.0, 0.55, 1.0, 1.0],      // 青みの強いシアン
            rapid: [0.75, 0.75, 0.75, 1.0],      // ライトグレー
            approach: [0.0, 0.2, 0.8, 1.0],      // 深いブルー
            retract: [1.0, 0.4, 0.0, 1.0],       // 鮮やかなオレンジ
            pass_retract: [1.0, 0.75, 0.0, 1.0], // アンバー
            down_cut: [1.0, 1.0, 1.0, 1.0],      // 白色（未使用）
            up_cut: [1.0, 0.5, 0.2, 1.0],        // オレンジ色（未使用）
        }
    }
}

/// テッセレーション設定（App層から注入）
#[derive(Debug, Clone, Copy)]
pub struct TessellationSettings {
    /// 円弧の分割数（セグメント数）
    pub arc_segments: u32,

    /// 最小セグメント長（これ以下の細分化は行わない）
    pub min_segment_length: f64,
}

impl Default for TessellationSettings {
    fn default() -> Self {
        Self {
            arc_segments: 32,
            min_segment_length: 0.1, // 0.1mm
        }
    }
}

/// 工具経路可視化設定（App層から注入される全設定）
#[derive(Debug, Clone)]
pub struct ToolPathVisualizationSettings {
    /// 色設定
    pub color_scheme: ToolPathColorScheme,

    /// テッセレーション設定
    pub tessellation: TessellationSettings,

    /// 切削方向による色分けを有効化
    pub color_by_cutting_direction: bool,

    /// 早送りセグメントを表示するか
    pub show_rapid: bool,

    /// アプローチセグメントを表示するか
    pub show_approach: bool,

    /// リトラクトセグメントを表示するか
    pub show_retract: bool,
}

impl Default for ToolPathVisualizationSettings {
    fn default() -> Self {
        Self {
            color_scheme: ToolPathColorScheme::default(),
            tessellation: TessellationSettings::default(),
            color_by_cutting_direction: false,
            show_rapid: true,
            show_approach: true,
            show_retract: true,
        }
    }
}

/// GPU描画用の頂点データ（位置）
///
/// Note: 色情報は別配列で管理（セグメントごとの色変更に対応）
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub position: [f32; 3],
}

/// 変換後の工具経路頂点データ
#[derive(Debug, Clone)]
pub struct ToolPathVertices {
    /// 頂点配列（線分の始点・終点の連続）
    pub vertices: Vec<Vertex3D>,

    /// セグメントごとの色配列（vertices の各ペアに対応）
    pub colors: Vec<[f32; 4]>,

    /// 各フェーズの頂点範囲（デバッグ用）
    pub phase_ranges: PhaseRanges,
}

/// 各フェーズの頂点インデックス範囲
#[derive(Debug, Clone, Copy, Default)]
pub struct PhaseRanges {
    /// アプローチフェーズの頂点範囲
    pub approach: (usize, usize),

    /// 切削フェーズの頂点範囲
    pub cutting: (usize, usize),

    /// リトラクトフェーズの頂点範囲
    pub retract: (usize, usize),
}

/// 工具経路をGPU描画用の頂点データに変換
///
/// # 引数
///
/// - `toolpath`: 変換元の工具経路
/// - `settings`: 可視化設定（App層から注入）
///
/// # 戻り値
///
/// GPU描画用の頂点配列と色配列
pub fn toolpath_to_vertices(
    toolpath: &ToolPath<f64>,
    settings: &ToolPathVisualizationSettings,
) -> ToolPathVertices {
    let mut vertices = Vec::new();
    let mut colors = Vec::new();

    let approach_start = vertices.len();

    // Phase 1: アプローチフェーズ
    if settings.show_approach {
        for segment in &toolpath.approach_segments {
            convert_segment(
                segment,
                &settings.color_scheme,
                &settings.tessellation,
                toolpath.cutting_direction,
                settings.color_by_cutting_direction,
                &mut vertices,
                &mut colors,
            );
        }
    }

    let approach_end = vertices.len();
    let cutting_start = vertices.len();

    tracing::debug!(
        "Phase 1完了: アプローチ頂点 {} 個（{} 線分）",
        approach_end - approach_start,
        (approach_end - approach_start) / 2
    );

    // Phase 2: 切削フェーズ（等高線ごと）
    for contour_level in &toolpath.contour_levels {
        let before_count = vertices.len();
        convert_contour_level(
            contour_level,
            settings,
            toolpath.cutting_direction,
            &mut vertices,
            &mut colors,
        );
        tracing::debug!(
            "等高線レベル {}: 頂点 {} 個追加（合計 {} 線分）",
            contour_level.level_index,
            vertices.len() - before_count,
            contour_level.segments.len()
        );
    }

    let cutting_end = vertices.len();
    let retract_start = vertices.len();

    tracing::debug!(
        "Phase 2完了: 切削頂点 {} 個（{} 線分）",
        cutting_end - cutting_start,
        (cutting_end - cutting_start) / 2
    );

    // Phase 3: リトラクトフェーズ
    if settings.show_retract {
        for segment in &toolpath.retract_segments {
            convert_segment(
                segment,
                &settings.color_scheme,
                &settings.tessellation,
                toolpath.cutting_direction,
                settings.color_by_cutting_direction,
                &mut vertices,
                &mut colors,
            );
        }
    }

    let retract_end = vertices.len();

    tracing::debug!(
        "Phase 3完了: リトラクト頂点 {} 個（{} 線分）",
        retract_end - retract_start,
        (retract_end - retract_start) / 2
    );

    tracing::info!(
        "ツールパス変換完了: 合計 {} 頂点（approach: {}, cutting: {}, retract: {}）",
        vertices.len(),
        (approach_end - approach_start) / 2,
        (cutting_end - cutting_start) / 2,
        (retract_end - retract_start) / 2
    );

    ToolPathVertices {
        vertices,
        colors,
        phase_ranges: PhaseRanges {
            approach: (approach_start, approach_end),
            cutting: (cutting_start, cutting_end),
            retract: (retract_start, retract_end),
        },
    }
}

/// 等高線レベルパスを頂点データに変換
fn convert_contour_level(
    contour: &ContourLevelPath<f64>,
    settings: &ToolPathVisualizationSettings,
    cutting_direction: CuttingDirection,
    vertices: &mut Vec<Vertex3D>,
    colors: &mut Vec<[f32; 4]>,
) {
    for segment in &contour.segments {
        // 早送りセグメントの表示フィルタリング
        if !settings.show_rapid && segment.is_rapid() {
            continue;
        }

        convert_segment(
            segment,
            &settings.color_scheme,
            &settings.tessellation,
            cutting_direction,
            settings.color_by_cutting_direction,
            vertices,
            colors,
        );
    }
}

/// 単一セグメントを頂点データに変換
fn convert_segment(
    segment: &PathSegment<f64>,
    color_scheme: &ToolPathColorScheme,
    tessellation: &TessellationSettings,
    cutting_direction: CuttingDirection,
    color_by_direction: bool,
    vertices: &mut Vec<Vertex3D>,
    colors: &mut Vec<[f32; 4]>,
) {
    // セグメント種別に応じた色を決定
    let color = get_segment_color(
        &segment.segment_type,
        color_scheme,
        cutting_direction,
        color_by_direction,
    );

    match &segment.geometry {
        PathGeometry::Line { end } => {
            // 直線セグメント: 始点 → 終点
            vertices.push(point_to_vertex(&segment.start));
            vertices.push(point_to_vertex(end));
            colors.push(color);
        }
        PathGeometry::Arc {
            end,
            center,
            direction,
        } => {
            // 円弧セグメント: テッセレーション
            let arc_vertices = tessellate_arc(
                &segment.start,
                end,
                center,
                *direction,
                tessellation.arc_segments,
            );

            // 線分列として頂点追加
            for i in 0..arc_vertices.len().saturating_sub(1) {
                vertices.push(arc_vertices[i]);
                vertices.push(arc_vertices[i + 1]);
                colors.push(color);
            }
        }
    }
}

/// セグメント種別と切削方向に応じた色を取得
fn get_segment_color(
    segment_type: &SegmentType<f64>,
    color_scheme: &ToolPathColorScheme,
    cutting_direction: CuttingDirection,
    color_by_direction: bool,
) -> [f32; 4] {
    match segment_type {
        SegmentType::Cutting { .. } => {
            if color_by_direction {
                // 切削方向による色分け
                match cutting_direction {
                    CuttingDirection::Down => color_scheme.down_cut,
                    CuttingDirection::Up => color_scheme.up_cut,
                }
            } else {
                // デフォルトの切削色
                color_scheme.cutting
            }
        }
        SegmentType::Rapid => color_scheme.rapid,
        SegmentType::Approach { .. } => color_scheme.approach,
        SegmentType::Retract { .. } => color_scheme.retract,
        SegmentType::PassRetract { .. } => color_scheme.pass_retract,
    }
}

/// Point3D を Vertex3D に変換
fn point_to_vertex(point: &Point3D<f64>) -> Vertex3D {
    Vertex3D {
        position: [point.x() as f32, point.y() as f32, point.z() as f32],
    }
}

/// 円弧をテッセレーション（線分分割）
///
/// # 引数
///
/// - `start`: 円弧始点
/// - `end`: 円弧終点
/// - `center`: 円弧中心点
/// - `direction`: 円弧方向（時計回り/反時計回り）
/// - `segments`: 分割数
///
/// # 戻り値
///
/// テッセレーション後の頂点配列
fn tessellate_arc(
    start: &Point3D<f64>,
    end: &Point3D<f64>,
    center: &Point3D<f64>,
    direction: ArcDirection,
    segments: u32,
) -> Vec<Vertex3D> {
    let mut result = Vec::with_capacity((segments + 1) as usize);

    // 半径計算
    let radius = {
        let dx = start.x() - center.x();
        let dy = start.y() - center.y();
        let dz = start.z() - center.z();
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    // 始点・終点の角度計算
    let start_angle = (start.y() - center.y()).atan2(start.x() - center.x());
    let end_angle = (end.y() - center.y()).atan2(end.x() - center.x());

    // 中心角計算（方向を考慮）
    let sweep_angle = match direction {
        ArcDirection::CounterClockwise => {
            if end_angle >= start_angle {
                end_angle - start_angle
            } else {
                end_angle - start_angle + 2.0 * std::f64::consts::PI
            }
        }
        ArcDirection::Clockwise => {
            if end_angle <= start_angle {
                end_angle - start_angle
            } else {
                end_angle - start_angle - 2.0 * std::f64::consts::PI
            }
        }
    };

    // 分割点を生成
    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let angle = start_angle + sweep_angle * t;

        let x = center.x() + radius * angle.cos();
        let y = center.y() + radius * angle.sin();
        let z = start.z() + (end.z() - start.z()) * t; // Z方向は線形補間

        result.push(Vertex3D {
            position: [x as f32, y as f32, z as f32],
        });
    }

    result
}

/// デバッグ用：サンプル工具経路を生成
///
/// シンプルな矩形加工経路を返します（UI表示テスト用）
pub fn create_sample_toolpath() -> ToolPath<f64> {
    cam_core::fixtures::create_sample_toolpath()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cam_core::ToolPath;

    #[test]
    fn test_default_color_scheme() {
        let scheme = ToolPathColorScheme::default();
        assert_eq!(scheme.cutting, [0.0, 0.55, 1.0, 1.0]); // 青みの強いシアン
        assert_eq!(scheme.rapid, [0.75, 0.75, 0.75, 1.0]); // ライトグレー
    }

    #[test]
    fn test_default_tessellation() {
        let settings = TessellationSettings::default();
        assert_eq!(settings.arc_segments, 32);
        assert_eq!(settings.min_segment_length, 0.1);
    }

    #[test]
    fn test_empty_toolpath_conversion() {
        let toolpath = ToolPath::new(
            "tool1".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![],
            vec![],
        );

        let settings = ToolPathVisualizationSettings::default();
        let result = toolpath_to_vertices(&toolpath, &settings);

        assert_eq!(result.vertices.len(), 0);
        assert_eq!(result.colors.len(), 0);
    }

    #[test]
    fn test_simple_line_segment_conversion() {
        use cam_core::{ContourLevelPath, PathSegment, SegmentType};

        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        );

        let contour = ContourLevelPath::new(0, 0.0, vec![segment]);

        let toolpath = ToolPath::new(
            "tool1".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![contour],
            vec![],
        );

        let settings = ToolPathVisualizationSettings::default();
        let result = toolpath_to_vertices(&toolpath, &settings);

        // 1つの線分 = 2頂点
        assert_eq!(result.vertices.len(), 2);
        assert_eq!(result.colors.len(), 1);

        // 始点確認
        assert_eq!(result.vertices[0].position, [0.0, 0.0, 0.0]);
        // 終点確認
        assert_eq!(result.vertices[1].position, [10.0, 0.0, 0.0]);

        // 切削色確認（デフォルト: 青みの強いシアン）
        assert_eq!(result.colors[0], [0.0, 0.55, 1.0, 1.0]);
    }

    #[test]
    fn test_cutting_direction_coloring() {
        use cam_core::{ContourLevelPath, PathSegment, SegmentType};

        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        );

        let contour = ContourLevelPath::new(0, 0.0, vec![segment]);

        // ダウンカット
        let toolpath_down = ToolPath::new(
            "tool1".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![contour.clone()],
            vec![],
        );

        let settings = ToolPathVisualizationSettings {
            color_by_cutting_direction: true,
            ..Default::default()
        };

        let result_down = toolpath_to_vertices(&toolpath_down, &settings);
        assert_eq!(result_down.colors[0], settings.color_scheme.down_cut);

        // アップカット
        let toolpath_up = ToolPath::new(
            "tool1".to_string(),
            CuttingDirection::Up,
            vec![],
            vec![contour],
            vec![],
        );

        let result_up = toolpath_to_vertices(&toolpath_up, &settings);
        assert_eq!(result_up.colors[0], settings.color_scheme.up_cut);
    }

    #[test]
    fn test_arc_tessellation() {
        // 半円（180度）のテッセレーション
        let start = Point3D::new(10.0, 0.0, 0.0);
        let end = Point3D::new(-10.0, 0.0, 0.0);
        let center = Point3D::new(0.0, 0.0, 0.0);

        let vertices = tessellate_arc(&start, &end, &center, ArcDirection::CounterClockwise, 8);

        // 9頂点（8セグメント + 1）
        assert_eq!(vertices.len(), 9);

        // 始点確認
        assert!((vertices[0].position[0] - 10.0).abs() < 0.01);
        assert!((vertices[0].position[1] - 0.0).abs() < 0.01);

        // 終点確認
        assert!((vertices[8].position[0] - (-10.0)).abs() < 0.01);
        assert!((vertices[8].position[1] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_sample_toolpath_is_time_continuous() {
        fn segment_end(segment: &PathSegment<f64>) -> Point3D<f64> {
            match segment.geometry {
                PathGeometry::Line { end } | PathGeometry::Arc { end, .. } => end,
            }
        }

        let toolpath = create_sample_toolpath();
        let mut ordered = Vec::new();
        ordered.extend(toolpath.approach_segments.iter());
        for contour in &toolpath.contour_levels {
            ordered.extend(contour.segments.iter());
        }
        ordered.extend(toolpath.retract_segments.iter());

        for pair in ordered.windows(2) {
            let prev = pair[0];
            let next = pair[1];
            let prev_end = segment_end(prev);
            let next_start = next.start;

            let dx = (prev_end.x() - next_start.x()).abs();
            let dy = (prev_end.y() - next_start.y()).abs();
            let dz = (prev_end.z() - next_start.z()).abs();

            assert!(
                dx <= 1e-9 && dy <= 1e-9 && dz <= 1e-9,
                "toolpath discontinuity: prev_end=({:.6},{:.6},{:.6}) next_start=({:.6},{:.6},{:.6})",
                prev_end.x(),
                prev_end.y(),
                prev_end.z(),
                next_start.x(),
                next_start.y(),
                next_start.z()
            );
        }
    }
}
