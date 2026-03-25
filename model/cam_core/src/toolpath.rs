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
//! use geo_algorithms::Point3D;
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

use crate::ExtAttribute;
use analysis::Scalar;
use geo_algorithms::{Point3D, Vector3D};

/// ToolPath wire schema version used by artifact binary v0.1.
pub const TOOLPATH_SCHEMA_VERSION_V0_1: (u16, u16) = (0, 1);

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

/// 機械軸種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineAxisKind {
    /// 直動軸（X/Y/Z/U/V/W など）
    Linear,
    /// 回転軸（A/B/C など）
    Rotary,
}

/// 任意名称の機械軸値
#[derive(Debug, Clone, PartialEq)]
pub struct MachineAxisValue<T: Scalar = f64> {
    /// 軸名（例: A, C, U, W）
    pub axis_name: String,
    /// 軸種別（直動/回転）
    pub axis_kind: MachineAxisKind,
    /// 軸値
    pub value: T,
}

impl<T: Scalar> MachineAxisValue<T> {
    /// 機械軸値を作成
    pub fn new(axis_name: String, axis_kind: MachineAxisKind, value: T) -> Self {
        Self {
            axis_name,
            axis_kind,
            value,
        }
    }
}

/// 姿勢補間ポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseInterpolationPolicy {
    /// セグメント内で姿勢固定（3+2 相当）
    FixedOrientation,
    /// 角度変化の最短経路を優先
    ShortestAngularPath,
    /// 連続変化を優先
    ContinuousPreferred,
    /// 機械制約を優先
    MachineConstrained,
}

/// 工具/プロセス姿勢
#[derive(Debug, Clone, PartialEq)]
pub struct ToolPose<T: Scalar = f64> {
    /// 姿勢評価時の位置
    pub position: Point3D<T>,
    /// 工具軸またはプロセス軸方向
    pub process_axis: Vector3D<T>,
    /// 任意軸名で管理される機械軸値
    pub machine_axes: Option<Vec<MachineAxisValue<T>>>,
}

impl<T: Scalar> ToolPose<T> {
    /// 姿勢を作成
    pub fn new(
        position: Point3D<T>,
        process_axis: Vector3D<T>,
        machine_axes: Option<Vec<MachineAxisValue<T>>>,
    ) -> Self {
        Self {
            position,
            process_axis,
            machine_axes,
        }
    }
}

/// 姿勢スパン
#[derive(Debug, Clone, PartialEq)]
pub struct ToolPoseSpan<T: Scalar = f64> {
    /// 始点姿勢
    pub start_pose: ToolPose<T>,
    /// 終点姿勢
    pub end_pose: ToolPose<T>,
    /// 姿勢補間ポリシー
    pub interpolation_policy: PoseInterpolationPolicy,
}

impl<T: Scalar> ToolPoseSpan<T> {
    /// 姿勢スパンを作成
    pub fn new(
        start_pose: ToolPose<T>,
        end_pose: ToolPose<T>,
        interpolation_policy: PoseInterpolationPolicy,
    ) -> Self {
        Self {
            start_pose,
            end_pose,
            interpolation_policy,
        }
    }
}

/// 姿勢付きセグメント
#[derive(Debug, Clone, PartialEq)]
pub struct PoseAnnotatedSegment<T: Scalar = f64> {
    /// 位置幾何セグメント
    pub segment: PathSegment<T>,
    /// セグメントに対応する姿勢情報
    pub pose_span: ToolPoseSpan<T>,
}

impl<T: Scalar> PoseAnnotatedSegment<T> {
    /// 姿勢付きセグメントを作成
    pub fn new(segment: PathSegment<T>, pose_span: ToolPoseSpan<T>) -> Self {
        Self { segment, pose_span }
    }
}

/// 機械構成の大分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineConfigurationClass {
    /// 2軸（XY平面など）
    TwoAxis,
    /// 3軸
    ThreeAxis,
    /// 4軸
    FourAxis,
    /// 5軸以上
    FiveAxisOrMore,
}

/// 加工時の運動モード分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KinematicMode {
    /// 純2軸（1平面内の輪郭加工）
    PureTwoAxis,
    /// 2.5軸（Z段付き2軸）
    TwoPointFiveAxis,
    /// 純3軸
    PureThreeAxis,
    /// 割り出し多軸（3+2）
    IndexedMultiAxis,
    /// 同時4軸
    ContinuousFourAxis,
    /// 同時5軸
    ContinuousFiveAxis,
}

