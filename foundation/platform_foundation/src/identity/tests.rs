use super::*;

#[test]
fn values_are_trimmed_and_empty_values_are_none() {
    assert_eq!(non_empty("  pc-a \n".to_string()), Some("pc-a".to_string()));
    assert_eq!(non_empty(" \t\n".to_string()), None);
}

#[test]
fn the_names_are_not_blank_when_known() {
    for name in [computer_name(), user_name()].into_iter().flatten() {
        assert!(!name.trim().is_empty());
        assert_eq!(name, name.trim());
    }
}
