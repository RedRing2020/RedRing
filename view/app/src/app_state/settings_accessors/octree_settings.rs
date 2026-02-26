use crate::app_state::AppState;

impl AppState {
    /// Octree深さグラデーション開始色（浅い）を設定
    pub fn set_octree_gradient_start(&mut self, color: [f32; 3]) {
        self.octree_visualization_settings.gradient_start = color;
    }

    /// Octree表示色を単色で設定（開始色・終了色の両方に同じ色を適用）
    pub fn set_octree_single_color(&mut self, color: [f32; 3]) {
        self.octree_visualization_settings.gradient_start = color;
        self.octree_visualization_settings.gradient_end = color;
    }

    /// Octree表示色をグラデーションで一括設定
    pub fn set_octree_gradient_colors(&mut self, start: [f32; 3], end: [f32; 3]) {
        self.octree_visualization_settings.gradient_start = start;
        self.octree_visualization_settings.gradient_end = end;
    }

    /// Octree深さグラデーション終了色（深い）を設定
    pub fn set_octree_gradient_end(&mut self, color: [f32; 3]) {
        self.octree_visualization_settings.gradient_end = color;
    }

    /// Octree深さグラデーション開始色（浅い）を取得
    pub fn octree_gradient_start(&self) -> [f32; 3] {
        self.octree_visualization_settings.gradient_start
    }

    /// Octree深さグラデーション終了色（深い）を取得
    pub fn octree_gradient_end(&self) -> [f32; 3] {
        self.octree_visualization_settings.gradient_end
    }

    /// Octreeトレランス: point_aabb_half_extent を設定
    pub fn set_octree_tolerance_point_aabb_half_extent(&mut self, value: f64) {
        self.octree_visualization_settings
            .octree_tolerance
            .point_aabb_half_extent = value;
    }

    /// Octreeトレランスを一括設定
    pub fn set_octree_tolerance(
        &mut self,
        point_aabb_half_extent: f64,
        query_expand: f64,
        nearest_prune_margin: f64,
    ) {
        self.octree_visualization_settings
            .octree_tolerance
            .point_aabb_half_extent = point_aabb_half_extent;
        self.octree_visualization_settings
            .octree_tolerance
            .query_expand = query_expand;
        self.octree_visualization_settings
            .octree_tolerance
            .nearest_prune_margin = nearest_prune_margin;
    }

    /// Octreeトレランスを単一値で一括設定（3項目に同値を適用）
    pub fn set_octree_tolerance_uniform(&mut self, value: f64) {
        self.set_octree_tolerance(value, value, value);
    }

    /// Octreeトレランス: query_expand を設定
    pub fn set_octree_tolerance_query_expand(&mut self, value: f64) {
        self.octree_visualization_settings
            .octree_tolerance
            .query_expand = value;
    }

    /// Octreeトレランス: nearest_prune_margin を設定
    pub fn set_octree_tolerance_nearest_prune_margin(&mut self, value: f64) {
        self.octree_visualization_settings
            .octree_tolerance
            .nearest_prune_margin = value;
    }

    /// Octreeトレランス: point_aabb_half_extent を取得
    pub fn octree_tolerance_point_aabb_half_extent(&self) -> f64 {
        self.octree_visualization_settings
            .octree_tolerance
            .point_aabb_half_extent
    }

    /// Octreeトレランス: query_expand を取得
    pub fn octree_tolerance_query_expand(&self) -> f64 {
        self.octree_visualization_settings
            .octree_tolerance
            .query_expand
    }

    /// Octreeトレランス: nearest_prune_margin を取得
    pub fn octree_tolerance_nearest_prune_margin(&self) -> f64 {
        self.octree_visualization_settings
            .octree_tolerance
            .nearest_prune_margin
    }
}
