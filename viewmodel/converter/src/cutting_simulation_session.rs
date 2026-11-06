//! 製品形状からの切削シミュレーションの投入・実行・取り込み
//!
//! 加工手順 1 つと加工工程 1 つを Application の保管先に持ち、投入・取り込みはその工程から行う。条件の写しは持たず、
//! 加工工程の中身（工具・走査線の条件）と加工手順の素材（素材の余裕・精度）を保管先から読み、書き戻す。
//! 加工の対象は、確かめ・投入・取り込みのときに表示中の製品形状のすべての面にする。
//! 実行の入口は、ジョブマネージャーができるまでの同じプロセスの中の実行とする。

use application::cam_simulation_workflow::{
    Aabb3D, CamSimulationWorkflow, CamSimulationWorkflowError,
};
pub use application::cam_simulation_workflow::{CamConditionError, CamConditionField};
pub use application::machining_sequence::{
    flat_surface_cusp_height, MachiningOperationContent, MachiningStock, OperationSpec,
    ScanlineCutDirection, ScanlineToolRange, StockMargins,
};
use application::machining_sequence::{
    scanline_finish_operation_content, MachiningOperationId, MachiningSequenceId,
    MachiningSequenceStore,
};
use application::machining_sequence::{ScanlineParams, Tool};

use application::cutting_simulation_playback::{
    CuttingSimulationPlayback, CuttingSimulationPlaybackError,
};

use crate::cutting_animation::CuttingAnimation;
use crate::failure_message_mapper::workflow_failure_message;
use crate::inverse_offset_shape_converter::{inverse_offset_shape_meshes, InverseOffsetShapeMesh};
use crate::product_shape_converter::ProductShapeScene;
use i18n_foundation::UiMessage;

/// 加工手順・加工工程の名前（名前を編集できるようになるまでの固定の名前）
const SEQUENCE_NAME: &str = "加工手順";
const OPERATION_NAME: &str = "スキャン加工";

/// 選べる工具の種類（ボールエンドミル・フラットエンドミル）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CuttingToolKind {
    #[default]
    BallEndMill,
    FlatEndMill,
}

impl CuttingToolKind {
    /// 工具 `tool` の種類（ボールエンドミル・フラットエンドミルでない場合は `None`）
    pub fn of(tool: &Tool<f64>) -> Option<Self> {
        if tool.is_ball_end_mill() {
            Some(Self::BallEndMill)
        } else if tool.is_flat_end_mill() {
            Some(Self::FlatEndMill)
        } else {
            None
        }
    }

    /// この種類の、直径 `diameter`（mm）・工具長 `cutting_length`（mm。`None` は長さに制限のない工具）の工具
    pub fn tool(self, diameter: f64, cutting_length: Option<f64>) -> Tool<f64> {
        // 工具長は、作った後に入力の値（指定しない場合は `None`）にする
        let mut tool = match self {
            Self::BallEndMill => {
                Tool::ball_end_mill(format!("ball_endmill_{diameter}mm"), diameter, 0.0)
            }
            Self::FlatEndMill => {
                Tool::flat_end_mill(format!("flat_endmill_{diameter}mm"), diameter, 0.0)
            }
        };
        tool.cutting_length = cutting_length;
        tool
    }
}

/// セッションを始めたときの加工工程の中身（直径 10 mm のボールエンドミル、ピックフィード 2 mm）
fn initial_operation_content() -> MachiningOperationContent {
    scanline_finish_operation_content(
        OPERATION_NAME.to_string(),
        CuttingToolKind::BallEndMill.tool(10.0, None),
        ScanlineParams {
            stepover: 2.0,
            feed_rate: 1200.0,
            tool_range: ScanlineToolRange::default(),
            cut_direction: ScanlineCutDirection::default(),
        },
    )
}

