//! CAM工具経路データ構造
//!
//! このモジュールは、NC工具経路を表現するデータ構造を提供します。
//!
//! # 概要
//!
//! CAMで生成される工具経路は以下の階層構造を持ちます：
//!
//! - **ToolPath**: 工具経路全体（複数の等高線を含む）
//! - **ContourLevelPath**: 単一等高線の経路（複数のセグメントを含む）
//! - **PathSegment**: 単一セグメント（切削、早送り、アプローチ等）
//!
//! # 例
//!
//! ```
//! use cam_core::{ToolPath, ContourLevelPath, PathSegment, SegmentType, CuttingDirection};
//! use geo_primitives::Point3D;
//!
//! // セグメント作成
//! let segment = PathSegment::new_line(
//!     Point3D::new(0.0, 0.0, 0.0),
//!     Point3D::new(10.0, 0.0, 0.0),
//!     SegmentType::Cutting {
//!         feed_rate: 500.0,
//!     },
//! );
//!
//! // 等高線経路作成
//! let contour = ContourLevelPath::new(
//!     0,
//!     -5.0,
//!     vec![segment],
//! );
//!
//! // 工具経路作成
//! let toolpath = ToolPath::new(
//!     "tool1".to_string(),
//!     CuttingDirection::Down,
//!     vec![],  // approach_segments
//!     vec![contour],
//!     vec![],  // retract_segments
//! );
//! ```

use analysis::Scalar;
use geo_primitives::Point3D;

/// 円弧方向
///
/// Gコードの円弧補間（G02/G03）に対応します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcDirection {
    /// 時計回り（G02: Clockwise）
    ///
    /// XY平面上で、+Z方向から見て時計回りの円弧。
    Clockwise,

    /// 反時計回り（G03: Counter-Clockwise）
    ///
    /// XY平面上で、+Z方向から見て反時計回りの円弧。
    CounterClockwise,
}

/// 経路の幾何形状
///
/// 工具経路セグメントの幾何的な形状を定義します。
/// 将来的にはNURBS補間も対応予定です。
#[derive(Debug, Clone, PartialEq)]
pub enum PathGeometry<T: Scalar> {
    /// 直線補間（G01）
    ///
    /// 始点から終点への直線移動。
    Line {
        /// 終点座標
        end: Point3D<T>,
    },

    /// 円弧補間（G02/G03）
    ///
    /// 始点から終点への円弧移動。
    Arc {
        /// 終点座標
        end: Point3D<T>,
        /// 円弧中心点座標
        center: Point3D<T>,
        /// 円弧方向（時計回り/反時計回り）
        direction: ArcDirection,
    },
    // 将来の拡張:
    // /// NURBS補間
    // Nurbs {
    //     control_points: Vec<Point3D<T>>,
    //     knots: Vec<T>,
    //     degree: usize,
    // },
}

impl<T: Scalar> PathGeometry<T> {
    /// 終点座標を取得
    pub fn end_point(&self) -> Point3D<T> {
        match self {
            PathGeometry::Line { end } => *end,
            PathGeometry::Arc { end, .. } => *end,
        }
    }
}

/// セグメント種別
///
/// 工具経路の各セグメントがどのような動作を表すか定義します。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentType<T: Scalar = f64> {
    /// 切削セグメント（G01/G02/G03）
    ///
    /// 実際に材料を切削する経路。
    /// 切削方向（ダウンカット/アップカット）は `ToolPath.cutting_direction` で指定。
    Cutting {
        /// 送り速度（mm/min）
        feed_rate: T,
    },

    /// 早送りセグメント（G00）
    ///
    /// エアカット、周回間の水平移動など。
    Rapid,

    /// アプローチセグメント
    ///
    /// 切削開始前の進入動作。
    /// ToolPath開始時（`approach_segments`）または周回間で使用。
    Approach {
        /// 送り速度（mm/min）
        feed_rate: T,
    },

    /// リトラクトセグメント
    ///
    /// 退避動作。低速で材料から離脱、または接触したまま移動。
    /// ToolPath終了時（`retract_segments`）または周回間で使用。
    Retract {
        /// 送り速度（mm/min）
        feed_rate: T,
    },

    /// 周回間リトラクト
    ///
    /// 周回間の退避動作。次に直接 Cutting へ遷移。
    /// 平面切削など、直線1つの退避で済む場合に使用。
    PassRetract {
        /// 送り速度（mm/min）
        feed_rate: T,
    },
}

