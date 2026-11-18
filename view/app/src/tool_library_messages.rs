//! ツールライブラリのウィンドウの文言（日本語・英語）

use viewmodel::message_catalog::{
    TableMessageCatalog, UiLocale, UiMessage, UiMessageArg, resolve_message,
};

const TOOL_LIBRARY_TEMPLATE_JA: &[(&str, &str)] = &[
    ("tool_library.window.title", "ツールライブラリ"),
    ("tool_library.libraries", "ライブラリ"),
    ("tool_library.library.create", "新しく作る…"),
    ("tool_library.library.open", "開く…"),
    ("tool_library.library.remove", "一覧から外す"),
    ("tool_library.library.rename", "名前を替える"),
    ("tool_library.library.none", "ライブラリがありません"),
    ("tool_library.library.read_only", "読むだけ"),
    ("tool_library.library.unreadable", "読めない"),
    ("tool_library.library.duplicate", "同じライブラリ"),
    (
        "tool_library.library.read_only.reason",
        "このライブラリに書く許可がないため、読むだけで使います",
    ),
    (
        "tool_library.library.unreadable.reason",
        "読めません: {reason}。前に読めた内容があれば、読むだけで使います",
    ),
    (
        "tool_library.library.duplicate.reason",
        "{first} と同じライブラリのため、使いません",
    ),
    ("tool_library.file_type", "ツールライブラリ"),
    ("tool_library.toolsets", "ツールセット"),
    ("tool_library.toolset.add", "追加"),
    ("tool_library.toolset.duplicate", "複写"),
    ("tool_library.toolset.remove", "削除"),
    ("tool_library.toolset.up", "上へ"),
    ("tool_library.toolset.down", "下へ"),
    ("tool_library.toolset.none", "ツールセットがありません"),
    ("tool_library.toolset.new_name", "新しいツールセット"),
    ("tool_library.toolset.copy_name", "{name} のコピー"),
    ("tool_library.column.name", "名前"),
    ("tool_library.column.kind", "種類"),
    ("tool_library.column.diameter", "径"),
    ("tool_library.column.corner_radius", "コーナー R"),
    ("tool_library.column.cutting_length", "工具長"),
    ("tool_library.kind.ball", "ボールエンドミル"),
    ("tool_library.kind.flat", "フラットエンドミル"),
    ("tool_library.kind.other", "そのほか"),
    ("tool_library.length.unlimited", "制限なし"),
    ("tool_library.edit.name", "名前"),
    ("tool_library.edit.diameter", "径"),
    ("tool_library.edit.specify_length", "工具長を指定する"),
    ("tool_library.edit.apply", "適用"),
    ("tool_library.edit.revert", "元に戻す"),
    ("tool_library.issue.empty_name", "名前を入力してください"),
    (
        "tool_library.issue.diameter",
        "径は 0 より大きい値にしてください",
    ),
    (
        "tool_library.issue.cutting_length",
        "工具長は 0 より大きい値にしてください",
    ),
    ("tool_library.result.created", "{file} を作りました"),
    ("tool_library.result.saved", "ライブラリに書きました"),
    ("tool_library.result.failed", "書けません: {reason}"),
    (
        "tool_library.lock.stale",
        "{owner} の書き込みのロックが残っています。書いている PC が止まった可能性があります。ロックを消して書きますか？",
    ),
];

const TOOL_LIBRARY_TEMPLATE_EN: &[(&str, &str)] = &[
    ("tool_library.window.title", "Tool library"),
    ("tool_library.libraries", "Libraries"),
    ("tool_library.library.create", "New…"),
    ("tool_library.library.open", "Open…"),
    ("tool_library.library.remove", "Remove from list"),
    ("tool_library.library.rename", "Rename"),
    ("tool_library.library.none", "There are no libraries"),
    ("tool_library.library.read_only", "read-only"),
    ("tool_library.library.unreadable", "unreadable"),
    ("tool_library.library.duplicate", "same library"),
    (
        "tool_library.library.read_only.reason",
        "No permission to write to this library, so it is used read-only",
    ),
    (
        "tool_library.library.unreadable.reason",
        "Cannot read: {reason}. The contents read before, if any, are used read-only",
    ),
    (
        "tool_library.library.duplicate.reason",
        "Not used because it is the same library as {first}",
    ),
    ("tool_library.file_type", "Tool library"),
    ("tool_library.toolsets", "Tool sets"),
    ("tool_library.toolset.add", "Add"),
    ("tool_library.toolset.duplicate", "Duplicate"),
    ("tool_library.toolset.remove", "Delete"),
    ("tool_library.toolset.up", "Up"),
    ("tool_library.toolset.down", "Down"),
    ("tool_library.toolset.none", "There are no tool sets"),
    ("tool_library.toolset.new_name", "New tool set"),
    ("tool_library.toolset.copy_name", "Copy of {name}"),
    ("tool_library.column.name", "Name"),
    ("tool_library.column.kind", "Type"),
    ("tool_library.column.diameter", "Diameter"),
    ("tool_library.column.corner_radius", "Corner R"),
    ("tool_library.column.cutting_length", "Length"),
    ("tool_library.kind.ball", "Ball end mill"),
    ("tool_library.kind.flat", "Flat end mill"),
    ("tool_library.kind.other", "Other"),
    ("tool_library.length.unlimited", "Unlimited"),
    ("tool_library.edit.name", "Name"),
    ("tool_library.edit.diameter", "Diameter"),
    (
        "tool_library.edit.specify_length",
        "Specify the tool length",
    ),
    ("tool_library.edit.apply", "Apply"),
    ("tool_library.edit.revert", "Revert"),
    ("tool_library.issue.empty_name", "Enter a name"),
    (
        "tool_library.issue.diameter",
        "The diameter must be greater than 0",
    ),
    (
        "tool_library.issue.cutting_length",
        "The tool length must be greater than 0",
    ),
    ("tool_library.result.created", "Created {file}"),
    ("tool_library.result.saved", "Written to the library"),
    ("tool_library.result.failed", "Cannot write: {reason}"),
    (
        "tool_library.lock.stale",
        "A write lock by {owner} remains. The PC that was writing may have stopped. Remove the lock and write?",
    ),
];

const TOOL_LIBRARY_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    TOOL_LIBRARY_TEMPLATE_JA,
    TOOL_LIBRARY_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

/// 文言のキー `key` の、引数 `args` を当てた文言
pub(crate) fn resolve_tool_library_text(
    locale: UiLocale,
    key: &str,
    args: &[(&str, String)],
) -> String {
    let message = UiMessage {
        key: key.to_string(),
        args: args
            .iter()
            .map(|(name, value)| UiMessageArg {
                name: (*name).to_string(),
                value: value.clone(),
            })
            .collect(),
    };
    resolve_message(&TOOL_LIBRARY_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_has_both_languages() {
        let ja: Vec<&str> = TOOL_LIBRARY_TEMPLATE_JA
            .iter()
            .map(|(key, _)| *key)
            .collect();
        let en: Vec<&str> = TOOL_LIBRARY_TEMPLATE_EN
            .iter()
            .map(|(key, _)| *key)
            .collect();
        assert_eq!(ja, en);
        for key in ja {
            for locale in [UiLocale::Ja, UiLocale::En] {
                assert_ne!(resolve_tool_library_text(locale, key, &[]), "__missing__");
            }
        }
    }
}
