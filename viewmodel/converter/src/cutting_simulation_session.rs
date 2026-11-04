//! 製品形状からの切削シミュレーションの投入・実行・取り込み
//!
//! 加工手順 1 つと加工工程 1 つを Application の保管先に持ち、投入・取り込みはその工程から行う。加工条件の入力画面は
//! 計算条件（`CuttingSimulationConditions`）を 1 組編集し、セッションはそれを工程の中身と手順の素材に書き込む。
//! 加工の対象は、確かめ・投入のときに表示中の製品形状のすべての面にする。
//! 実行の入口は、ジョブマネージャーができるまでの同じプロセスの中の実行とする。

use application::cam_simulation_workflow::{
    CamConditionError, CamSimulationWorkflow, CamSimulationWorkflowError,
};
use application::machining_sequence::{
    MachiningOperationContent, MachiningOperationId, MachiningSequenceId, MachiningSequenceStore,
    MachiningStock,
};
use cam_algorithms::CamSolverError;
pub use cam_algorithms::ScanlineInputError;
use cam_core::{MachiningStage, OperationSpec, ToleranceProfile, Tool};
pub use cam_core::{ScanlineCutDirection, ScanlineParams, ScanlineToolRange, StockMargins};
use cam_sim::CuttingSimulationInputError;
use geo_algorithms::Aabb3D;

use crate::cutting_animation::{CuttingAnimation, CuttingAnimationError};
use crate::inverse_offset_shape_converter::{inverse_offset_shape_meshes, InverseOffsetShapeMesh};
use crate::product_shape_converter::ProductShapeScene;

/// ツールパス生成のトレランスを決める精度プロファイル
const TOLERANCE_PROFILE: ToleranceProfile = ToleranceProfile::PressRough;

/// 加工手順・加工工程の名前（加工条件のウィンドウで編集できるようになるまでの固定の名前）
const SEQUENCE_NAME: &str = "加工手順";
const OPERATION_NAME: &str = "スキャン加工";

/// 工具の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CuttingToolKind {
    #[default]
    BallEndMill,
    FlatEndMill,
}

/// 切削シミュレーションの計算条件（工具・加工・素材）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CuttingSimulationConditions {
    /// 工具の種類
    pub tool_kind: CuttingToolKind,
    /// 工具の直径（mm）
    pub tool_diameter: f64,
    /// 工具長（mm。`None` は長さに制限のない工具）
    pub cutting_length: Option<f64>,
    /// 走査線の条件
    pub scanline: ScanlineParams,
    /// 製品形状を囲む箱から素材の箱の各面までの余裕（mm）
    pub stock_margins: StockMargins,
    /// 素材の形状の許容誤差（mm。トリデクセルの格子の間隔を決める）
    pub precision: f64,
}

impl Default for CuttingSimulationConditions {
    fn default() -> Self {
        Self {
            tool_kind: CuttingToolKind::default(),
            tool_diameter: 10.0,
            cutting_length: None,
            scanline: ScanlineParams {
                stepover: 2.0,
                feed_rate: 1200.0,
                tool_range: ScanlineToolRange::default(),
                cut_direction: ScanlineCutDirection::default(),
            },
            stock_margins: StockMargins::default(),
            precision: 0.05,
        }
    }
}

impl CuttingSimulationConditions {
    /// 平らな面で、隣り合うパスの間がピックフィードのときに残る山の高さ（mm。ピックフィードが工具径を超える
    /// 場合は `None`）
    ///
    /// ボールエンドミルは r − √(r² − (s/2)²)、フラットエンドミルは底が平らなので 0 とする。パスの間はステップ
    /// オーバー以下になるため、平らな面での上限の値になる。傾いた面では山の高さが変わるため、目安の値とする。
    pub fn cusp_height(&self) -> Option<f64> {
        let stepover = self.scanline.stepover;
        if stepover > self.tool_diameter {
            return None;
        }
        match self.tool_kind {
            CuttingToolKind::BallEndMill => {
                let radius = 0.5 * self.tool_diameter;
                let half = 0.5 * stepover;
                Some(radius - (radius * radius - half * half).sqrt())
            }
            CuttingToolKind::FlatEndMill => Some(0.0),
        }
    }

