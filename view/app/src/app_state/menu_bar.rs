//! AppState のメニューバーの項目を扱うモジュール。

use super::AppState;
use crate::menu_bar_messages::resolve_menu_bar_text;
use crate::menu_bar_ui::MenuBarUiState;

/// 読み込める形状ファイルの形式（ファイルを開く画面の種類の名前と拡張子。大文字・小文字を区別する OS のため両方）
const SHAPE_FILE_FORMATS: &[(&str, &[&str])] = &[("STL", &["stl", "STL"])];

impl AppState {
    pub(crate) fn prepare_menu_bar_state(&self) -> MenuBarUiState {
        MenuBarUiState {
            locale: self.settings_panel_locale,
            load_requested: false,
            controls_requested: false,
            about_requested: false,
        }
    }

    pub(crate) fn apply_menu_bar_state(&mut self, state: MenuBarUiState) {
        if state.load_requested {
            self.shape_file_load_requested = true;
        }
        if state.controls_requested {
            self.open_help_controls();
        }
        if state.about_requested {
            self.open_help_about();
        }
    }

    /// 選ばれたメニューの項目を行う（描画の途中ではなく、イベントの処理が終わった後に呼ぶ）
    pub fn run_requested_menu_actions(&mut self) {
        if std::mem::take(&mut self.shape_file_load_requested) {
            self.load_shape_file();
        }
    }

    /// ファイルを開く画面で選んだ形状ファイルを、ドラッグ＆ドロップと同じ経路で読み込む（選ばなかった場合は何もしない）
    ///
    /// 画面を開いている間は、アプリの描画が止まる。
    fn load_shape_file(&mut self) {
        let supported: Vec<&str> = SHAPE_FILE_FORMATS
            .iter()
            .flat_map(|(_, extensions)| extensions.iter().copied())
            .collect();
        let mut dialog = rfd::FileDialog::new().set_parent(&*self.window).add_filter(
            resolve_menu_bar_text(self.settings_panel_locale, "menu.file.load.supported"),
            &supported,
        );
        for (name, extensions) in SHAPE_FILE_FORMATS {
            dialog = dialog.add_filter(*name, extensions);
        }
        if let Some(path) = dialog.pick_file() {
            self.import_product_shape(&path);
        }
    }
}
