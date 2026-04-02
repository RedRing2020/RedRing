//! SVG形状データパーサー
//!
//! SVGファイルから基本的な幾何形状（Circle, Line, Triangle, Arc）を抽出します。
//! RedRingのCAD/CAMシステムで使用するテストデータおよびサンプルデータの
//! 外部管理を目的としています。
//!
//! # サポート要素
//!
//! - `<circle>` - 円形状
//! - `<line>` - 線分
//! - `<polygon>` - 多角形（3点の場合は三角形として扱う）
//! - `<path d="M... A...">` - 円弧（楕円弧コマンド）
//!
//! # 使用例
//!
//! ```rust,no_run
//! use geo_io::svg;
//! use std::path::Path;
//!
//! let shapes = svg::parse_svg_file(Path::new("shapes.svg"))?;
//! println!("円の数: {}", shapes.circles.len());
//! println!("線分の数: {}", shapes.lines.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
use std::path::Path;
use thiserror::Error;

/// SVG解析エラー
#[derive(Error, Debug)]
pub enum SvgError {
    #[error("Failed to read SVG file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse XML: {0}")]
    XmlError(#[from] roxmltree::Error),

    #[error("Invalid SVG structure: {0}")]
    InvalidStructure(String),

    #[error("Invalid attribute value: {0}")]
    InvalidAttribute(String),

    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),

    #[error("Unsupported SVG feature: {0}")]
    UnsupportedFeature(String),
}

/// 円形状データ
#[derive(Debug, Clone, PartialEq)]
pub struct CircleData {
    /// 中心座標 (x, y, z) - SVGは2Dなのでz=0
    pub center: (f64, f64, f64),
    /// 法線ベクトル (x, y, z) - 常に(0, 0, 1)
    pub normal: (f64, f64, f64),
    /// 半径
    pub radius: f64,
}

/// 線分データ
#[derive(Debug, Clone, PartialEq)]
pub struct LineData {
    /// 始点 (x, y, z)
    pub start: (f64, f64, f64),
    /// 終点 (x, y, z)
    pub end: (f64, f64, f64),
}

/// 三角形データ
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleData {
    /// 頂点0 (x, y, z)
    pub p0: (f64, f64, f64),
    /// 頂点1 (x, y, z)
    pub p1: (f64, f64, f64),
    /// 頂点2 (x, y, z)
    pub p2: (f64, f64, f64),
}

/// 円弧データ
#[derive(Debug, Clone, PartialEq)]
pub struct ArcData {
    /// 中心座標 (x, y, z)
    pub center: (f64, f64, f64),
    /// 半径
    pub radius: f64,
    /// 法線ベクトル (x, y, z)
    pub normal: (f64, f64, f64),
    /// 開始方向ベクトル (x, y, z)
    pub start_direction: (f64, f64, f64),
    /// 開始角度（ラジアン）
    pub start_angle: f64,
    /// 終了角度（ラジアン）
    pub end_angle: f64,
}

/// NURBS曲線データ
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurveData {
    /// 次数
    pub degree: usize,
    /// 制御点 (x, y, z)
    pub control_points: Vec<(f64, f64, f64)>,
    /// ノットベクトル
    pub knots: Vec<f64>,
    /// 重み（None = 非有理NURBS）
    pub weights: Option<Vec<f64>>,
}

/// SVGから抽出された形状データ
#[derive(Debug, Default)]
pub struct SvgShapeData {
    /// 円形状のリスト
    pub circles: Vec<CircleData>,
    /// 線分のリスト
    pub lines: Vec<LineData>,
    /// 三角形のリスト
    pub triangles: Vec<TriangleData>,
    /// 円弧のリスト
    pub arcs: Vec<ArcData>,
    /// NURBS曲線のリスト
    pub nurbs_curves: Vec<NurbsCurveData>,
}

