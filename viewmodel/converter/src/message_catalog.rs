pub use i18n_foundation::{
    resolve_message, MessageCatalog, MessageTemplateTable, TableMessageCatalog, UiLocale,
    UiMessage, UiMessageArg,
};

#[cfg(test)]
mod tests {
    use super::{resolve_message, TableMessageCatalog, UiLocale, UiMessage, UiMessageArg};

    const DEMO_JA: &[(&str, &str)] = &[
        ("demo.hello", "こんにちは {name}"),
        ("demo.done", "完了しました"),
    ];
    const DEMO_EN: &[(&str, &str)] = &[("demo.hello", "Hello {name}"), ("demo.done", "Done")];

    #[test]
    fn table_message_catalog_resolves_by_locale() {
        let catalog = TableMessageCatalog::new(DEMO_JA, DEMO_EN, "不明", "Unknown");
        let message = UiMessage {
            key: "demo.hello".to_string(),
            args: vec![UiMessageArg {
                name: "name".to_string(),
                value: "RedRing".to_string(),
            }],
        };

        assert_eq!(
            resolve_message(&catalog, UiLocale::Ja, &message),
            "こんにちは RedRing"
        );
        assert_eq!(
            resolve_message(&catalog, UiLocale::En, &message),
            "Hello RedRing"
        );
    }

    #[test]
    fn table_message_catalog_uses_fallback() {
        let catalog = TableMessageCatalog::new(DEMO_JA, DEMO_EN, "不明", "Unknown");
        let message = UiMessage {
            key: "demo.missing".to_string(),
            args: Vec::new(),
        };

        assert_eq!(resolve_message(&catalog, UiLocale::Ja, &message), "不明");
        assert_eq!(resolve_message(&catalog, UiLocale::En, &message), "Unknown");
    }
}
