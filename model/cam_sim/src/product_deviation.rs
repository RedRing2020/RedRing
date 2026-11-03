//! 最後の時点の素材と製品形状の比較（削り込み・削り残し。設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §5.3）
//!
//! 最後の時点の、削った面の上のデクセルの区間の端点（正本から求めた厳密な点）ごとに、製品形状の三角形までの最短距離を
//! 求め、製品形状の上側の包絡より下（内側）を負とする。削り込みは一度できると後から戻らないため最後の時点に残り、
//! 削り残しは加工の後でなければ意味がないため、最後の時点だけを比べる。

use cam_algorithms::ProductSurface;

use crate::tri_dexel::TriDexelModel;

/// 最後の時点の素材の、削った面の上の点ごとの、製品形状までの符号つきの距離
#[derive(Debug, Clone, PartialEq)]
pub struct ProductDeviations {
    /// 判定の許容値（ツールパス生成のトレランス）。距離が `-tolerance` より小さい点は削り込み、`tolerance` より
    /// 大きい点は削り残し
    pub tolerance: f64,
    /// 格子（X・Y・Z の順）ごとの、最後の時点の区間の端点（デクセルの番号の順・区間の順・始まりと終わりの順）の距離。
    /// 製品形状の上側の包絡より下（内側）を負とする。素材の範囲の端にある、削っていない端点は NaN
    pub distances: [Vec<f32>; 3],
}

/// 最後の時点の素材 `model` を、製品形状 `surface` と比べる（判定の許容値は `tolerance`）
pub(crate) fn compare_with_product(
    model: &TriDexelModel,
    surface: &ProductSurface,
    tolerance: f64,
) -> ProductDeviations {
    let distances = model.endpoint_values(|point| {
        let distance = surface.distance_to(point);
        let inside = surface
            .height_at(point.x(), point.y())
            .is_some_and(|height| point.z() < height);
        (if inside { -distance } else { distance }) as f32
    });
    ProductDeviations {
        tolerance,
        distances,
    }
}

#[cfg(test)]
mod tests {
    use cam_algorithms::ProductSurface;
    use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
    use geo_algorithms::{Aabb3D, Point3D, TriangleMesh3D, TriangleMesh3DConstructor};

    use super::compare_with_product;
    use crate::stock_model::StockModel;
    use crate::tri_dexel::TriDexelModel;

    const TOLERANCE: f64 = 0.001;