/// SVGファイルをパースして形状データを抽出
pub fn parse_svg_file(path: &Path) -> Result<SvgShapeData, SvgError> {
    let content = fs::read_to_string(path)?;
    parse_svg_string(&content)
}

/// SVG文字列をパースして形状データを抽出
pub fn parse_svg_string(svg_content: &str) -> Result<SvgShapeData, SvgError> {
    let doc = roxmltree::Document::parse(svg_content)?;
    let mut shapes = SvgShapeData::default();

    // SVGルート要素を探す
    let svg_root = doc
        .root()
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "svg")
        .ok_or_else(|| SvgError::InvalidStructure("No SVG root element found".to_string()))?;

    // 全ての子要素を走査
    traverse_elements(&svg_root, &mut shapes)?;

    Ok(shapes)
}

/// 要素を再帰的に走査して形状を抽出
fn traverse_elements(node: &roxmltree::Node, shapes: &mut SvgShapeData) -> Result<(), SvgError> {
    if !node.is_element() {
        return Ok(());
    }

    match node.tag_name().name() {
        "circle" => {
            if let Some(circle) = parse_circle(node)? {
                shapes.circles.push(circle);
            }
        }
        "line" => {
            if let Some(line) = parse_line(node)? {
                shapes.lines.push(line);
            }
        }
        "polygon" => {
            if let Some(triangle) = parse_polygon(node)? {
                shapes.triangles.push(triangle);
            }
        }
        "path" => {
            // NURBS曲線をチェック（data-nurbs属性があれば）
            if let Some(nurbs) = parse_nurbs_path(node)? {
                shapes.nurbs_curves.push(nurbs);
            } else if let Some(arc) = parse_path_arc(node)? {
                // pathからarcを抽出
                shapes.arcs.push(arc);
            }
        }
        _ => {
            // その他の要素は無視（グループ要素などは子要素を走査）
        }
    }

    // 子要素を再帰的に処理
    for child in node.children() {
        traverse_elements(&child, shapes)?;
    }

    Ok(())
}

/// circle要素をパース
fn parse_circle(node: &roxmltree::Node) -> Result<Option<CircleData>, SvgError> {
    let cx = get_attribute_f64(node, "cx")?;
    let cy = get_attribute_f64(node, "cy")?;
    let r = get_attribute_f64(node, "r")?;

    if r <= 0.0 {
        return Ok(None); // 無効な半径は無視
    }

    Ok(Some(CircleData {
        center: (cx, cy, 0.0),
        normal: (0.0, 0.0, 1.0),
        radius: r,
    }))
}

/// line要素をパース
fn parse_line(node: &roxmltree::Node) -> Result<Option<LineData>, SvgError> {
    let x1 = get_attribute_f64(node, "x1")?;
    let y1 = get_attribute_f64(node, "y1")?;
    let x2 = get_attribute_f64(node, "x2")?;
    let y2 = get_attribute_f64(node, "y2")?;

    // 長さゼロの線分は無視
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx * dx + dy * dy < 1e-10 {
        return Ok(None);
    }

    Ok(Some(LineData {
        start: (x1, y1, 0.0),
        end: (x2, y2, 0.0),
    }))
}

/// polygon要素をパース（3点のみ三角形として扱う）
fn parse_polygon(node: &roxmltree::Node) -> Result<Option<TriangleData>, SvgError> {
    let points_str = node
        .attribute("points")
        .ok_or_else(|| SvgError::MissingAttribute("points".to_string()))?;

    let points = parse_points_attribute(points_str)?;

    // 3点のみ三角形として扱う
    if points.len() == 3 {
        Ok(Some(TriangleData {
            p0: (points[0].0, points[0].1, 0.0),
            p1: (points[1].0, points[1].1, 0.0),
            p2: (points[2].0, points[2].1, 0.0),
        }))
    } else {
        // 3点以外は現時点では非対応
        Ok(None)
    }
}