/// 切削方向
///
/// ダウンカット（同方向）とアップカット（逆方向）を識別します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuttingDirection {
    /// ダウンカット（同方向切削）
    ///
    /// 工具回転方向と送り方向が同じ。仕上がり良好。
    Down,

    /// アップカット（逆方向切削）
    ///
    /// 工具回転方向と送り方向が逆。荒加工向き。
    Up,
}

/// 工具経路セグメント
///
/// 工具経路の最小単位。始点、幾何形状、およびセグメント種別を持ちます。
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment<T: Scalar = f64> {
    /// セグメント始点（絶対座標）
    pub start: Point3D<T>,

    /// 幾何形状（直線または円弧）
    pub geometry: PathGeometry<T>,

    /// セグメント種別
    pub segment_type: SegmentType<T>,
}

impl<T: Scalar> PathSegment<T> {
    /// 新しい直線セグメントを作成
    ///
    /// # 引数
    ///
    /// - `start`: セグメント始点
    /// - `end`: セグメント終点
    /// - `segment_type`: セグメント種別
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{PathSegment, SegmentType, CuttingDirection};
    /// use geo_primitives::Point3D;
    ///
    /// let segment = PathSegment::new_line(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(10.0, 0.0, 0.0),
    ///     SegmentType::Cutting {
    ///         feed_rate: 500.0,
    ///     },
    /// );
    /// ```
    pub fn new_line(start: Point3D<T>, end: Point3D<T>, segment_type: SegmentType<T>) -> Self {
        Self {
            start,
            geometry: PathGeometry::Line { end },
            segment_type,
        }
    }

    /// 新しい円弧セグメントを作成
    ///
    /// # 引数
    ///
    /// - `start`: セグメント始点
    /// - `end`: セグメント終点
    /// - `center`: 円弧中心点
    /// - `direction`: 円弧方向（時計回り/反時計回り）
    /// - `segment_type`: セグメント種別
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{PathSegment, SegmentType, ArcDirection};
    /// use geo_primitives::Point3D;
    ///
    /// let segment = PathSegment::new_arc(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(10.0, 10.0, 0.0),
    ///     Point3D::new(5.0, 5.0, 0.0),
    ///     ArcDirection::CounterClockwise,
    ///     SegmentType::Rapid,
    /// );
    /// ```
    pub fn new_arc(
        start: Point3D<T>,
        end: Point3D<T>,
        center: Point3D<T>,
        direction: ArcDirection,
        segment_type: SegmentType<T>,
    ) -> Self {
        Self {
            start,
            geometry: PathGeometry::Arc {
                end,
                center,
                direction,
            },
            segment_type,
        }
    }

    /// 終点座標を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.geometry.end_point()
    }

    /// 切削セグメントかどうか判定
    pub fn is_cutting(&self) -> bool {
        matches!(self.segment_type, SegmentType::Cutting { .. })
    }

    /// 早送りセグメントかどうか判定
    pub fn is_rapid(&self) -> bool {
        matches!(self.segment_type, SegmentType::Rapid)
    }

    /// セグメント長さを計算
    ///
    /// # 戻り値
    ///
    /// 直線の場合は始点から終点までの直線距離、
    /// 円弧の場合は円弧長。
    pub fn length(&self) -> T {
        match &self.geometry {
            PathGeometry::Line { end } => {
                let dx = end.x() - self.start.x();
                let dy = end.y() - self.start.y();
                let dz = end.z() - self.start.z();
                (dx * dx + dy * dy + dz * dz).sqrt()
            }
            PathGeometry::Arc { end, center, .. } => {
                // 円弧長 = 半径 × 中心角
                let radius = {
                    let dx = self.start.x() - center.x();
                    let dy = self.start.y() - center.y();
                    let dz = self.start.z() - center.z();
                    (dx * dx + dy * dy + dz * dz).sqrt()
                };

                // 始点→中心、終点→中心のベクトル
                let v1_x = self.start.x() - center.x();
                let v1_y = self.start.y() - center.y();
                let v1_z = self.start.z() - center.z();

                let v2_x = end.x() - center.x();
                let v2_y = end.y() - center.y();
                let v2_z = end.z() - center.z();

                // 内積: cos(θ) = (v1 · v2) / (|v1| × |v2|)
                let dot = v1_x * v2_x + v1_y * v2_y + v1_z * v2_z;
                let cos_angle = dot / (radius * radius);

                // 中心角（ラジアン）
                let angle = cos_angle.acos();

                // 円弧長
                radius * angle
            }
        }
    }
}

