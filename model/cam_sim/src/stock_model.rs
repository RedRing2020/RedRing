//! 素材の正本（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md）
//!
//! 初期素材 `W₀`（直方体）と、工具の掃引プリミティブの時系列 `p₁, …, pₙ` で、現在の素材を
//! `W = W₀ \ (p₁ ∪ … ∪ pₙ)` として厳密に表す。精度は浮動小数点の精度で、メモリは工具の移動の数に
//! 比例する。点の問い合わせは、掃引プリミティブの包含箱の BVH で候補を絞る。

mod bvh;
#[cfg(test)]
mod tests;

use cam_core::{Tool, ToolPath};
use geo_algorithms::{
    Aabb3D, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM, LineSegment3DProperties, Point3D,
};

use crate::collect_toolpath_line_segments;
use crate::error::SimulationError;
use bvh::BoundingVolumeHierarchy;

/// 掃引する工具の形（3 軸。工具軸は +Z、高さは工具の先端から測る）
#[derive(Debug, Clone, Copy, PartialEq)]
enum SweptToolShape {
    /// 先端から刃長までの円柱
    FlatEndMill { radius: f64, cutting_length: f64 },
    /// 先端の球と、球の中心から刃長までの円柱（刃長が直径より短い場合も、球の全体を工具に含める）
    BallEndMill { radius: f64, cutting_length: f64 },
}

/// 工具の先端が線分に沿って直線移動したときの掃引体
#[derive(Debug, Clone, Copy, PartialEq)]
struct SweptToolPrimitive {
    start: Point3D<f64>,
    end: Point3D<f64>,
    shape: SweptToolShape,
}

impl SweptToolPrimitive {
    /// 掃引体の包含箱（移動の両端の工具の包含箱の和）
    fn bounds(&self) -> Aabb3D<f64> {
        let (radius, top) = match self.shape {
            SweptToolShape::FlatEndMill {
                radius,
                cutting_length,
            } => (radius, cutting_length),
            SweptToolShape::BallEndMill {
                radius,
                cutting_length,
            } => (radius, cutting_length.max(2.0 * radius)),
        };
        let (start, end) = (self.start, self.end);
        Aabb3D::new(
            Point3D::new(
                start.x().min(end.x()) - radius,
                start.y().min(end.y()) - radius,
                start.z().min(end.z()),
            ),
            Point3D::new(
                start.x().max(end.x()) + radius,
                start.y().max(end.y()) + radius,
                start.z().max(end.z()) + top,
            ),
        )
    }

    /// 点が掃引体に含まれるか
    fn contains(&self, point: Point3D<f64>) -> bool {
        match self.shape {
            SweptToolShape::FlatEndMill {
                radius,
                cutting_length,
            } => self.swept_cylinder_contains(point, radius, 0.0, cutting_length),
            SweptToolShape::BallEndMill {
                radius,
                cutting_length,
            } => {
                self.swept_sphere_contains(point, radius)
                    || self.swept_cylinder_contains(point, radius, radius, cutting_length)
            }
        }
    }

    /// 先端からの高さ `[bottom, top]`・半径 `radius` の垂直の円柱の掃引体に、点が含まれるか
    ///
    /// 高さの条件を満たす移動のパラメータの区間で、XY 平面上の工具の軸と点の距離の最小が半径以下か。
    fn swept_cylinder_contains(
        &self,
        point: Point3D<f64>,
        radius: f64,
        bottom: f64,
        top: f64,
    ) -> bool {
        let (start, direction) = (self.start, self.direction());
        let height = point.z() - start.z();
        let (mut t_min, mut t_max) = (0.0_f64, 1.0_f64);
        if direction[2] == 0.0 {
            if !(bottom..=top).contains(&height) {
                return false;
            }
        } else {
            // bottom ≤ height - t dz ≤ top
            let (a, b) = (
                (height - top) / direction[2],
                (height - bottom) / direction[2],
            );
            t_min = t_min.max(a.min(b));
            t_max = t_max.min(a.max(b));
            if t_min > t_max {
                return false;
            }
        }
        let (dx, dy) = (point.x() - start.x(), point.y() - start.y());
        let length_squared = direction[0] * direction[0] + direction[1] * direction[1];
        let t = if length_squared == 0.0 {
            t_min
        } else {
            ((dx * direction[0] + dy * direction[1]) / length_squared).clamp(t_min, t_max)
        };
        let (rx, ry) = (dx - t * direction[0], dy - t * direction[1]);
        rx * rx + ry * ry <= radius * radius
    }

