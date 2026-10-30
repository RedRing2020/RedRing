//! 製品形状からの切削シミュレーションの投入・実行・取り込み
//!
//! 工具・加工の条件は、工具ライブラリと条件の入力ができるまでの初期値（`CuttingSimulationConditions`）を使う。
//! 実行の入口は、ジョブマネージャーができるまでの同じプロセスの中の実行とする。

use application::cam_simulation_workflow::{
    CamSimulationWorkflow, CamSimulationWorkflowError, ProductShapeSimulationRequest,
};
use cam_algorithms::{OperationDefinition, ProcessTemplate, ScanlineParams, ScanlineToolRange};
use cam_core::Tool;
use cam_sim::SnapshotInterval;
pub use cam_sim::StockMargins;

use crate::cutting_animation::{CuttingAnimation, CuttingAnimationError};
use crate::product_shape_converter::ProductShapeScene;

/// 工具の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuttingToolKind {
    BallEndMill,
    FlatEndMill,
}

/// 切削シミュレーションの工具・加工の条件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CuttingSimulationConditions {
    /// 工具の直径（mm）
    pub tool_diameter: f64,
    /// 工具長（mm。`None` は長さに制限のない工具）
    pub cutting_length: Option<f64>,
    /// 走査線の条件
    pub scanline: ScanlineParams,
    /// 製品形状を囲む箱から素材の箱の各面までの余裕（mm）
    pub stock_margins: StockMargins,
    /// 素材の形状の許容誤差（mm）
    pub precision: f64,
}

impl Default for CuttingSimulationConditions {
    fn default() -> Self {
        Self {
            tool_diameter: 10.0,
            cutting_length: None,
            scanline: ScanlineParams {
                stepover: 2.0,
                sample_pitch: 1.0,
                feed_rate: 1200.0,
                tool_range: ScanlineToolRange::default(),
            },
            stock_margins: StockMargins::default(),
            precision: 0.01,
        }
    }
}