    fn tool(&self) -> Tool<f64> {
        // 工具長は、作った後に条件の値（指定しない場合は `None`）にする
        let mut tool = match self.tool_kind {
            CuttingToolKind::BallEndMill => Tool::ball_end_mill(
                format!("ball_endmill_{}mm", self.tool_diameter),
                self.tool_diameter,
                0.0,
            ),
            CuttingToolKind::FlatEndMill => Tool::flat_end_mill(
                format!("flat_endmill_{}mm", self.tool_diameter),
                self.tool_diameter,
                0.0,
            ),
        };
        tool.cutting_length = self.cutting_length;
        tool
    }

    /// 加工工程の中身（工具・走査線の条件。仕上げのスキャン加工）
    fn operation_content(&self) -> MachiningOperationContent {
        MachiningOperationContent {
            name: OPERATION_NAME.to_string(),
            stage: MachiningStage::Finish,
            spec: OperationSpec::Scanline(self.scanline),
            tool: self.tool(),
            boundary: None,
            tolerance_profile: TOLERANCE_PROFILE,
        }
    }

    /// 加工手順の素材（素材の余裕・精度）
    fn stock(&self) -> MachiningStock {
        MachiningStock {
            margins: self.stock_margins,
            precision: self.precision,
        }
    }
}

/// 取り込んだ切削シミュレーションの計算に使った条件
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedConditions {
    /// 工具
    pub tool: Tool<f64>,
    /// 走査線の条件
    pub scanline: ScanlineParams,
    /// ツールパス生成のトレランス（mm）
    pub toolpath_tolerance: f64,
    /// 素材（直方体）
    pub stock: Aabb3D<f64>,
    /// 素材の形状の許容誤差（mm）
    pub precision: f64,
}

/// 取り込んだ切削シミュレーション
#[derive(Debug)]
pub struct ImportedCuttingSimulation {
    /// 切削シミュレーションのジョブの番号
    pub job_id: u64,
    /// 切削アニメーション
    pub animation: CuttingAnimation,
    /// 計算に使った条件
    pub conditions: ImportedConditions,
}

/// 計算条件の確かめで見つかった問題（入力画面の欄に振り分けられる単位）
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionIssue {
    /// 加工の対象の製品形状がない、または形状にできない（理由）
    NoProductShape(String),
    /// 素材の余裕が負または有限でない
    InvalidStockMargins,
    /// 素材が体積を持つ直方体にならない
    InvalidStock,
    /// 工具の値が不正
    InvalidTool,
    /// 走査線の条件の誤り（確かめた条件の工具径。mm）
    InvalidScanline {
        error: ScanlineInputError,
        tool_diameter: f64,
    },
    /// そのほかのツールパス生成の入力の誤り（理由）
    InvalidToolpathInput(String),
    /// 精度が正の有限値でない
    InvalidPrecision,
    /// 精度がツールパス生成のトレランス（mm）より細かい
    PrecisionFinerThanTolerance { tolerance: f64 },
    /// 精度が細かすぎて格子の線の本数の上限を超える（上限に収まる最も細かい精度。mm）
    TooManyDexelLines { finest_precision: f64 },
    /// そのほかの理由で投入できない（理由）
    Other(String),
}