/// path要素から円弧を抽出（簡易実装）
fn parse_path_arc(_node: &roxmltree::Node) -> Result<Option<ArcData>, SvgError> {
    // TODO: pathのd属性をパースしてArcコマンドを抽出
    // 現時点では未実装
    Ok(None)
}

/// path要素からNURBS曲線を抽出
///
/// カスタムdata-*属性を使用してNURBSパラメータを指定します：
/// - data-nurbs="true" - NURBS曲線であることを示す
/// - data-degree="3" - 次数
/// - data-control-points="x1,y1,z1; x2,y2,z2; ..." - 制御点リスト
/// - data-knots="k1,k2,k3,..." - ノットベクトル
/// - data-weights="w1,w2,w3,..." - 重み（オプション）
fn parse_nurbs_path(node: &roxmltree::Node) -> Result<Option<NurbsCurveData>, SvgError> {
    // data-nurbs属性をチェック
    let is_nurbs = node
        .attribute("data-nurbs")
        .map(|v| v == "true")
        .unwrap_or(false);

    if !is_nurbs {
        return Ok(None);
    }

    // 次数を取得
    let degree_str = node
        .attribute("data-degree")
        .ok_or_else(|| SvgError::MissingAttribute("data-degree".to_string()))?;
    let degree = degree_str
        .parse::<usize>()
        .map_err(|_| SvgError::InvalidAttribute(format!("Invalid degree: {}", degree_str)))?;

    // 制御点を取得
    let control_points_str = node
        .attribute("data-control-points")
        .ok_or_else(|| SvgError::MissingAttribute("data-control-points".to_string()))?;
    let control_points = parse_control_points(control_points_str)?;

    // ノットベクトルを取得
    let knots_str = node
        .attribute("data-knots")
        .ok_or_else(|| SvgError::MissingAttribute("data-knots".to_string()))?;
    let knots = parse_float_array(knots_str)?;

    // 重み（オプション）
    let weights = if let Some(weights_str) = node.attribute("data-weights") {
        Some(parse_float_array(weights_str)?)
    } else {
        None
    };

    Ok(Some(NurbsCurveData {
        degree,
        control_points,
        knots,
        weights,
    }))
}

/// 制御点リストをパース（"x1,y1,z1; x2,y2,z2; ..."形式）
fn parse_control_points(s: &str) -> Result<Vec<(f64, f64, f64)>, SvgError> {
    let mut points = Vec::new();

    for point_str in s.split(';') {
        let coords: Vec<&str> = point_str.split(',').map(str::trim).collect();
        if coords.len() != 3 {
            return Err(SvgError::InvalidAttribute(format!(
                "Invalid control point format: {}",
                point_str
            )));
        }

        let x = coords[0]
            .parse::<f64>()
            .map_err(|_| SvgError::InvalidAttribute(format!("Invalid x: {}", coords[0])))?;
        let y = coords[1]
            .parse::<f64>()
            .map_err(|_| SvgError::InvalidAttribute(format!("Invalid y: {}", coords[1])))?;
        let z = coords[2]
            .parse::<f64>()
            .map_err(|_| SvgError::InvalidAttribute(format!("Invalid z: {}", coords[2])))?;

        points.push((x, y, z));
    }

    Ok(points)
}

/// 浮動小数点数配列をパース（"v1,v2,v3,..."形式）
fn parse_float_array(s: &str) -> Result<Vec<f64>, SvgError> {
    s.split(',')
        .map(str::trim)
        .map(|v| {
            v.parse::<f64>()
                .map_err(|_| SvgError::InvalidAttribute(format!("Invalid float: {}", v)))
        })
        .collect()
}