impl CuttingSimulationConditions {
    fn tool(&self, kind: CuttingToolKind) -> Tool<f64> {
        // 工具長は、作った後に条件の値（指定しない場合は `None`）にする
        let mut tool = match kind {
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
}

/// 取り込んだ切削シミュレーション
#[derive(Debug)]
pub struct ImportedCuttingSimulation {
    /// 切削シミュレーションのジョブの番号
    pub job_id: u64,
    /// 切削アニメーション
    pub animation: CuttingAnimation,
}

/// 投入・実行・取り込みの失敗
#[derive(Debug)]
pub enum CuttingSimulationSessionError {
    /// 投入・実行・取り込みの失敗
    Workflow(CamSimulationWorkflowError),
    /// 取り込める（正常終了した）切削シミュレーションがない
    NoSucceededSimulation,
    /// 取り込んだ結果から切削アニメーションを作れない
    Animation(CuttingAnimationError),
}

impl std::fmt::Display for CuttingSimulationSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workflow(error) => write!(f, "{error}"),
            Self::NoSucceededSimulation => write!(f, "no succeeded cutting simulation to import"),
            Self::Animation(error) => write!(f, "{error}"),
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
#[derive(Debug, Default)]
pub struct CuttingSimulationSession {
    workflow: CamSimulationWorkflow,
    conditions: CuttingSimulationConditions,
}

impl CuttingSimulationSession {
    pub fn new(conditions: CuttingSimulationConditions) -> Self {
        Self {
            workflow: CamSimulationWorkflow::new(),
            conditions,
        }
    }

    /// 次の投入に使う工具長（`None` は長さに制限のない工具）
    pub fn tool_length(&self) -> Option<f64> {
        self.conditions.cutting_length
    }

    /// 次の投入から使う工具長を `length` にする（`None` は長さに制限のない工具）
    pub fn set_tool_length(&mut self, length: Option<f64>) {
        self.conditions.cutting_length = length;
    }

    /// 次の投入に使う素材の余裕
    pub fn stock_margins(&self) -> StockMargins {
        self.conditions.stock_margins
    }

    /// 次の投入から使う素材の余裕を `margins` にする
    pub fn set_stock_margins(&mut self, margins: StockMargins) {
        self.conditions.stock_margins = margins;
    }

    /// 表示中の製品形状のすべての面を加工の対象として、工具 `tool` で投入する（実行しない）
    ///
    /// # Errors
    ///
    /// 表示中の製品形状がない場合を含む、投入の失敗。
    pub fn submit(
        &mut self,
        shapes: &ProductShapeScene,
        tool: CuttingToolKind,
    ) -> Result<(), CuttingSimulationSessionError> {
        let request = ProductShapeSimulationRequest {
            face_ids: shapes.store().visible_face_ids(),
            template: ProcessTemplate {
                template_id: "default".to_string(),
                default_operation_type: Some("scanline".to_string()),
                default_tolerance_profile: Some("press_rough".to_string()),
            },
            operation: OperationDefinition {
                operation_id: "scanline-1".to_string(),
                machining_stage: "finish".to_string(),
                operation_type: None,
                tolerance_profile: None,
                boundary: None,
            },
            tool: self.conditions.tool(tool),
            scanline: self.conditions.scanline,
            stock_margins: self.conditions.stock_margins,
            precision: self.conditions.precision,
            snapshot_interval: SnapshotInterval::default(),
        };
        self.workflow
            .submit_product_shapes(shapes.store(), request)?;
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

    /// 正常終了した切削シミュレーションのうち最も新しいものを取り込み、切削アニメーションにする
    ///
    /// # Errors
    ///
    /// 取り込める切削シミュレーションがない場合、取り込みの失敗、切削アニメーションを作れない場合。
    pub fn import_latest(
        &self,
    ) -> Result<ImportedCuttingSimulation, CuttingSimulationSessionError> {
        let latest = self
            .workflow
            .succeeded_simulations()
            .last()
            .map(|job| job.id)
            .ok_or(CuttingSimulationSessionError::NoSucceededSimulation)?;
        let result = self.workflow.import(latest)?;
        let animation = CuttingAnimation::new(
            &result.toolpath_artifact,
            &result.simulation_artifact,
            result.tool,
        )
        .map_err(CuttingSimulationSessionError::Animation)?;
        Ok(ImportedCuttingSimulation {
            job_id: latest,
            animation,
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
                sample_pitch: 10.0,
                feed_rate: 1200.0,
                tool_range: ScanlineToolRange::default(),
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
        let path = std::env::temp_dir().join(format!(
            "redring_cutting_simulation_session_{}.stl",
            std::process::id()
        ));
        geo_io::stl::save_stl(&pyramid, &path).unwrap();
        let mut scene = ProductShapeScene::default();
        scene.import_stl(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        scene
    }

    #[test]
    fn submitted_product_shapes_are_imported_as_a_cutting_animation_after_running() {
        let scene = scene_with_pyramid();
        let mut session = CuttingSimulationSession::new(coarse());
        session
            .submit(&scene, CuttingToolKind::BallEndMill)
            .unwrap();
        // 実行するまでは取り込めない
        assert!(matches!(
            session.import_latest(),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        let imported = session.import_latest().unwrap();
        assert_eq!(imported.job_id, 2);
        assert!(imported.animation.frame_count() > 1);
        assert!(!imported.animation.stock_blocks().is_empty());
    }

    #[test]
    fn nothing_is_submitted_without_visible_product_shapes() {
        let mut session = CuttingSimulationSession::new(coarse());
        let error = session
            .submit(&ProductShapeScene::default(), CuttingToolKind::FlatEndMill)
            .unwrap_err();
        assert!(matches!(
            error,
            CuttingSimulationSessionError::Workflow(CamSimulationWorkflowError::ProductShape(
                ProductShapeError::NoFaces
            ))
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }
}