impl ConditionIssue {
    fn from_condition_error(error: &CamConditionError, tool_diameter: f64) -> Self {
        match error {
            CamConditionError::InvalidStockMargins(_) => Self::InvalidStockMargins,
            CamConditionError::PrecisionFinerThanTolerance { tolerance, .. } => {
                Self::PrecisionFinerThanTolerance {
                    tolerance: *tolerance,
                }
            }
            CamConditionError::ToolpathInput(CamSolverError::InvalidScanline(error)) => {
                Self::InvalidScanline {
                    error: *error,
                    tool_diameter,
                }
            }
            CamConditionError::ToolpathInput(error) => {
                Self::InvalidToolpathInput(error.reason().to_string())
            }
            CamConditionError::SimulationInput(error) => match error {
                CuttingSimulationInputError::Stock => Self::InvalidStock,
                CuttingSimulationInputError::Tool => Self::InvalidTool,
                CuttingSimulationInputError::Precision => Self::InvalidPrecision,
                CuttingSimulationInputError::TooManyDexelLines {
                    finest_precision, ..
                } => Self::TooManyDexelLines {
                    finest_precision: *finest_precision,
                },
                CuttingSimulationInputError::SnapshotInterval => Self::Other(error.to_string()),
            },
        }
    }

    fn from_workflow_error(error: &CamSimulationWorkflowError, tool_diameter: f64) -> Vec<Self> {
        match error {
            CamSimulationWorkflowError::Conditions(errors) => errors
                .iter()
                .map(|error| Self::from_condition_error(error, tool_diameter))
                .collect(),
            CamSimulationWorkflowError::ProductShape(_)
            | CamSimulationWorkflowError::EmptyProductShape => {
                vec![Self::NoProductShape(error.to_string())]
            }
            other => vec![Self::Other(other.to_string())],
        }
    }
}

/// 投入・実行・取り込みの失敗
#[derive(Debug)]
pub enum CuttingSimulationSessionError {
    /// 投入・実行・取り込みの失敗
    Workflow(CamSimulationWorkflowError),
    /// 加工工程の今の改訂の、取り込める（正常終了した）切削シミュレーションがない
    NoSucceededSimulation,
    /// 取り込んだ結果から切削アニメーションを作れない
    Animation(CuttingAnimationError),
    /// 逆オフセット形状を表示用に分けられない（表示のきざみの誤り）
    InverseOffsetDisplay(CamSolverError),
}

impl std::fmt::Display for CuttingSimulationSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workflow(error) => write!(f, "{error}"),
            Self::NoSucceededSimulation => write!(
                f,
                "no succeeded cutting simulation of the current conditions to import"
            ),
            Self::Animation(error) => write!(f, "{error}"),
            Self::InverseOffsetDisplay(error) => {
                write!(f, "inverse offset shape cannot be displayed: {error}")
            }
        }
    }
}

impl std::error::Error for CuttingSimulationSessionError {}

impl From<CamSimulationWorkflowError> for CuttingSimulationSessionError {
    fn from(value: CamSimulationWorkflowError) -> Self {
        Self::Workflow(value)
    }
}

/// 製品形状からの切削シミュレーションの投入・実行・取り込み
#[derive(Debug)]
pub struct CuttingSimulationSession {
    workflow: CamSimulationWorkflow,
    conditions: CuttingSimulationConditions,
    sequences: MachiningSequenceStore,
    sequence: MachiningSequenceId,
    operation: MachiningOperationId,
}

impl Default for CuttingSimulationSession {
    fn default() -> Self {
        Self::new(CuttingSimulationConditions::default())
    }
}

impl CuttingSimulationSession {
    /// 計算条件 `conditions` の加工工程を 1 つ持つ加工手順で始める（加工の対象は、確かめ・投入のときに決める）
    pub fn new(conditions: CuttingSimulationConditions) -> Self {
        let mut sequences = MachiningSequenceStore::new();
        let sequence =
            sequences.create_sequence(SEQUENCE_NAME.to_string(), Vec::new(), conditions.stock());
        let operation = sequences
            .add_operation(sequence, conditions.operation_content())
            .expect("the sequence was just created");
        Self {
            workflow: CamSimulationWorkflow::new(),
            conditions,
            sequences,
            sequence,
            operation,
        }
    }

    /// 次に投入する計算条件
    pub fn conditions(&self) -> CuttingSimulationConditions {
        self.conditions
    }

