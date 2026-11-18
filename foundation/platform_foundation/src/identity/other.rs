//! この PC の名前と利用者の名前の、Windows 以外の実装

pub(super) fn computer_name() -> Option<String> {
    super::variable("HOSTNAME").or_else(|| {
        std::fs::read_to_string("/etc/hostname")
            .ok()
            .and_then(super::non_empty)
    })
}

pub(super) fn user_name() -> Option<String> {
    super::variable("USER")
}
