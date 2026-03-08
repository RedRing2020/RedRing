#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMessageArg {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMessage {
    pub key: String,
    pub args: Vec<UiMessageArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLocale {
    Ja,
    En,
}

pub type MessageTemplateTable = &'static [(&'static str, &'static str)];

/// 文言キーをテンプレートへ解決する抽象。
/// ドメインごとに実装を差し替えることで再利用可能にする。
pub trait MessageCatalog {
    fn template(&self, locale: UiLocale, key: &str) -> &'static str;
}

/// ロケールごとの静的テーブルで解決する汎用カタログ実装。
/// Job 以外のドメインでも、テーブル定義だけで再利用できる。
pub struct TableMessageCatalog {
    ja: MessageTemplateTable,
    en: MessageTemplateTable,
    fallback_ja: &'static str,
    fallback_en: &'static str,
}

impl TableMessageCatalog {
    pub const fn new(
        ja: MessageTemplateTable,
        en: MessageTemplateTable,
        fallback_ja: &'static str,
        fallback_en: &'static str,
    ) -> Self {
        Self {
            ja,
            en,
            fallback_ja,
            fallback_en,
        }
    }

    fn lookup(table: MessageTemplateTable, key: &str) -> Option<&'static str> {
        table
            .iter()
            .find(|(candidate, _)| *candidate == key)
            .map(|(_, template)| *template)
    }
}

impl MessageCatalog for TableMessageCatalog {
    fn template(&self, locale: UiLocale, key: &str) -> &'static str {
        match locale {
            UiLocale::Ja => Self::lookup(self.ja, key).unwrap_or(self.fallback_ja),
            UiLocale::En => Self::lookup(self.en, key).unwrap_or(self.fallback_en),
        }
    }
}

pub fn resolve_message<C: MessageCatalog>(
    catalog: &C,
    locale: UiLocale,
    message: &UiMessage,
) -> String {
    let template = catalog.template(locale, &message.key);

    let mut resolved = template.to_string();
    for arg in &message.args {
        resolved = resolved.replace(&format!("{{{}}}", arg.name), &arg.value);
    }
    resolved
}

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