    /// 次に投入する計算条件を `conditions` にし、加工工程の中身と加工手順の素材に書き込む（変わった場合は、工程の
    /// 改訂の番号が増える）
    pub fn set_conditions(&mut self, conditions: CuttingSimulationConditions) {
        self.conditions = conditions;
        self.sequences
            .set_operation_content(
                self.sequence,
                self.operation,
                conditions.operation_content(),
            )
            .expect("the session holds its machining operation");
        self.sequences
            .set_stock(self.sequence, conditions.stock())
            .expect("the session holds its machining sequence");
    }

    /// ツールパス生成のトレランス（mm）
    pub fn toolpath_tolerance(&self) -> f64 {
        TOLERANCE_PROFILE.tolerance_mm()
    }

    /// 加工の対象を、表示中の製品形状のすべての面にする（面の集合が変わった場合は、工程の改訂の番号が増える）
    fn target_visible_faces(&mut self, shapes: &ProductShapeScene) {
        self.sequences
            .set_target_faces(self.sequence, shapes.store().visible_face_ids())
            .expect("the session holds its machining sequence");
    }

    /// 表示中の製品形状と次に投入する計算条件を、投入と同じ判定で確かめ、問題をすべて返す（問題がなければ空）
    pub fn check(&mut self, shapes: &ProductShapeScene) -> Vec<ConditionIssue> {
        self.target_visible_faces(shapes);
        match CamSimulationWorkflow::check_operation(
            shapes.store(),
            &self.sequences,
            self.sequence,
            self.operation,
        ) {
            Ok(()) => Vec::new(),
            Err(error) => {
                ConditionIssue::from_workflow_error(&error, self.conditions.tool_diameter)
            }
        }
    }

    /// 表示中の製品形状と次に投入する計算条件で、投入したときのツールパス生成と同じ逆オフセット形状を作り、角度
    /// `angle_step`（ラジアン）のきざみで種類ごとのメッシュにする（デバッグ表示）
    ///
    /// # Errors
    ///
    /// 投入前の確認の誤り、逆オフセット形状を作れない場合、`angle_step` が表示用のきざみの範囲の外の場合。
    pub fn inverse_offset_shape_meshes(
        &mut self,
        shapes: &ProductShapeScene,
        angle_step: f64,
    ) -> Result<Vec<InverseOffsetShapeMesh>, CuttingSimulationSessionError> {
        self.target_visible_faces(shapes);
        let shape = CamSimulationWorkflow::inverse_offset_shape(
            shapes.store(),
            &self.sequences,
            self.sequence,
            self.operation,
        )?;
        inverse_offset_shape_meshes(&shape, angle_step)
            .map_err(CuttingSimulationSessionError::InverseOffsetDisplay)
    }

    /// 表示中の製品形状のすべての面を加工の対象として、次に投入する計算条件の加工工程から投入する（実行しない）
    ///
    /// # Errors
    ///
    /// 表示中の製品形状がない場合・計算条件に誤りがある場合を含む、投入の失敗。
    pub fn submit(
        &mut self,
        shapes: &ProductShapeScene,
    ) -> Result<(), CuttingSimulationSessionError> {
        self.target_visible_faces(shapes);
        self.workflow.submit_operation(
            shapes.store(),
            &self.sequences,
            self.sequence,
            self.operation,
        )?;
        Ok(())
    }

    /// キューにあるジョブを実行し、実行したジョブの数を返す（ジョブマネージャーの実行の入口）
    ///
    /// # Errors
    ///
    /// ジョブの状態の更新の失敗。
    pub fn run_queued_jobs(&mut self) -> Result<usize, CuttingSimulationSessionError> {
        Ok(self.workflow.run_queued_jobs()?)
    }