/// 等高線レベル経路
///
/// 単一の等高線（Z座標が一定）における工具経路を表します。
///
/// # 例
///
/// ```
/// use cam_core::{ContourLevelPath, PathSegment, SegmentType};
/// use geo_primitives::Point3D;
///
/// let segments = vec![
///     PathSegment::new_line(
///         Point3D::new(0.0, 0.0, -5.0),
///         Point3D::new(10.0, 0.0, -5.0),
///         SegmentType::Rapid,
///     ),
/// ];
///
/// let contour = ContourLevelPath::new(0, -5.0, segments);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ContourLevelPath<T: Scalar = f64> {
    /// 等高線レベルインデックス（0から開始）
    pub level_index: usize,

    /// Z座標（高さ）
    pub z_level: T,

    /// このレベルのセグメント列
    pub segments: Vec<PathSegment<T>>,
}

impl<T: Scalar + std::iter::Sum> ContourLevelPath<T> {
    /// 新しい等高線経路を作成
    ///
    /// # 引数
    ///
    /// - `level_index`: 等高線レベルインデックス
    /// - `z_level`: Z座標
    /// - `segments`: セグメント列
    pub fn new(level_index: usize, z_level: T, segments: Vec<PathSegment<T>>) -> Self {
        Self {
            level_index,
            z_level,
            segments,
        }
    }

    /// 切削セグメントのみを抽出
    pub fn cutting_segments(&self) -> Vec<&PathSegment<T>> {
        self.segments.iter().filter(|s| s.is_cutting()).collect()
    }

    /// 早送りセグメントのみを抽出
    pub fn rapid_segments(&self) -> Vec<&PathSegment<T>> {
        self.segments.iter().filter(|s| s.is_rapid()).collect()
    }

    /// 総経路長を計算
    pub fn total_length(&self) -> T {
        self.segments.iter().map(|s| s.length()).sum()
    }

    /// 切削経路長のみを計算
    pub fn cutting_length(&self) -> T {
        self.segments
            .iter()
            .filter(|s| s.is_cutting())
            .map(|s| s.length())
            .sum()
    }
}

/// CAM工具経路全体
///
/// 1つの島または輪郭の完全な加工シーケンス。
/// Approach → Cutting → Retract の3フェーズ構造。
///
/// # 例
///
/// ```
/// use cam_core::{ToolPath, ContourLevelPath, CuttingDirection};
///
/// let toolpath = ToolPath::new(
///     "tool1".to_string(),
///     CuttingDirection::Down,
///     vec![],  // approach_segments
///     vec![ContourLevelPath::new(0, -5.0, vec![])],
///     vec![],  // retract_segments
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ToolPath<T: Scalar = f64> {
    /// 使用工具のID
    pub tool_id: String,

    /// この経路の切削方向（経路全体で統一）
    pub cutting_direction: CuttingDirection,

    /// アプローチフェーズ（ToolPath開始部、編集可能）
    pub approach_segments: Vec<PathSegment<T>>,

    /// 切削フェーズ（等高線ごと）
    pub contour_levels: Vec<ContourLevelPath<T>>,

    /// リトラクトフェーズ（ToolPath終了部、編集可能）
    pub retract_segments: Vec<PathSegment<T>>,
}

impl<T: Scalar + std::iter::Sum> ToolPath<T> {
    /// 新しい工具経路を作成
    ///
    /// # 引数
    ///
    /// - `tool_id`: 使用工具のID
    /// - `cutting_direction`: 切削方向
    /// - `approach_segments`: アプローチセグメント
    /// - `contour_levels`: 等高線経路のリスト
    /// - `retract_segments`: リトラクトセグメント
    pub fn new(
        tool_id: String,
        cutting_direction: CuttingDirection,
        approach_segments: Vec<PathSegment<T>>,
        contour_levels: Vec<ContourLevelPath<T>>,
        retract_segments: Vec<PathSegment<T>>,
    ) -> Self {
        Self {
            tool_id,
            cutting_direction,
            approach_segments,
            contour_levels,
            retract_segments,
        }
    }

