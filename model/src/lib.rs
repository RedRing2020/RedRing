// Model crate now serves as minimal abstraction layer only
//
// Migration completed:
// - geometry -> geo_primitives (concrete implementations)
// - analysis -> analysis crate (numerical computing)
// - geometry_common -> geo_foundation/extensions/intersection (new design)
// - geometry_kind -> geo_foundation/classification (unified classification)
// - geometry_trait -> geo_foundation/core + extensions (trait abstraction)

// No public API - model now serves as workspace organization only
