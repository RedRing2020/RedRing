use viewmodel::message_catalog::{resolve_message, TableMessageCatalog, UiLocale, UiMessage};

const MENU_BAR_TEMPLATE_JA: &[(&str, &str)] = &[
    ("menu.file", "ファイル"),
    ("menu.file.load", "読み込み…"),
    ("menu.file.load.supported", "対応する形状ファイル"),
    ("menu.help", "ヘルプ"),
    ("menu.help.controls", "操作の一覧（F1）"),
    ("menu.help.about", "バージョンの情報"),
];

const MENU_BAR_TEMPLATE_EN: &[(&str, &str)] = &[
    ("menu.file", "File"),
    ("menu.file.load", "Load…"),
    ("menu.file.load.supported", "Supported shape files"),
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