    /// 総セグメント数を取得
    pub fn total_segments(&self) -> usize {
        self.contour_levels.iter().map(|c| c.segments.len()).sum()
    }

    /// 総経路長を計算
    pub fn total_length(&self) -> T {
        self.contour_levels.iter().map(|c| c.total_length()).sum()
    }

    /// 総切削経路長を計算
    pub fn total_cutting_length(&self) -> T {
        self.contour_levels.iter().map(|c| c.cutting_length()).sum()
    }

    /// 等高線数を取得
    pub fn level_count(&self) -> usize {
        self.contour_levels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_segment_creation() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            SegmentType::Cutting {
                feed_rate: 500.0,
            },
        );

        assert!(segment.is_cutting());
        assert!(!segment.is_rapid());
        assert_eq!(segment.length(), 10.0);
        assert_eq!(segment.end_point(), Point3D::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_arc_segment_creation() {
        // 半径5の90度円弧（反時計回り）
        let segment = PathSegment::new_arc(
            Point3D::new(5.0, 0.0, 0.0),
            Point3D::new(0.0, 5.0, 0.0),
            Point3D::new(0.0, 0.0, 0.0),
            ArcDirection::CounterClockwise,
            SegmentType::Rapid,
        );

        assert!(segment.is_rapid());
        assert_eq!(segment.end_point(), Point3D::new(0.0, 5.0, 0.0));

        // 円弧長 = 半径5 × π/2 ≈ 7.854
        let expected_length = 5.0 * std::f64::consts::PI / 2.0;
        assert!((segment.length() - expected_length).abs() < 0.001);
    }

    #[test]
    fn test_arc_segment_180_degrees() {
        // 半径10の180度円弧
        let segment = PathSegment::new_arc(
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(-10.0, 0.0, 0.0),
            Point3D::new(0.0, 0.0, 0.0),
            ArcDirection::Clockwise,
            SegmentType::Cutting {
                feed_rate: 500.0,
            },
        );

        // 円弧長 = 半径10 × π ≈ 31.416
        let expected_length = 10.0 * std::f64::consts::PI;
        assert!((segment.length() - expected_length).abs() < 0.001);
    }

    #[test]
    fn test_retract_and_pass_retract() {
        let retract = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(0.0, 0.0, 5.0),
            SegmentType::Retract { feed_rate: 300.0 },
        );
        assert_eq!(retract.length(), 5.0);

        let pass_retract = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            SegmentType::PassRetract { feed_rate: 200.0 },
        );
        assert_eq!(pass_retract.length(), 1.0);
    }

    #[test]
    fn test_contour_level_path() {
        let segments = vec![
            PathSegment::new_line(
                Point3D::new(0.0, 0.0, -5.0),
                Point3D::new(10.0, 0.0, -5.0),
                SegmentType::Cutting {
                    feed_rate: 500.0,
                },
            ),
            PathSegment::new_line(
                Point3D::new(10.0, 0.0, -5.0),
                Point3D::new(10.0, 10.0, -5.0),
                SegmentType::Rapid,
            ),
        ];

        let contour = ContourLevelPath::new(0, -5.0, segments);

        assert_eq!(contour.level_index, 0);
        assert_eq!(contour.z_level, -5.0);
        assert_eq!(contour.cutting_segments().len(), 1);
        assert_eq!(contour.rapid_segments().len(), 1);
        assert_eq!(contour.total_length(), 20.0);
        assert_eq!(contour.cutting_length(), 10.0);
    }

    #[test]
    fn test_tool_path() {
        let contours = vec![
            ContourLevelPath::new(0, -5.0, vec![]),
            ContourLevelPath::new(1, -10.0, vec![]),
        ];

        let toolpath = ToolPath::new(
            "tool1".to_string(),
            CuttingDirection::Down,
            vec![],
            contours,
            vec![],
        );

        assert_eq!(toolpath.tool_id, "tool1");
        assert_eq!(toolpath.cutting_direction, CuttingDirection::Down);
        assert_eq!(toolpath.level_count(), 2);
    }
}
