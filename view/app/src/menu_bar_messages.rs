use viewmodel::message_catalog::{TableMessageCatalog, UiLocale, UiMessage, resolve_message};

const MENU_BAR_TEMPLATE_JA: &[(&str, &str)] = &[
    ("menu.file", "ファイル"),
    ("menu.file.new", "新規"),
    ("menu.file.open", "開く…"),
    ("menu.file.save", "保存"),
    ("menu.file.save_as", "名前を付けて保存…"),
    ("menu.file.load", "読み込み…"),
    ("menu.file.exit", "終了"),
    ("file.model_file", "モデルファイル"),
    ("file.untitled", "無題"),
    (
        "file.confirm.unsaved",
        "保存していない変更があります。保存しますか？",
    ),
    ("file.confirm.save", "保存する"),
    ("file.confirm.dont_save", "保存しない"),
    ("file.confirm.cancel", "やめる"),
    ("menu.file.load.supported", "対応する形状ファイル"),
    ("menu.machining", "加工"),
    ("menu.machining.open_sequence", "加工手順を開く"),
    ("menu.machining.close_sequence", "加工手順を閉じる"),
    ("menu.machining.operation_list", "工程リスト"),
    ("menu.machining.operation_settings", "工程の設定"),
    ("menu.machining.sequence_settings", "手順の設定"),
    ("menu.view", "表示"),
    ("menu.view.language", "言語"),
    ("menu.help", "ヘルプ"),
    ("menu.help.controls", "操作の一覧（F1）"),
    ("menu.help.about", "バージョンの情報"),
];

const MENU_BAR_TEMPLATE_EN: &[(&str, &str)] = &[
    ("menu.file", "File"),
    ("menu.file.new", "New"),
    ("menu.file.open", "Open…"),
    ("menu.file.save", "Save"),
    ("menu.file.save_as", "Save As…"),
    ("menu.file.load", "Load…"),
    ("menu.file.exit", "Exit"),
    ("file.model_file", "Model file"),
    ("file.untitled", "Untitled"),
    (
        "file.confirm.unsaved",
        "There are unsaved changes. Do you want to save them?",
    ),
    ("file.confirm.save", "Save"),
    ("file.confirm.dont_save", "Don't Save"),
    ("file.confirm.cancel", "Cancel"),
    ("menu.file.load.supported", "Supported shape files"),
    ("menu.machining", "Machining"),
    ("menu.machining.open_sequence", "Open machining sequence"),
    ("menu.machining.close_sequence", "Close machining sequence"),
    ("menu.machining.operation_list", "Operation list"),
    ("menu.machining.operation_settings", "Operation settings"),
    ("menu.machining.sequence_settings", "Sequence settings"),
    ("menu.view", "View"),
    ("menu.view.language", "Language"),
    ("menu.help", "Help"),
    ("menu.help.controls", "Controls (F1)"),
    ("menu.help.about", "About"),
];

pub const MENU_BAR_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    MENU_BAR_TEMPLATE_JA,
    MENU_BAR_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

pub fn resolve_menu_bar_text(locale: UiLocale, key: &str) -> String {
    let message = UiMessage {
        key: key.to_string(),
        args: Vec::new(),
    };
    resolve_message(&MENU_BAR_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::resolve_menu_bar_text;
    use viewmodel::message_catalog::UiLocale;

    #[test]
    fn resolves_menu_bar_text_for_ja_and_en() {
        assert_eq!(
            resolve_menu_bar_text(UiLocale::Ja, "menu.file.load"),
            "読み込み…"
        );
        assert_eq!(
            resolve_menu_bar_text(UiLocale::En, "menu.file.load"),
            "Load…"
        );
    }
}
