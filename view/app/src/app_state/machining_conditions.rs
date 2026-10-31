//! AppState の加工条件のウィンドウ（計算条件の入力と投入）を扱うモジュール。

use super::AppState;
use crate::machining_conditions_ui::MachiningConditionsUiState;

impl AppState {
    pub fn is_machining_conditions_open(&self) -> bool {
        self.machining_conditions_open
    }

    pub fn toggle_machining_conditions(&mut self) {
        self.machining_conditions_open = !self.machining_conditions_open;
        tracing::info!(
            "加工条件のウィンドウ{}（Shift+M）",
            if self.machining_conditions_open {
                "表示"
            } else {
                "非表示"
            }
        );
    }

    /// 計算条件を確かめ直す必要がある場合（条件・製品形状が変わった後）に確かめる
    fn refresh_condition_issues(&mut self) {
        if self.condition_issues.is_none() {
            self.condition_issues = Some(self.cutting_simulation.check(&self.product_shapes));
        }
    }

    pub(crate) fn prepare_machining_conditions_state(&mut self) -> MachiningConditionsUiState {
        if self.machining_conditions_open {
            self.refresh_condition_issues();
        }
        MachiningConditionsUiState {
            open: self.machining_conditions_open,
            locale: self.settings_panel_locale,
            face_count: self.product_shapes.store().visible_face_ids().len(),
            conditions: self.cutting_simulation.conditions(),
            tool_length_input: self.tool_length_input,
            toolpath_tolerance: self.cutting_simulation.toolpath_tolerance(),
            issues: self.condition_issues.clone().unwrap_or_default(),
            imported: self
                .cutting_animation_job_id
                .zip(self.imported_conditions.clone()),
            submit_requested: false,
        }
    }

    pub(crate) fn apply_machining_conditions_state(&mut self, state: MachiningConditionsUiState) {
        self.machining_conditions_open = state.open;
        self.tool_length_input = state.tool_length_input;
        if state.conditions != self.cutting_simulation.conditions() {
            self.cutting_simulation.set_conditions(state.conditions);
            self.condition_issues = None;
        }
        if state.submit_requested {
            self.submit_cutting_simulation();
        }
    }
}