    /// 先端から半径の高さに中心を持つ球の掃引体（カプセル）に、点が含まれるか
    fn swept_sphere_contains(&self, point: Point3D<f64>, radius: f64) -> bool {
        let direction = self.direction();
        let offset = [
            point.x() - self.start.x(),
            point.y() - self.start.y(),
            point.z() - (self.start.z() + radius),
        ];
        let length_squared = direction.iter().map(|c| c * c).sum::<f64>();
        let t = if length_squared == 0.0 {
            0.0
        } else {
            (offset
                .iter()
                .zip(direction)
                .map(|(o, d)| o * d)
                .sum::<f64>()
                / length_squared)
                .clamp(0.0, 1.0)
        };
        let distance_squared = offset
            .iter()
            .zip(direction)
            .map(|(o, d)| (o - t * d) * (o - t * d))
            .sum::<f64>();
        distance_squared <= radius * radius
    }

    fn direction(&self) -> [f64; 3] {
        [
            self.end.x() - self.start.x(),
            self.end.y() - self.start.y(),
            self.end.z() - self.start.z(),
        ]
    }
}

/// 素材の正本
#[derive(Debug, Clone)]
pub struct StockModel {
    stock: Aabb3D<f64>,
    primitives: Vec<SweptToolPrimitive>,
    hierarchy: BoundingVolumeHierarchy,
}

impl StockModel {
    /// 直方体の素材を、ツールパスの切削の区間に沿って工具で除去した正本を作る
    ///
    /// 円弧の区間は既定の弦の許容誤差で直線の区間の列にする。ツールパスの座標は工具の先端の位置とする。
    ///
    /// # Errors
    ///
    /// 工具がフラット・ボールエンドミルでない場合は `SimulationError::UnsupportedToolType` を返す。
    pub fn from_toolpath(
        stock: Aabb3D<f64>,
        toolpath: &ToolPath<f64>,
        tool: &Tool<f64>,
    ) -> Result<Self, SimulationError> {
        let shape = if tool.is_flat_end_mill() {
            SweptToolShape::FlatEndMill {
                radius: tool.radius(),
                cutting_length: tool.cutting_length,
            }
        } else if tool.is_ball_end_mill() {
            SweptToolShape::BallEndMill {
                radius: tool.radius(),
                cutting_length: tool.cutting_length,
            }
        } else {
            return Err(SimulationError::UnsupportedToolType);
        };
        let primitives: Vec<_> =
            collect_toolpath_line_segments(toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM)
                .into_iter()
                .filter(|(_, is_cutting)| *is_cutting)
                .map(|(segment, _)| SweptToolPrimitive {
                    start: segment.start_point(),
                    end: segment.end_point(),
                    shape,
                })
                .collect();
        let bounds: Vec<_> = primitives.iter().map(SweptToolPrimitive::bounds).collect();
        Ok(Self {
            stock,
            hierarchy: BoundingVolumeHierarchy::build(&bounds),
            primitives,
        })
    }

    /// 点が残った材料の内部か（境界を含む素材に含まれ、どの掃引体にも含まれない）
    pub fn contains_material(&self, point: Point3D<f64>) -> bool {
        self.stock.contains_point(point)
            && !self
                .hierarchy
                .any_containing(point, |index| self.primitives[index].contains(point))
    }
}