/// 姿勢データ保持ポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseDataPolicy {
    /// 位置中心データだけで互換運用可能
    PositionOnlyCompatible,
    /// 姿勢レイヤーは任意
    PoseLayerOptional,
    /// 姿勢レイヤーが必須
    PoseLayerRequired,
}

/// ToolPath の運動学メタ情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolPathKinematicMeta {
    /// 機械構成の大分類
    pub configuration_class: MachineConfigurationClass,
    /// 加工時の運動モード
    pub kinematic_mode: KinematicMode,
    /// 姿勢データ保持ポリシー
    pub pose_data_policy: PoseDataPolicy,
}

impl ToolPathKinematicMeta {
    /// 運動学メタ情報を作成
    pub const fn new(
        configuration_class: MachineConfigurationClass,
        kinematic_mode: KinematicMode,
        pose_data_policy: PoseDataPolicy,
    ) -> Self {
        Self {
            configuration_class,
            kinematic_mode,
            pose_data_policy,
        }
    }

    /// 既存3軸互換向けの既定メタ
    pub const fn three_axis_position_only() -> Self {
        Self {
            configuration_class: MachineConfigurationClass::ThreeAxis,
            kinematic_mode: KinematicMode::PureThreeAxis,
            pose_data_policy: PoseDataPolicy::PositionOnlyCompatible,
        }
    }

    /// 2軸（平面輪郭加工）向けの既定メタ
    pub const fn two_axis_position_only() -> Self {
        Self {
            configuration_class: MachineConfigurationClass::TwoAxis,
            kinematic_mode: KinematicMode::PureTwoAxis,
            pose_data_policy: PoseDataPolicy::PositionOnlyCompatible,
        }
    }

    /// 2.5軸（Z段付き輪郭加工）向けの既定メタ
    pub const fn two_point_five_axis_position_only() -> Self {
        Self {
            configuration_class: MachineConfigurationClass::TwoAxis,
            kinematic_mode: KinematicMode::TwoPointFiveAxis,
            pose_data_policy: PoseDataPolicy::PositionOnlyCompatible,
        }
    }

    /// 位置中心データのみで互換運用可能か
    pub const fn is_position_only_compatible(&self) -> bool {
        matches!(
            self.pose_data_policy,
            PoseDataPolicy::PositionOnlyCompatible
        )
    }

    /// 姿勢レイヤーを省略可能か
    pub const fn allows_optional_pose_layer(&self) -> bool {
        !self.requires_pose_layer()
    }

    /// 姿勢レイヤーが必須か
    pub const fn requires_pose_layer(&self) -> bool {
        matches!(self.pose_data_policy, PoseDataPolicy::PoseLayerRequired)
    }
}

/// 工具経路セグメント
///
/// 工具経路の最小単位。始点、幾何形状、およびセグメント種別を持ちます。
/// 拡張属性により、ユーザースクリプトが独自情報を追加できます。
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment<T: Scalar = f64> {
    /// セグメント始点（絶対座標）
    pub start: Point3D<T>,

    /// 幾何形状（直線または円弧）
    pub geometry: PathGeometry<T>,

    /// セグメント種別
    pub segment_type: SegmentType<T>,

    /// セグメント拡張属性（ユーザーが付与可能）
    pub ext_attributes: Vec<ExtAttribute>,
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
    /// use geo_algorithms::Point3D;
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
            ext_attributes: Vec::new(),
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
    /// use geo_algorithms::Point3D;
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
            ext_attributes: Vec::new(),
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
/// use geo_algorithms::Point3D;
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
/// 拡張属性により、ユーザースクリプトが独自情報を追加できます。
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

    /// ToolPath全体の拡張属性（ユーザーが付与可能）
    pub ext_attributes: Vec<ExtAttribute>,

    /// ToolPath全体の運動学メタ情報
    pub kinematic_meta: ToolPathKinematicMeta,
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
        Self::new_with_kinematic_meta(
            tool_id,
            cutting_direction,
            approach_segments,
            contour_levels,
            retract_segments,
            ToolPathKinematicMeta::three_axis_position_only(),
        )
    }

    /// 運動学メタを明示して新しい工具経路を作成
    pub fn new_with_kinematic_meta(
        tool_id: String,
        cutting_direction: CuttingDirection,
        approach_segments: Vec<PathSegment<T>>,
        contour_levels: Vec<ContourLevelPath<T>>,
        retract_segments: Vec<PathSegment<T>>,
        kinematic_meta: ToolPathKinematicMeta,
    ) -> Self {
        Self {
            tool_id,
            cutting_direction,
            approach_segments,
            contour_levels,
            retract_segments,
            ext_attributes: Vec::new(),
            kinematic_meta,
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
#[path = "toolpath_tests.rs"]
mod tests;
