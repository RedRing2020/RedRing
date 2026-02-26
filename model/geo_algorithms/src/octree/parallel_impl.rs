use super::{HasPosition, Octree};
use geo_core::{Aabb3D, Point3D};
use geo_foundation::Scalar;
use rayon::prelude::*;

impl<T: Scalar + Sync, D: Clone + Sync> Octree<T, D> {
    /// 複数範囲を並列に検索
    pub fn query_regions_parallel<'a>(&'a self, regions: &'a [Aabb3D<T>]) -> Vec<Vec<&'a D>> {
        regions
            .par_iter()
            .map(|region| self.query_region(region))
            .collect()
    }
}

impl<T: Scalar + Send + Sync, D: Clone + HasPosition<T> + Sync> Octree<T, D> {
    /// 複数点の最近傍探索を並列に実行
    pub fn nearest_many_parallel<'a>(
        &'a self,
        points: &'a [Point3D<T>],
    ) -> Vec<Option<(&'a D, T)>> {
        points.par_iter().map(|point| self.nearest(point)).collect()
    }
}
