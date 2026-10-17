//! SVG形状データパーサー
//!
//! SVGファイルから基本的な幾何形状を抽出します。
//! 形状の表示を実機でデバッグするための、テストデータの読み込みに使います。
//!
//! # サポート要素
//!
//! - `<circle>` - 円形状
//! - `<line>` - 線分
//! - `<polygon>` - 三角形（3 点の場合。4 点以上は未対応のエラー）
//! - `<path d="...">` - 線分・円弧・楕円弧（M / L / H / V / Z / A コマンド。Bézier 曲線は未対応のエラー）
//! - `<path data-nurbs="true" ...>` - NURBS 曲線
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

use geo_contracts::{Angle, Direction3D, Point3D};
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
    /// 中心 - SVGは2Dなのでz=0
    pub center: Point3D<f64>,
    /// 法線 - 常に Z 軸正方向
    pub normal: Direction3D<f64>,
    /// 半径
    pub radius: f64,
}

/// 線分データ
#[derive(Debug, Clone, PartialEq)]
pub struct LineData {
    /// 始点
    pub start: Point3D<f64>,
    /// 終点
    pub end: Point3D<f64>,
}

/// 三角形データ
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleData {
    /// 頂点0
    pub p0: Point3D<f64>,
    /// 頂点1
    pub p1: Point3D<f64>,
    /// 頂点2
    pub p2: Point3D<f64>,
}

/// 円弧データ
#[derive(Debug, Clone, PartialEq)]
pub struct ArcData {
    /// 中心
    pub center: Point3D<f64>,
    /// 半径
    pub radius: f64,
    /// 円弧平面の法線
    pub normal: Direction3D<f64>,
    /// 角度 0 の方向
    pub start_direction: Direction3D<f64>,
    /// 開始角
    pub start_angle: Angle<f64>,
    /// 終了角（開始角から反時計回り）
    pub end_angle: Angle<f64>,
}

/// 楕円弧データ
#[derive(Debug, Clone, PartialEq)]
pub struct EllipseArcData {
    /// 中心
    pub center: Point3D<f64>,
    /// 長半軸
    pub semi_major: f64,
    /// 短半軸
    pub semi_minor: f64,
    /// 楕円弧平面の法線
    pub normal: Direction3D<f64>,
    /// 長軸の方向（角度 0 の方向）
    pub major_axis_direction: Direction3D<f64>,
    /// 開始角（楕円のパラメータの角度）
    pub start_angle: Angle<f64>,
    /// 終了角（開始角から反時計回り）
    pub end_angle: Angle<f64>,
}

/// NURBS曲線データ
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurveData {
    /// 次数
    pub degree: usize,
    /// 制御点
    pub control_points: Vec<Point3D<f64>>,
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
    /// 楕円弧のリスト
    pub ellipse_arcs: Vec<EllipseArcData>,
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
            // data-nurbs 属性があれば NURBS 曲線、なければ d 属性の線分・円弧・楕円弧
            if let Some(nurbs) = parse_nurbs_path(node)? {
                shapes.nurbs_curves.push(nurbs);
            } else {
                parse_path_data(node, shapes)?;
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
        center: Point3D::new(cx, cy, 0.0),
        normal: Direction3D::positive_z(),
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
        start: Point3D::new(x1, y1, 0.0),
        end: Point3D::new(x2, y2, 0.0),
    }))
}

/// polygon要素をパース（3点のみ三角形として扱う）
fn parse_polygon(node: &roxmltree::Node) -> Result<Option<TriangleData>, SvgError> {
    let points_str = node
        .attribute("points")
        .ok_or_else(|| SvgError::MissingAttribute("points".to_string()))?;

    let points = parse_points_attribute(points_str)?;

    // 3 点は三角形。4 点以上の多角形の扱いは決まっていないので未対応のエラーにする
    match points.len() {
        3 => Ok(Some(TriangleData {
            p0: Point3D::new(points[0].0, points[0].1, 0.0),
            p1: Point3D::new(points[1].0, points[1].1, 0.0),
            p2: Point3D::new(points[2].0, points[2].1, 0.0),
        })),
        count if count > 3 => Err(SvgError::UnsupportedFeature(format!(
            "polygon with {count} points"
        ))),
        _ => Ok(None),
    }
}