    /// 加工工程の今の改訂（今の計算条件と加工の対象）で正常終了した切削シミュレーションのうち、最も新しいものを
    /// 取り込み、切削アニメーションにする
    ///
    /// # Errors
    ///
    /// 今の改訂の取り込める切削シミュレーションがない場合、取り込みの失敗、切削アニメーションを作れない場合。
    pub fn import_latest(
        &self,
    ) -> Result<ImportedCuttingSimulation, CuttingSimulationSessionError> {
        let latest = self
            .workflow
            .latest_revision_simulation(&self.sequences, self.sequence, self.operation)?
            .map(|job| job.id)
            .ok_or(CuttingSimulationSessionError::NoSucceededSimulation)?;
        let result = self.workflow.import(latest)?;
        let cam_core::OperationSpec::Scanline(scanline) = result.solver_input.operation;
        let conditions = ImportedConditions {
            tool: result.simulation_input.tool.clone(),
            scanline,
            toolpath_tolerance: result.solver_input.chord_tolerance,
            stock: result.simulation_input.stock,
            precision: result.simulation_input.precision,
        };
        let animation = CuttingAnimation::new(
            &result.toolpath_artifact,
            &result.simulation_artifact,
            result.simulation_input.tool,
            result.simulation_input.precision,
        )
        .map_err(CuttingSimulationSessionError::Animation)?;
        Ok(ImportedCuttingSimulation {
            job_id: latest,
            animation,
            conditions,
        })
    }
}

#[cfg(test)]
mod tests {
    use application::cam_simulation_workflow::CamSimulationWorkflowError;
    use application::product_shape::ProductShapeError;
    use geo_algorithms::{Point3D, TriangleMesh3D};
    use geo_contracts::TriangleMesh3DConstructor;

    use super::*;

    /// 粗い条件（テストを速くする）
    fn coarse() -> CuttingSimulationConditions {
        CuttingSimulationConditions {
            tool_diameter: 6.0,
            cutting_length: Some(30.0),
            scanline: ScanlineParams {
                stepover: 6.0,
                feed_rate: 1200.0,
                tool_range: ScanlineToolRange::default(),
                cut_direction: ScanlineCutDirection::default(),
            },
            ..CuttingSimulationConditions::default()
        }
    }

