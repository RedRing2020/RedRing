//! この PC の名前と利用者の名前の、Windows の実装

pub(super) fn computer_name() -> Option<String> {
    super::variable("COMPUTERNAME")
}

pub(super) fn user_name() -> Option<String> {
    super::variable("USERNAME")
}