    /// 高さ 0 の、x が `x_range` の範囲の長方形の製品形状（y は -5..5）
    fn plane(x_range: (f64, f64)) -> ProductSurface {
        let (low, high) = x_range;
        let mesh = TriangleMesh3D::new(
            vec![
                Point3D::new(low, -5.0, 0.0),
                Point3D::new(high, -5.0, 0.0),
                Point3D::new(high, 5.0, 0.0),
                Point3D::new(low, 5.0, 0.0),
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        )
        .unwrap();
        ProductSurface::new(&mesh).unwrap()
    }

    /// 素材（-5..5 × -5..5 × -2..1）を、直径 2 のボールエンドミルで、y = 0 は先端の高さ 0（製品形状どおり）、
    /// y = 3 は先端の高さ -0.5（製品形状より 0.5 深い）で X の向きに削った、最後の時点のトリデクセル
    fn model() -> TriDexelModel {
        let stock = Aabb3D::new(Point3D::new(-5.0, -5.0, -2.0), Point3D::new(5.0, 5.0, 1.0));
        let points = [
            Point3D::new(-6.0, 0.0, 0.0),
            Point3D::new(6.0, 0.0, 0.0),
            Point3D::new(6.0, 3.0, -0.5),
            Point3D::new(-6.0, 3.0, -0.5),
        ];
        let toolpath = ToolPath::new(
            "BEM2".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(
                0,
                0.0,
                points
                    .windows(2)
                    .map(|pair| {
                        PathSegment::new_line(
                            pair[0],
                            pair[1],
                            SegmentType::Cutting { feed_rate: 100.0 },
                        )
                    })
                    .collect(),
            )],
            vec![],
        );
        let tool = Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0);
        let model = StockModel::from_toolpath(stock, &toolpath, &tool).unwrap();
        TriDexelModel::from_stock_model(&model, 0.25, &[model.primitives().len()])
    }

    /// 最後の時点の区間の端点と、`distances` の同じ位置の距離の組（格子ごと）
    fn endpoints_with(
        model: &TriDexelModel,
        distances: &[Vec<f32>; 3],
    ) -> Vec<(Point3D<f64>, f32)> {
        let mut pairs = Vec::new();
        for (grid, layout) in model.layouts().iter().enumerate() {
            let axis = usize::from(layout.axis);
            let plane_axes: Vec<usize> = (0..3).filter(|&other| other != axis).collect();
            let mut values = distances[grid].iter();
            for i in 0..layout.count[0] as usize {
                for j in 0..layout.count[1] as usize {
                    let index = i * layout.count[1] as usize + j;
                    for &(low, high) in model.current_intervals(grid, index) {
                        for end in [low, high] {
                            let mut point = [0.0; 3];
                            point[plane_axes[0]] = layout.origin[0] + i as f64 * layout.pitch[0];
                            point[plane_axes[1]] = layout.origin[1] + j as f64 * layout.pitch[1];
                            point[axis] = end;
                            let [x, y, z] = point;
                            pairs.push((Point3D::new(x, y, z), *values.next().unwrap()));
                        }
                    }
                }
            }
            assert!(values.next().is_none(), "grid {grid}");
        }
        pairs
    }

    #[test]
    fn distances_to_a_plane_are_the_heights_of_the_cut_points() {
        let model = model();
        let deviations = compare_with_product(&model, &plane((-5.0, 5.0)), TOLERANCE);
        assert_eq!(deviations.tolerance, TOLERANCE);
        let pairs = endpoints_with(&model, &deviations.distances);
        let cut: Vec<_> = pairs.iter().filter(|(_, d)| !d.is_nan()).collect();
        assert!(!cut.is_empty());
        // 製品形状は高さ 0 の平面のため、削った点の距離は点の高さ（下は削り込みで負、上は削り残しで正）
        for (point, distance) in &cut {
            assert!(
                (f64::from(*distance) - point.z()).abs() < 1e-5,
                "{point:?}: {distance}"
            );
        }
        // 製品形状どおりに削った y = 0 の付近に削り込みはなく、0.5 深く削った y = 3 の付近だけにある
        assert!(
            cut.iter()
                .filter(|(point, _)| point.y() < 1.5)
                .all(|(_, d)| f64::from(*d) >= -TOLERANCE)
        );
        let deepest = cut.iter().map(|(_, d)| *d).fold(f32::INFINITY, f32::min);
        assert!((-0.5..-0.45).contains(&deepest), "{deepest}");
        // 削っていない端点（素材の範囲の端）は NaN
        let uncut = pairs.iter().filter(|(_, d)| d.is_nan());
        let stock = [(-5.0, 5.0), (-5.0, 5.0), (-2.0, 1.0)];
        for (point, _) in uncut {
            let coordinates = [point.x(), point.y(), point.z()];
            assert!(
                coordinates
                    .iter()
                    .zip(stock)
                    .any(|(&value, (low, high))| value == low || value == high),
                "{point:?}"
            );
        }
    }

    #[test]
    fn points_where_there_is_no_product_are_outside() {
        let model = model();
        // 製品形状は x < 0 だけ
        let deviations = compare_with_product(&model, &plane((-5.0, 0.0)), TOLERANCE);
        let pairs = endpoints_with(&model, &deviations.distances);
        let beyond: Vec<_> = pairs
            .iter()
            .filter(|(point, d)| !d.is_nan() && point.x() > 0.5 && point.z() < 0.0)
            .collect();
        assert!(!beyond.is_empty());
        // 製品形状の端（x = 0 の辺）までの距離で、外側（正）
        for (point, distance) in beyond {
            let expected = point.x().hypot(point.z());
            assert!(
                (f64::from(*distance) - expected).abs() < 1e-5,
                "{point:?}: {distance}"
            );
        }
    }
}