    fn scene_with_pyramid() -> ProductShapeScene {
        let pyramid = TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(20.0, 0.0, 0.0),
                Point3D::new(20.0, 20.0, 0.0),
                Point3D::new(0.0, 20.0, 0.0),
                Point3D::new(10.0, 10.0, 6.0),
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
        .unwrap();
        // 並べて走るテストどうしで、一時ファイルを分ける
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "redring_cutting_simulation_session_{}_{}.stl",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        geo_io::stl::save_stl(&pyramid, &path).unwrap();
        let mut scene = ProductShapeScene::default();
        scene.import_stl(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        scene
    }

    #[test]
    fn cusp_heights_on_flat_surfaces_follow_the_tool_and_the_stepover() {
        let with = |tool_kind, stepover| {
            let mut conditions = CuttingSimulationConditions {
                tool_kind,
                tool_diameter: 10.0,
                ..CuttingSimulationConditions::default()
            };
            conditions.scanline.stepover = stepover;
            conditions.cusp_height()
        };
        let ball = CuttingToolKind::BallEndMill;
        let flat = CuttingToolKind::FlatEndMill;

        // r = 5 mm、s = 2 mm: 5 − √24
        assert!((with(ball, 2.0).unwrap() - (5.0 - 24.0_f64.sqrt())).abs() < 1e-12);
        // ピックフィードが工具径と同じなら、山の高さは工具の半径
        assert_eq!(with(ball, 10.0), Some(5.0));
        assert_eq!(with(flat, 2.0), Some(0.0));
        // ピックフィードが工具径を超える場合は、値を持たない
        assert_eq!(with(ball, 10.5), None);
        assert_eq!(with(flat, 10.5), None);
    }

    #[test]
    fn submitted_product_shapes_are_imported_as_a_cutting_animation_after_running() {
        let scene = scene_with_pyramid();
        let mut session = CuttingSimulationSession::new(coarse());
        assert_eq!(session.check(&scene), vec![]);
        session.submit(&scene).unwrap();
        // 実行するまでは取り込めない
        assert!(matches!(
            session.import_latest(),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        let mut imported = session.import_latest().unwrap();
        assert_eq!(imported.job_id, 2);
        assert!(imported.animation.frame_count() > 1);
        assert!(!imported.animation.take_initial_stock_blocks().is_empty());
    }

    #[test]
    fn results_of_previous_conditions_are_not_imported_until_submitted_again() {
        let scene = scene_with_pyramid();
        let mut session = CuttingSimulationSession::new(coarse());
        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();
        assert_eq!(session.import_latest().unwrap().job_id, 2);

        // 条件を変えると、前の条件の結果は取り込まない（再計算待ち）
        session.set_conditions(CuttingSimulationConditions {
            precision: 0.2,
            ..session.conditions()
        });
        assert!(matches!(
            session.import_latest(),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));
        // 同じ条件を設定し直しても、改訂の番号は戻らない
        session.set_conditions(session.conditions());
        assert!(session.import_latest().is_err());

        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();
        let imported = session.import_latest().unwrap();
        assert_eq!(imported.job_id, 4);
        assert_eq!(imported.conditions.precision, 0.2);
    }

    #[test]
    fn the_set_conditions_are_submitted_and_returned_with_the_imported_result() {
        let scene = scene_with_pyramid();
        let mut session = CuttingSimulationSession::new(coarse());
        let conditions = CuttingSimulationConditions {
            tool_kind: CuttingToolKind::FlatEndMill,
            cutting_length: Some(25.0),
            stock_margins: StockMargins {
                top: 2.0,
                ..StockMargins::default()
            },
            precision: 0.2,
            ..session.conditions()
        };
        let conditions = CuttingSimulationConditions {
            scanline: ScanlineParams {
                cut_direction: ScanlineCutDirection::ZigZag,
                ..conditions.scanline
            },
            ..conditions
        };
        session.set_conditions(conditions);
        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();

        let imported = session.import_latest().unwrap().conditions;
        assert!(imported.tool.is_flat_end_mill());
        assert_eq!(imported.tool.diameter(), conditions.tool_diameter);
        assert_eq!(imported.tool.cutting_length, conditions.cutting_length);
        assert_eq!(imported.scanline, conditions.scanline);
        assert_eq!(imported.toolpath_tolerance, session.toolpath_tolerance());
        assert_eq!(imported.precision, 0.2);
        // 四角錐（0〜20, 0〜20, 0〜6）を囲む箱の上に 2 を足した素材
        assert_eq!(imported.stock.max().z(), 8.0);
        assert_eq!(imported.stock.min().x(), 0.0);
    }

    #[test]
    fn conditions_with_errors_are_reported_by_field_and_not_submitted() {
        let scene = scene_with_pyramid();
        let mut session = CuttingSimulationSession::new(coarse());
        session.set_conditions(CuttingSimulationConditions {
            stock_margins: StockMargins {
                bottom: -1.0,
                ..StockMargins::default()
            },
            precision: 0.0005,
            // ピックフィードが工具径 6 mm を超える
            scanline: ScanlineParams {
                stepover: 7.0,
                ..session.conditions().scanline
            },
            ..session.conditions()
        });
        let issues = session.check(&scene);
        assert_eq!(
            issues,
            vec![
                ConditionIssue::InvalidStockMargins,
                ConditionIssue::PrecisionFinerThanTolerance {
                    tolerance: session.toolpath_tolerance()
                },
                ConditionIssue::InvalidScanline {
                    error: ScanlineInputError::StepoverExceedsToolDiameter,
                    tool_diameter: 6.0,
                },
            ]
        );
        assert!(session.submit(&scene).is_err());
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn nothing_is_submitted_without_visible_product_shapes() {
        let mut session = CuttingSimulationSession::new(coarse());
        assert!(matches!(
            session.check(&ProductShapeScene::default())[..],
            [ConditionIssue::NoProductShape(_)]
        ));
        let error = session.submit(&ProductShapeScene::default()).unwrap_err();
        assert!(matches!(
            error,
            CuttingSimulationSessionError::Workflow(CamSimulationWorkflowError::ProductShape(
                ProductShapeError::NoFaces
            ))
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }
}
