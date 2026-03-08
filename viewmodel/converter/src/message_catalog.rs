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

/// 文言キーをテンプレートへ解決する抽象。
/// ドメインごとに実装を差し替えることで再利用可能にする。
pub trait MessageCatalog {
    fn template(&self, locale: UiLocale, key: &str) -> &'static str;
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
