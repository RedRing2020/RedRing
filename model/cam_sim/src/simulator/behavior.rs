use cam_core::Tool;
use geo_algorithms::octree::VoxelOctree;
use geo_algorithms::{LineSegment3D, Point3D, Scalar};

use crate::error::SimulationError;

/// 工具の切削動作を抽象化するtrait。
///
/// cam_core の `Tool` は純粋なデータ構造として保ち、
/// シミュレーション固有の振る舞いはこのtraitに集約する。
pub(super) trait ToolCuttingBehavior<T: Scalar> {
    /// セグメントを工具中心軌跡へ変換する。
    ///
    /// ボールエンドミルはZ方向へradius分オフセット、
    /// フラットエンドミルはそのまま返す。
    fn prepare_segments(
        &self,
        segments: Vec<(LineSegment3D<T>, bool)>,
    ) -> Vec<(LineSegment3D<T>, bool)>;

    /// VoxelOctreeから1セグメント分の材料を除去する。
    fn remove_material(&self, voxel_tree: &mut VoxelOctree<T>, segment: &LineSegment3D<T>);
}

pub(super) struct FlatEndMillBehavior<T: Scalar> {
    pub radius: T,
}

pub(super) struct BallEndMillBehavior<T: Scalar> {
    pub radius: T,
}

impl<T: Scalar> ToolCuttingBehavior<T> for FlatEndMillBehavior<T> {
    fn prepare_segments(
        &self,
        segments: Vec<(LineSegment3D<T>, bool)>,
    ) -> Vec<(LineSegment3D<T>, bool)> {
        segments
    }

    fn remove_material(&self, voxel_tree: &mut VoxelOctree<T>, segment: &LineSegment3D<T>) {
        voxel_tree.remove_material_swept_cylinder(segment, self.radius);
    }
}

impl<T: Scalar> ToolCuttingBehavior<T> for BallEndMillBehavior<T> {
    fn prepare_segments(
        &self,
        segments: Vec<(LineSegment3D<T>, bool)>,
    ) -> Vec<(LineSegment3D<T>, bool)> {
        segments
            .into_iter()
            .filter_map(|(segment, is_cutting)| {
                let start = Point3D::new(
                    segment.start().x(),
                    segment.start().y(),
                    segment.start().z() + self.radius,
                );
                let end = Point3D::new(
                    segment.end().x(),
                    segment.end().y(),
                    segment.end().z() + self.radius,
                );
                LineSegment3D::new(start, end).map(|line| (line, is_cutting))
            })
            .collect()
    }

    fn remove_material(&self, voxel_tree: &mut VoxelOctree<T>, segment: &LineSegment3D<T>) {
        voxel_tree.remove_material_capsule(segment, self.radius);
    }
}

/// `Tool` から対応する切削動作を生成するファクトリ関数。
///
/// 工具種別の分岐はここ1箇所に集約する。
pub(super) fn tool_cutting_behavior<T: Scalar>(
    tool: &Tool<T>,
) -> Result<Box<dyn ToolCuttingBehavior<T>>, SimulationError> {
    if tool.is_flat_end_mill() {
        Ok(Box::new(FlatEndMillBehavior {
            radius: tool.radius(),
        }))
    } else if tool.is_ball_end_mill() {
        Ok(Box::new(BallEndMillBehavior {
            radius: tool.radius(),
        }))
    } else {
        Err(SimulationError::UnsupportedToolType)
    }
}