/// path要素の d 属性から線分・円弧・楕円弧を抽出
///
/// 読み取るコマンドは M / L / H / V / Z / A（小文字は相対座標）。Bézier 曲線（C / S / Q / T）は
/// 未対応のエラーにする。円弧の中心と角度は SVG の仕様（SVG 1.1 付録 F.6.5・F.6.6）に従って求める。
fn parse_path_data(node: &roxmltree::Node, shapes: &mut SvgShapeData) -> Result<(), SvgError> {
    let data = node
        .attribute("d")
        .ok_or_else(|| SvgError::MissingAttribute("d".to_string()))?;
    let mut reader = PathDataReader::new(data);
    let mut current = (0.0, 0.0);
    let mut subpath_start = (0.0, 0.0);
    let mut command = None;

    while let Some(next) = reader.next_command_or_implicit(command)? {
        let relative = next.is_ascii_lowercase();
        let offset = if relative { current } else { (0.0, 0.0) };
        match next.to_ascii_uppercase() {
            'M' => {
                let (x, y) = reader.coordinate_pair()?;
                current = (offset.0 + x, offset.1 + y);
                subpath_start = current;
                // 続く座標の組は直線（L / l）として扱う
                command = Some(if relative { 'l' } else { 'L' });
                continue;
            }
            'L' => {
                let (x, y) = reader.coordinate_pair()?;
                current = push_line(shapes, current, (offset.0 + x, offset.1 + y));
            }
            'H' => {
                let x = reader.number()?;
                current = push_line(shapes, current, (offset.0 + x, current.1));
            }
            'V' => {
                let y = reader.number()?;
                current = push_line(shapes, current, (current.0, offset.1 + y));
            }
            'Z' => {
                current = push_line(shapes, current, subpath_start);
            }
            'A' => {
                let rx = reader.number()?;
                let ry = reader.number()?;
                let rotation = reader.number()?;
                let large_arc = reader.flag()?;
                let sweep = reader.flag()?;
                let (x, y) = reader.coordinate_pair()?;
                let end = (offset.0 + x, offset.1 + y);
                push_arc(shapes, current, end, (rx, ry), rotation, large_arc, sweep)?;
                current = end;
            }
            other => {
                return Err(SvgError::UnsupportedFeature(format!(
                    "path command '{other}'"
                )));
            }
        }
        command = Some(next);
    }
    Ok(())
}

/// 長さが 0 でない線分を加え、終点を返す
fn push_line(shapes: &mut SvgShapeData, start: (f64, f64), end: (f64, f64)) -> (f64, f64) {
    if start != end {
        shapes.lines.push(LineData {
            start: Point3D::new(start.0, start.1, 0.0),
            end: Point3D::new(end.0, end.1, 0.0),
        });
    }
    end
}

/// 端点で表した楕円弧（A コマンド）を中心と角度で表し、円弧または楕円弧として加える
///
/// SVG の座標をそのまま使うので、向きのフラグ 1（角度が増える向き）は法線が Z 軸正方向の反時計回りになる。
/// 端点が一致する場合は何も加えず、半径が 0 の場合は線分とする（F.6.2）。
fn push_arc(
    shapes: &mut SvgShapeData,
    start: (f64, f64),
    end: (f64, f64),
    radii: (f64, f64),
    rotation_degrees: f64,
    large_arc: bool,
    sweep: bool,
) -> Result<(), SvgError> {
    if start == end {
        return Ok(());
    }
    let (mut rx, mut ry) = (radii.0.abs(), radii.1.abs());
    if rx == 0.0 || ry == 0.0 {
        push_line(shapes, start, end);
        return Ok(());
    }
    let rotation = rotation_degrees.to_radians();
    let (cos, sin) = (rotation.cos(), rotation.sin());

    // F.6.5 の手順 1: 端点の中点を原点とし、楕円の軸に揃えた座標
    let (half_dx, half_dy) = ((start.0 - end.0) / 2.0, (start.1 - end.1) / 2.0);
    let x1 = cos * half_dx + sin * half_dy;
    let y1 = -sin * half_dx + cos * half_dy;

    // F.6.6: 端点に届かない半径を拡大する
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }

    // F.6.5 の手順 2〜3: 中心
    let numerator = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let magnitude = (numerator / denominator).max(0.0).sqrt();
    let coefficient = if large_arc == sweep {
        -magnitude
    } else {
        magnitude
    };
    let center_x1 = coefficient * rx * y1 / ry;
    let center_y1 = -coefficient * ry * x1 / rx;
    let center = Point3D::new(
        cos * center_x1 - sin * center_y1 + (start.0 + end.0) / 2.0,
        sin * center_x1 + cos * center_y1 + (start.1 + end.1) / 2.0,
        0.0,
    );

    // F.6.5 の手順 4: 開始角と角度の幅（向きのフラグで符号を決める）
    let start_angle = ((y1 - center_y1) / ry).atan2((x1 - center_x1) / rx);
    let end_angle = ((-y1 - center_y1) / ry).atan2((-x1 - center_x1) / rx);
    let mut span = end_angle - start_angle;
    if sweep && span < 0.0 {
        span += std::f64::consts::TAU;
    } else if !sweep && span > 0.0 {
        span -= std::f64::consts::TAU;
    }
    // 反時計回りの開始角と終了角（時計回りの弧は終点の側から始める）
    let (from, to) = if sweep {
        (start_angle, start_angle + span)
    } else {
        (start_angle + span, start_angle)
    };

    if rx == ry {
        shapes.arcs.push(ArcData {
            center,
            radius: rx,
            normal: Direction3D::positive_z(),
            start_direction: direction(cos, sin)?,
            start_angle: Angle::from_radians(from),
            end_angle: Angle::from_radians(to),
        });
    } else if rx > ry {
        shapes.ellipse_arcs.push(EllipseArcData {
            center,
            semi_major: rx,
            semi_minor: ry,
            normal: Direction3D::positive_z(),
            major_axis_direction: direction(cos, sin)?,
            start_angle: Angle::from_radians(from),
            end_angle: Angle::from_radians(to),
        });
    } else {
        // y 方向の半径が長い場合は、長軸を 90 度回した方向とし、角度を 90 度戻す
        let quarter = std::f64::consts::FRAC_PI_2;
        shapes.ellipse_arcs.push(EllipseArcData {
            center,
            semi_major: ry,
            semi_minor: rx,
            normal: Direction3D::positive_z(),
            major_axis_direction: direction(-sin, cos)?,
            start_angle: Angle::from_radians(from - quarter),
            end_angle: Angle::from_radians(to - quarter),
        });
    }
    Ok(())
}