/// セッションを始めたときの加工手順の素材（製品形状を囲む箱、精度 0.05 mm）
fn initial_stock() -> MachiningStock {
    MachiningStock {
        margins: StockMargins::default(),
        precision: 0.05,
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

/// 計算条件の確かめで見つかった問題（製品形状がない・条件の誤り・そのほか）
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionIssue {
    /// 加工の対象の製品形状がない、または形状にできない
    NoProductShape,
    /// 計算条件の誤り（出す欄は `CamConditionError::field`）
    Condition(CamConditionError),
    /// そのほかの理由で投入できない（画面に出す理由）
    Other(UiMessage),
}

impl ConditionIssue {
    fn from_workflow_error(error: &CamSimulationWorkflowError) -> Vec<Self> {
        match error {
            CamSimulationWorkflowError::Conditions(errors) => {
                errors.iter().cloned().map(Self::Condition).collect()
            }
            CamSimulationWorkflowError::ProductShape(_)
            | CamSimulationWorkflowError::EmptyProductShape => {
                vec![Self::NoProductShape]
            }
            other => vec![Self::Other(workflow_failure_message(other))],
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
    /// 取り込んだ結果を再生できない
    Playback(CuttingSimulationPlaybackError),
}

impl std::fmt::Display for CuttingSimulationSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workflow(error) => write!(f, "{error}"),
            Self::NoSucceededSimulation => write!(
                f,
                "no succeeded cutting simulation of the current conditions to import"
            ),
            Self::Playback(error) => write!(f, "{error}"),
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
    sequences: MachiningSequenceStore,
    sequence: MachiningSequenceId,
    operation: MachiningOperationId,
}

impl Default for CuttingSimulationSession {
    /// 加工工程を 1 つ持つ加工手順で始める（加工の対象は、確かめ・投入・取り込みのときに決める）
    fn default() -> Self {
        let mut sequences = MachiningSequenceStore::new();
        let sequence =
            sequences.create_sequence(SEQUENCE_NAME.to_string(), Vec::new(), initial_stock());
        let operation = sequences
            .add_operation(sequence, initial_operation_content())
            .expect("the sequence was just created");
        Self {
            workflow: CamSimulationWorkflow::new(),
            sequences,
            sequence,
            operation,
        }
    }
}

impl CuttingSimulationSession {
    /// 加工工程の中身（工具・走査線の条件）
    pub fn operation_content(&self) -> &MachiningOperationContent {
        self.sequences
            .operation_content(self.sequence, self.operation)
            .expect("the session holds its machining operation")
    }

    /// 加工工程の中身を `content` にする（今と違えば、工程の改訂の番号が増える）
    pub fn set_operation_content(&mut self, content: MachiningOperationContent) {
        self.sequences
            .set_operation_content(self.sequence, self.operation, content)
            .expect("the session holds its machining operation");
    }

    /// 加工手順の素材（素材の余裕・精度）
    pub fn stock(&self) -> MachiningStock {
        *self
            .sequences
            .stock(self.sequence)
            .expect("the session holds its machining sequence")
    }

    /// 加工手順の素材を `stock` にする（今と違えば、工程の改訂の番号が増える）
    pub fn set_stock(&mut self, stock: MachiningStock) {
        self.sequences
            .set_stock(self.sequence, stock)
            .expect("the session holds its machining sequence");
    }

    /// 加工工程のツールパス生成のトレランス（mm）
    pub fn toolpath_tolerance(&self) -> f64 {
        self.sequences
            .operation_tolerance(self.sequence, self.operation)
            .expect("the session holds its machining operation")
    }

    /// 加工の対象を、表示中の製品形状のすべての面にする（面の集合が変わった場合は、工程の改訂の番号が増える）
    fn target_visible_faces(&mut self, shapes: &ProductShapeScene) {
        self.sequences
            .set_target_faces(self.sequence, shapes.store().visible_face_ids())
            .expect("the session holds its machining sequence");
    }

    /// 表示中の製品形状と加工工程・加工手順の条件を、投入と同じ判定で確かめ、問題をすべて返す（問題がなければ空）
    pub fn check(&mut self, shapes: &ProductShapeScene) -> Vec<ConditionIssue> {
        self.target_visible_faces(shapes);
        match CamSimulationWorkflow::check_operation(
            shapes.store(),
            &self.sequences,
            self.sequence,
            self.operation,
        ) {
            Ok(()) => Vec::new(),
            Err(error) => ConditionIssue::from_workflow_error(&error),
        }
    }

    /// 表示中の製品形状と加工工程・加工手順の条件で、投入したときのツールパス生成と同じ逆オフセット形状を作り、角度
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
        let parts = CamSimulationWorkflow::inverse_offset_shape_parts(
            shapes.store(),
            &self.sequences,
            self.sequence,
            self.operation,
            angle_step,
        )?;
        Ok(inverse_offset_shape_meshes(parts))
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程から投入する（実行しない）
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

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程の今の改訂（今の計算条件と加工の対象）で正常終了した
    /// 切削シミュレーションのうち、最も新しいものを取り込み、切削アニメーションにする
    ///
    /// # Errors
    ///
    /// 今の改訂の取り込める切削シミュレーションがない場合、取り込みの失敗、切削アニメーションを作れない場合。
    pub fn import_latest(
        &mut self,
        shapes: &ProductShapeScene,
    ) -> Result<ImportedCuttingSimulation, CuttingSimulationSessionError> {
        self.target_visible_faces(shapes);
        let latest = self
            .workflow
            .latest_revision_simulation(&self.sequences, self.sequence, self.operation)?
            .map(|job| job.id)
            .ok_or(CuttingSimulationSessionError::NoSucceededSimulation)?;
        let result = self.workflow.import(latest)?;
        let OperationSpec::Scanline(scanline) = result.solver_input.operation;
        let conditions = ImportedConditions {
            tool: result.simulation_input.tool.clone(),
            scanline,
            toolpath_tolerance: result.solver_input.chord_tolerance,
            stock: result.simulation_input.stock,
            precision: result.simulation_input.precision,
        };
        let playback = CuttingSimulationPlayback::open(&result)
            .map_err(CuttingSimulationSessionError::Playback)?;
        let animation = CuttingAnimation::new(playback, result.simulation_input.tool);
        Ok(ImportedCuttingSimulation {
            job_id: latest,
            animation,
            conditions,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use application::cam_simulation_workflow::CamSimulationWorkflowError;
    use application::product_shape::ProductShapeError;
    use geo_algorithms::{Point3D, TriangleMesh3D};
    use geo_contracts::TriangleMesh3DConstructor;

    use super::*;

    /// 走査線の条件 `content` を `edit` で変えた中身
    fn with_scanline(
        content: MachiningOperationContent,
        edit: impl FnOnce(&mut ScanlineParams),
    ) -> MachiningOperationContent {
        let OperationSpec::Scanline(mut scanline) = content.spec;
        edit(&mut scanline);
        MachiningOperationContent {
            spec: OperationSpec::Scanline(scanline),
            ..content
        }
    }

    /// 粗い条件（直径 6 mm・工具長 30 mm のボールエンドミル、ピックフィード 6 mm。テストを速くする）の加工工程を
    /// 持つセッション
    fn coarse_session() -> CuttingSimulationSession {
        let mut session = CuttingSimulationSession::default();
        let content = MachiningOperationContent {
            tool: CuttingToolKind::BallEndMill.tool(6.0, Some(30.0)),
            ..session.operation_content().clone()
        };
        session.set_operation_content(with_scanline(content, |scanline| scanline.stepover = 6.0));
        session
    }

    /// 表示中の製品形状（四角錐）を粗い条件で投入・実行・取り込みした切削アニメーション
    ///
    /// `cutting_length` を渡すと、工具長をそれにする（`Some(None)` は工具長を指定しない工具）。
    pub(crate) fn imported_animation(cutting_length: Option<Option<f64>>) -> CuttingAnimation {
        let mut session = coarse_session();
        if let Some(length) = cutting_length {
            let mut content = session.operation_content().clone();
            content.tool.cutting_length = length;
            session.set_operation_content(content);
        }
        let scene = scene_with_pyramid();
        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();
        session.import_latest(&scene).unwrap().animation
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
    fn tools_are_made_from_and_read_back_as_the_selectable_kinds() {
        for kind in [CuttingToolKind::BallEndMill, CuttingToolKind::FlatEndMill] {
            let tool = kind.tool(8.0, Some(25.0));
            assert_eq!(CuttingToolKind::of(&tool), Some(kind));
            assert_eq!(tool.diameter(), 8.0);
            assert_eq!(tool.cutting_length, Some(25.0));
        }
        assert_eq!(
            CuttingToolKind::FlatEndMill.tool(8.0, None).cutting_length,
            None
        );
        let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 30.0);
        assert_eq!(CuttingToolKind::of(&radius), None);
    }

    #[test]
    fn the_edited_content_and_stock_are_held_by_the_machining_operation() {
        let mut session = CuttingSimulationSession::default();
        let content = with_scanline(session.operation_content().clone(), |scanline| {
            scanline.cut_direction = ScanlineCutDirection::ZigZag
        });
        let stock = MachiningStock {
            precision: 0.2,
            ..session.stock()
        };
        session.set_operation_content(content.clone());
        session.set_stock(stock);
        assert_eq!(session.operation_content(), &content);
        assert_eq!(session.stock(), stock);
    }

    #[test]
    fn submitted_product_shapes_are_imported_as_a_cutting_animation_after_running() {
        let scene = scene_with_pyramid();
        let mut session = coarse_session();
        assert_eq!(session.check(&scene), vec![]);
        session.submit(&scene).unwrap();
        // 実行するまでは取り込めない
        assert!(matches!(
            session.import_latest(&scene),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        let mut imported = session.import_latest(&scene).unwrap();
        assert_eq!(imported.job_id, 2);
        assert!(imported.animation.frame_count() > 1);
        assert!(!imported.animation.take_initial_stock_blocks().is_empty());
    }

    #[test]
    fn results_of_previous_conditions_are_not_imported_until_submitted_again() {
        let scene = scene_with_pyramid();
        let mut session = coarse_session();
        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();
        assert_eq!(session.import_latest(&scene).unwrap().job_id, 2);

        // 素材を変えると、前の条件の結果は取り込まない（再計算待ち）
        session.set_stock(MachiningStock {
            precision: 0.2,
            ..session.stock()
        });
        assert!(matches!(
            session.import_latest(&scene),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();
        let imported = session.import_latest(&scene).unwrap();
        assert_eq!(imported.job_id, 4);
        assert_eq!(imported.conditions.precision, 0.2);
        // 同じ条件を設定し直しても、今の条件の結果のまま取り込める
        session.set_operation_content(session.operation_content().clone());
        session.set_stock(session.stock());
        assert_eq!(session.import_latest(&scene).unwrap().job_id, 4);
        // 工程の中身を変えても、前の条件の結果は取り込まない
        let content = with_scanline(session.operation_content().clone(), |scanline| {
            scanline.feed_rate = 600.0
        });
        session.set_operation_content(content);
        assert!(matches!(
            session.import_latest(&scene),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));
        // 表示中の製品形状がなくなると、加工の対象が変わり、前の対象の結果は取り込まない
        assert!(matches!(
            session.import_latest(&ProductShapeScene::default()),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));
    }

    #[test]
    fn the_edited_conditions_are_submitted_and_returned_with_the_imported_result() {
        let scene = scene_with_pyramid();
        let mut session = coarse_session();
        let content = with_scanline(
            MachiningOperationContent {
                tool: CuttingToolKind::FlatEndMill.tool(6.0, Some(25.0)),
                ..session.operation_content().clone()
            },
            |scanline| scanline.cut_direction = ScanlineCutDirection::ZigZag,
        );
        session.set_operation_content(content.clone());
        session.set_stock(MachiningStock {
            margins: StockMargins {
                top: 2.0,
                ..StockMargins::default()
            },
            precision: 0.2,
        });
        session.submit(&scene).unwrap();
        session.run_queued_jobs().unwrap();

        let imported = session.import_latest(&scene).unwrap().conditions;
        assert_eq!(imported.tool, content.tool);
        assert_eq!(OperationSpec::Scanline(imported.scanline), content.spec);
        assert_eq!(imported.toolpath_tolerance, session.toolpath_tolerance());
        assert_eq!(imported.precision, 0.2);
        // 四角錐（0〜20, 0〜20, 0〜6）を囲む箱の上に 2 を足した素材
        assert_eq!(imported.stock.max().z(), 8.0);
        assert_eq!(imported.stock.min().x(), 0.0);
    }

    #[test]
    fn conditions_with_errors_are_reported_by_field_and_not_submitted() {
        let scene = scene_with_pyramid();
        let mut session = coarse_session();
        session.set_stock(MachiningStock {
            margins: StockMargins {
                bottom: -1.0,
                ..StockMargins::default()
            },
            precision: 0.0005,
        });
        // ピックフィードが工具径 6 mm を超える
        let content = with_scanline(session.operation_content().clone(), |scanline| {
            scanline.stepover = 7.0
        });
        session.set_operation_content(content);
        let issues = session.check(&scene);
        assert_eq!(
            issues,
            vec![
                ConditionIssue::Condition(CamConditionError::InvalidStockMargins(StockMargins {
                    bottom: -1.0,
                    ..StockMargins::default()
                })),
                ConditionIssue::Condition(CamConditionError::PrecisionFinerThanTolerance {
                    precision: 0.0005,
                    tolerance: session.toolpath_tolerance()
                }),
                ConditionIssue::Condition(CamConditionError::StepoverExceedsToolDiameter {
                    tool_diameter: 6.0,
                }),
            ]
        );
        assert!(session.submit(&scene).is_err());
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn nothing_is_submitted_without_visible_product_shapes() {
        let mut session = coarse_session();
        assert!(matches!(
            session.check(&ProductShapeScene::default())[..],
            [ConditionIssue::NoProductShape]
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