/// points属性をパースして座標リストに変換
fn parse_points_attribute(points_str: &str) -> Result<Vec<(f64, f64)>, SvgError> {
    let mut points = Vec::new();
    let tokens: Vec<&str> = points_str
        .split(|c: char| c.is_whitespace() || c == ',')
        .collect();

    let mut i = 0;
    while i + 1 < tokens.len() {
        if !tokens[i].is_empty() && !tokens[i + 1].is_empty() {
            let x = tokens[i]
                .parse::<f64>()
                .map_err(|_| SvgError::InvalidAttribute(format!("Invalid x: {}", tokens[i])))?;
            let y = tokens[i + 1]
                .parse::<f64>()
                .map_err(|_| SvgError::InvalidAttribute(format!("Invalid y: {}", tokens[i + 1])))?;
            points.push((x, y));
            i += 2;
        } else {
            i += 1;
        }
    }

    Ok(points)
}

/// ノードから属性をf64として取得
fn get_attribute_f64(node: &roxmltree::Node, name: &str) -> Result<f64, SvgError> {
    let value_str = node
        .attribute(name)
        .ok_or_else(|| SvgError::MissingAttribute(name.to_string()))?;

    value_str
        .parse::<f64>()
        .map_err(|_| SvgError::InvalidAttribute(format!("Invalid {}: {}", name, value_str)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_circle() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <circle cx="10" cy="20" r="5" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.circles.len(), 1);
        let circle = &shapes.circles[0];
        assert_eq!(circle.center, (10.0, 20.0, 0.0));
        assert_eq!(circle.radius, 5.0);
    }

    #[test]
    fn test_parse_line() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <line x1="-2" y1="0" x2="2" y2="0" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.lines.len(), 1);
        let line = &shapes.lines[0];
        assert_eq!(line.start, (-2.0, 0.0, 0.0));
        assert_eq!(line.end, (2.0, 0.0, 0.0));
    }

    #[test]
    fn test_parse_triangle() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <polygon points="0,0 1,0 0.5,1" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.triangles.len(), 1);
        let tri = &shapes.triangles[0];
        assert_eq!(tri.p0, (0.0, 0.0, 0.0));
        assert_eq!(tri.p1, (1.0, 0.0, 0.0));
        assert_eq!(tri.p2, (0.5, 1.0, 0.0));
    }

    #[test]
    fn test_parse_multiple_shapes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <circle cx="0" cy="0" r="5" />
                <line x1="0" y1="0" x2="10" y2="10" />
                <polygon points="0,0 1,0 0.5,1" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.circles.len(), 1);
        assert_eq!(shapes.lines.len(), 1);
        assert_eq!(shapes.triangles.len(), 1);
    }

    #[test]
    fn test_ignore_invalid_radius() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <circle cx="0" cy="0" r="0" />
                <circle cx="0" cy="0" r="-5" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.circles.len(), 0);
    }

    #[test]
    fn test_ignore_zero_length_line() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <line x1="0" y1="0" x2="0" y2="0" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.lines.len(), 0);
    }

    #[test]
    fn test_parse_nurbs_curve() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <path
                    data-nurbs="true"
                    data-degree="3"
                    data-control-points="0,0,0; 1,0,0; 1,1,0; 0,1,0"
                    data-knots="0,0,0,0,1,1,1,1"
                    data-weights="1,1,1,1" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.nurbs_curves.len(), 1);
        let nurbs = &shapes.nurbs_curves[0];
        assert_eq!(nurbs.degree, 3);
        assert_eq!(nurbs.control_points.len(), 4);
        assert_eq!(nurbs.control_points[0], (0.0, 0.0, 0.0));
        assert_eq!(nurbs.control_points[3], (0.0, 1.0, 0.0));
        assert_eq!(nurbs.knots.len(), 8);
        assert_eq!(nurbs.weights.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn test_parse_nurbs_without_weights() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <path
                    data-nurbs="true"
                    data-degree="2"
                    data-control-points="0,0,0; 1,0,0; 1,1,0"
                    data-knots="0,0,0,1,1,1" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.nurbs_curves.len(), 1);
        let nurbs = &shapes.nurbs_curves[0];
        assert_eq!(nurbs.degree, 2);
        assert_eq!(nurbs.control_points.len(), 3);
        assert!(nurbs.weights.is_none());
    }
}
