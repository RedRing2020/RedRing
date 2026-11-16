use super::primary_language;

#[test]
fn the_primary_language_is_taken_from_a_locale_name() {
    assert_eq!(primary_language("ja-JP").as_deref(), Some("ja"));
    assert_eq!(primary_language("ja_JP.UTF-8").as_deref(), Some("ja"));
    assert_eq!(primary_language("en").as_deref(), Some("en"));
    assert_eq!(primary_language("EN-us").as_deref(), Some("en"));
    assert_eq!(primary_language("de_DE@euro").as_deref(), Some("de"));
    for unknown in ["", "C", "C.UTF-8", "POSIX", "1234"] {
        assert_eq!(primary_language(unknown), None, "{unknown:?}");
    }
}

#[test]
fn the_user_language_is_a_lowercase_code_when_known() {
    let language = super::user_language();
    // Windows では、表示の言語がいつも決まっている
    if cfg!(windows) {
        assert!(language.is_some());
    }
    if let Some(language) = language {
        assert!(!language.is_empty());
        assert!(language.bytes().all(|byte| byte.is_ascii_lowercase()));
    }
}
