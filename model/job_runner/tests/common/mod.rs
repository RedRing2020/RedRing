//! ランナー・ジョブマネージャーの試験で共通に使う、作業のフォルダと入力

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use cam_algorithms::{CamSolverInput, SolverGeometry};
use cam_core::{
    CoordinateFrame, LengthUnit, OperationSpec, ScanlineCutDirection, ScanlineParams,
    ScanlineToolRange, Tool,
};
use cam_sim::{
    CuttingSimulationInput, SnapshotInterval, write_cam_solver_input_v1,
    write_cutting_simulation_input_v1,
};
use geo_algorithms::{
    Aabb3D, Point3D, TessellationLimits, TriangleMesh3D, TriangleMesh3DConstructor,
};

/// 試験ごとの作業のフォルダ（試験の終わりに消す）
pub struct WorkDir(pub PathBuf);

impl WorkDir {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "redring-runner-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 素材（0..60 × 0..60 × 0..10）の中の四角錐（底面 z=0、頂点 z=10）
pub fn pyramid() -> SolverGeometry {
    SolverGeometry::TriangleMesh(
        TriangleMesh3D::new(
            vec![
                Point3D::new(10.0, 10.0, 0.0),
                Point3D::new(50.0, 10.0, 0.0),
                Point3D::new(50.0, 50.0, 0.0),
                Point3D::new(10.0, 50.0, 0.0),
                Point3D::new(30.0, 30.0, 10.0),
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
        .unwrap(),
    )
}

pub fn tool() -> Tool<f64> {
    Tool::ball_end_mill("BEM10".to_string(), 10.0, 40.0)
}

pub fn cam_input() -> CamSolverInput {
    CamSolverInput {
        operation_id: "op-1".to_string(),
        tool: tool(),
        geometry: pyramid(),
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover: 5.0,
            feed_rate: 1200.0,
            tool_range: ScanlineToolRange::GeometryExtent,
            cut_direction: ScanlineCutDirection::DownCut,
            angle_degrees: 0.0,
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.01,
        tessellation_limits: TessellationLimits::default(),
        boundary: None,
    }
}

pub fn sim_input() -> CuttingSimulationInput {
    CuttingSimulationInput {
        stock: Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(60.0, 60.0, 10.0)),
        tool: tool(),
        precision: 0.5,
        snapshot_interval: SnapshotInterval::ByToolAndGrid,
        product: pyramid(),
        tolerance: 0.01,
    }
}

pub fn write_cam_input(path: &Path, input: &CamSolverInput) {
    let mut bytes = Vec::new();
    write_cam_solver_input_v1(&mut bytes, input).unwrap();
    fs::write(path, bytes).unwrap();
}

pub fn write_sim_input(path: &Path) {
    let mut bytes = Vec::new();
    write_cutting_simulation_input_v1(&mut bytes, &sim_input()).unwrap();
    fs::write(path, bytes).unwrap();
}
