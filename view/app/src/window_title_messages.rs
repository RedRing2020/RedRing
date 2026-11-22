//! 切削シミュレーションを取り込んだ後のウィンドウのタイトルの文言（日本語・英語）

use viewmodel::message_catalog::{
    TableMessageCatalog, UiLocale, UiMessage, UiMessageArg, resolve_message,
};

const WINDOW_TITLE_TEMPLATE_JA: &[(&str, &str)] = &[
    (
        "title.endpoint",
        "RedRing | 工具: 区間の端点 {endpoint}/{count}（{place}） | 素材: Snapshot {frame}/{frames}（工具の手前） remain={remain}mm3 | {mode} {speed}x",
    ),
    ("title.endpoint.first_start", "最初の区間の始点"),
    ("title.endpoint.segment_end", "区間 {segment} の終点"),
];

const WINDOW_TITLE_TEMPLATE_EN: &[(&str, &str)] = &[
    (
        "title.endpoint",
        "RedRing | Tool: segment endpoint {endpoint}/{count} ({place}) | Stock: Snapshot {frame}/{frames} (before the tool) remain={remain}mm3 | {mode} {speed}x",
    ),
    ("title.endpoint.first_start", "start of the first segment"),
    ("title.endpoint.segment_end", "end of segment {segment}"),
];

const WINDOW_TITLE_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    WINDOW_TITLE_TEMPLATE_JA,
    WINDOW_TITLE_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

pub(crate) fn resolve_window_title_text(
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
    resolve_message(&WINDOW_TITLE_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_japanese_and_english_texts() {
        let ja: Vec<&str> = WINDOW_TITLE_TEMPLATE_JA
            .iter()
            .map(|(key, _)| *key)
            .collect();
        let en: Vec<&str> = WINDOW_TITLE_TEMPLATE_EN
            .iter()
            .map(|(key, _)| *key)
            .collect();
        assert_eq!(ja, en);
    }

    #[test]
    fn endpoint_title_has_every_argument_filled_in() {
        let args = [
            ("endpoint", "1".to_string()),
            ("count", "3".to_string()),
            ("place", "p".to_string()),
            ("frame", "2".to_string()),
            ("frames", "9".to_string()),
            ("remain", "1.000".to_string()),
            ("mode", "PLAY".to_string()),
            ("speed", "2".to_string()),
        ];
        for locale in [UiLocale::Ja, UiLocale::En] {
            let text = resolve_window_title_text(locale, "title.endpoint", &args);
            assert!(
                !text.contains('{') && !text.contains("__missing__"),
                "{text}"
            );
            let text = resolve_window_title_text(
                locale,
                "title.endpoint.segment_end",
                &[("segment", "4".to_string())],
            );
            assert!(text.contains('4') && !text.contains('{'), "{text}");
        }
    }
}