/// XY 平面上の方向
fn direction(x: f64, y: f64) -> Result<Direction3D<f64>, SvgError> {
    Direction3D::new(x, y, 0.0)
        .ok_or_else(|| SvgError::InvalidAttribute(format!("Invalid direction: ({x}, {y})")))
}

/// path要素の d 属性の字句の読み取り
struct PathDataReader<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> PathDataReader<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            chars: data.chars().peekable(),
        }
    }

    fn skip_separators(&mut self) {
        while self
            .chars
            .peek()
            .is_some_and(|c| c.is_whitespace() || *c == ',')
        {
            self.chars.next();
        }
    }

    /// 次のコマンド。コマンドの文字がなく数値が続く場合は、前のコマンドを繰り返す
    fn next_command_or_implicit(
        &mut self,
        previous: Option<char>,
    ) -> Result<Option<char>, SvgError> {
        self.skip_separators();
        match self.chars.peek() {
            None => Ok(None),
            Some(c) if c.is_ascii_alphabetic() => Ok(self.chars.next()),
            Some(_) => match previous {
                Some(previous) if !previous.eq_ignore_ascii_case(&'Z') => Ok(Some(previous)),
                _ => Err(SvgError::InvalidAttribute(
                    "path data must start with a command".to_string(),
                )),
            },
        }
    }

    fn number(&mut self) -> Result<f64, SvgError> {
        self.skip_separators();
        let mut text = String::new();
        if let Some(sign) = self.chars.next_if(|c| *c == '+' || *c == '-') {
            text.push(sign);
        }
        let mut seen_point = false;
        let mut seen_exponent = false;
        while let Some(c) = self.chars.peek().copied() {
            if c.is_ascii_digit() {
                text.push(c);
            } else if c == '.' && !seen_point && !seen_exponent {
                seen_point = true;
                text.push(c);
            } else if (c == 'e' || c == 'E') && !seen_exponent && !text.is_empty() {
                seen_exponent = true;
                text.push(c);
                self.chars.next();
                if let Some(sign) = self.chars.next_if(|c| *c == '+' || *c == '-') {
                    text.push(sign);
                }
                continue;
            } else {
                break;
            }
            self.chars.next();
        }
        text.parse::<f64>()
            .map_err(|_| SvgError::InvalidAttribute(format!("Invalid path number: '{text}'")))
    }

    fn coordinate_pair(&mut self) -> Result<(f64, f64), SvgError> {
        Ok((self.number()?, self.number()?))
    }

    /// A コマンドのフラグ（区切りなしで続けて書かれることがあるので 1 文字で読む）
    fn flag(&mut self) -> Result<bool, SvgError> {
        self.skip_separators();
        match self.chars.next() {
            Some('0') => Ok(false),
            Some('1') => Ok(true),
            other => Err(SvgError::InvalidAttribute(format!(
                "Invalid arc flag: {other:?}"
            ))),
        }
    }
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
fn parse_control_points(s: &str) -> Result<Vec<Point3D<f64>>, SvgError> {
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

        points.push(Point3D::new(x, y, z));
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
        assert_eq!(circle.center, Point3D::new(10.0, 20.0, 0.0));
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
        assert_eq!(line.start, Point3D::new(-2.0, 0.0, 0.0));
        assert_eq!(line.end, Point3D::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_parse_triangle() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <polygon points="0,0 1,0 0.5,1" />
            </svg>"#;

        let shapes = parse_svg_string(svg).unwrap();
        assert_eq!(shapes.triangles.len(), 1);
        let tri = &shapes.triangles[0];
        assert_eq!(tri.p0, Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(tri.p1, Point3D::new(1.0, 0.0, 0.0));
        assert_eq!(tri.p2, Point3D::new(0.5, 1.0, 0.0));
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
        assert_eq!(nurbs.control_points[0], Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(nurbs.control_points[3], Point3D::new(0.0, 1.0, 0.0));
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

    fn path_shapes(d: &str) -> Result<SvgShapeData, SvgError> {
        parse_svg_string(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="{d}" /></svg>"#
        ))
    }

    fn point(x: f64, y: f64) -> Point3D<f64> {
        Point3D::new(x, y, 0.0)
    }

    /// 円弧の開始角・終了角の点
    fn arc_ends(arc: &ArcData) -> (Point3D<f64>, Point3D<f64>) {
        let at = |angle: Angle<f64>| {
            let reference = arc.start_direction.as_vector();
            let normal = arc.normal.as_vector();
            let side = normal.cross(reference);
            let (cos, sin) = (angle.to_radians().cos(), angle.to_radians().sin());
            arc.center + (reference * cos + side * sin) * arc.radius
        };
        (at(arc.start_angle), at(arc.end_angle))
    }

    /// 楕円弧の開始角・終了角の点
    fn ellipse_arc_ends(arc: &EllipseArcData) -> (Point3D<f64>, Point3D<f64>) {
        let at = |angle: Angle<f64>| {
            let major = arc.major_axis_direction.as_vector();
            let minor = arc.normal.as_vector().cross(major);
            let (cos, sin) = (angle.to_radians().cos(), angle.to_radians().sin());
            arc.center + major * (arc.semi_major * cos) + minor * (arc.semi_minor * sin)
        };
        (at(arc.start_angle), at(arc.end_angle))
    }

    /// 反時計回りの両端が、SVG の始点・終点（順不同）と一致する
    fn assert_same_ends(actual: (Point3D<f64>, Point3D<f64>), a: Point3D<f64>, b: Point3D<f64>) {
        let close = |p: Point3D<f64>, q: Point3D<f64>| p.distance_to(q) < 1e-12;
        assert!(
            (close(actual.0, a) && close(actual.1, b))
                || (close(actual.0, b) && close(actual.1, a)),
            "ends: {actual:?}"
        );
    }

    fn span_degrees(start: Angle<f64>, end: Angle<f64>) -> f64 {
        (end.to_radians() - start.to_radians()).to_degrees()
    }

    #[test]
    fn test_parse_path_lines() {
        let shapes = path_shapes("M 0 0 L 1 0 H 2 V 1 Z").unwrap();
        let segments: Vec<_> = shapes.lines.iter().map(|l| (l.start, l.end)).collect();
        assert_eq!(
            segments,
            vec![
                (point(0.0, 0.0), point(1.0, 0.0)),
                (point(1.0, 0.0), point(2.0, 0.0)),
                (point(2.0, 0.0), point(2.0, 1.0)),
                (point(2.0, 1.0), point(0.0, 0.0)),
            ]
        );

        // 相対座標と、M の後に続く座標の組（直線として扱う）
        let shapes = path_shapes("m 1 1 2 0 0 2 z").unwrap();
        let segments: Vec<_> = shapes.lines.iter().map(|l| (l.start, l.end)).collect();
        assert_eq!(
            segments,
            vec![
                (point(1.0, 1.0), point(3.0, 1.0)),
                (point(3.0, 1.0), point(3.0, 3.0)),
                (point(3.0, 3.0), point(1.0, 1.0)),
            ]
        );
    }

    #[test]
    fn test_parse_path_circular_arcs() {
        // 小さい弧・角度が増える向き: 中心 (0, 0)、0 度から 90 度
        let shapes = path_shapes("M 1 0 A 1 1 0 0 1 0 1").unwrap();
        let arc = &shapes.arcs[0];
        assert!(arc.center.distance_to(point(0.0, 0.0)) < 1e-12);
        assert!((span_degrees(arc.start_angle, arc.end_angle) - 90.0).abs() < 1e-9);
        assert_same_ends(arc_ends(arc), point(1.0, 0.0), point(0.0, 1.0));

        // 小さい弧・角度が減る向き: 中心 (1, 1)
        let shapes = path_shapes("M 1 0 A 1 1 0 0 0 0 1").unwrap();
        let arc = &shapes.arcs[0];
        assert!(arc.center.distance_to(point(1.0, 1.0)) < 1e-12);
        assert!((span_degrees(arc.start_angle, arc.end_angle) - 90.0).abs() < 1e-9);
        assert_same_ends(arc_ends(arc), point(1.0, 0.0), point(0.0, 1.0));

        // 大きい弧・角度が増える向き（フラグを区切らずに書く）: 中心 (1, 1)、270 度
        let shapes = path_shapes("M1,0A1,1,0,11,0,1").unwrap();
        let arc = &shapes.arcs[0];
        assert!(arc.center.distance_to(point(1.0, 1.0)) < 1e-12);
        assert!((span_degrees(arc.start_angle, arc.end_angle) - 270.0).abs() < 1e-9);
        assert_same_ends(arc_ends(arc), point(1.0, 0.0), point(0.0, 1.0));

        // 端点に届かない半径は拡大する（半径 1、中心 (1, 0) の半円）
        let shapes = path_shapes("M 0 0 A 0.5 0.5 0 0 1 2 0").unwrap();
        let arc = &shapes.arcs[0];
        assert!((arc.radius - 1.0).abs() < 1e-12);
        assert!(arc.center.distance_to(point(1.0, 0.0)) < 1e-12);
        assert_same_ends(arc_ends(arc), point(0.0, 0.0), point(2.0, 0.0));
    }

    #[test]
    fn test_parse_path_elliptical_arcs() {
        // x 方向の半径が長い楕円弧
        let shapes = path_shapes("M 2 0 A 2 1 0 0 1 0 1").unwrap();
        let arc = &shapes.ellipse_arcs[0];
        assert!(arc.center.distance_to(point(0.0, 0.0)) < 1e-12);
        assert_eq!((arc.semi_major, arc.semi_minor), (2.0, 1.0));
        assert_same_ends(ellipse_arc_ends(arc), point(2.0, 0.0), point(0.0, 1.0));

        // y 方向の半径が長い楕円弧（長軸は y 方向）
        let shapes = path_shapes("M 1 0 A 1 2 0 0 1 0 2").unwrap();
        let arc = &shapes.ellipse_arcs[0];
        assert_eq!((arc.semi_major, arc.semi_minor), (2.0, 1.0));
        assert!(arc.major_axis_direction.as_vector().y().abs() > 1.0 - 1e-12);
        assert_same_ends(ellipse_arc_ends(arc), point(1.0, 0.0), point(0.0, 2.0));

        // 回転した楕円弧（長軸が 45 度）
        let shapes =
            path_shapes("M 0 0 A 2 1 45 0 1 1.4142135623730951 1.4142135623730951").unwrap();
        let arc = &shapes.ellipse_arcs[0];
        assert_same_ends(
            ellipse_arc_ends(arc),
            point(0.0, 0.0),
            point(2.0_f64.sqrt(), 2.0_f64.sqrt()),
        );
    }

    #[test]
    fn test_parse_path_degenerate_arcs() {
        // 端点が一致する円弧は加えず、半径 0 の円弧は線分とする
        let shapes = path_shapes("M 1 1 A 1 1 0 0 1 1 1").unwrap();
        assert!(shapes.arcs.is_empty() && shapes.lines.is_empty());
        let shapes = path_shapes("M 0 0 A 0 1 0 0 1 2 0").unwrap();
        assert_eq!(shapes.lines.len(), 1);
    }

    #[test]
    fn test_unsupported_path_and_polygon_are_errors() {
        assert!(matches!(
            path_shapes("M 0 0 C 1 1 2 1 3 0"),
            Err(SvgError::UnsupportedFeature(_))
        ));
        let square = r#"<svg xmlns="http://www.w3.org/2000/svg">
                <polygon points="0,0 1,0 1,1 0,1" />
            </svg>"#;
        assert!(matches!(
            parse_svg_string(square),
            Err(SvgError::UnsupportedFeature(_))
        ));
    }
}
