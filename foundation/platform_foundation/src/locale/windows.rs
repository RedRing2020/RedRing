//! 利用者の表示の言語の、Windows の実装

use windows_sys::Win32::Globalization::{GetUserDefaultUILanguage, LCIDToLocaleName};

/// 言語の名前の長さの上限（終わりの 0 を含む。`LOCALE_NAME_MAX_LENGTH`）
const LOCALE_NAME_MAX_LENGTH: usize = 85;

/// 表示の言語（`GetUserDefaultUILanguage`）の、主な言語のコード
pub(super) fn user_language() -> Option<String> {
    // SAFETY: 引数のない、状態を変えない関数
    let language_id = unsafe { GetUserDefaultUILanguage() };
    let mut name = [0u16; LOCALE_NAME_MAX_LENGTH];
    // SAFETY: `name` の長さを渡す（言語の ID は、並べ方の既定の LCID としてそのまま使える）
    let length = unsafe {
        LCIDToLocaleName(
            u32::from(language_id),
            name.as_mut_ptr(),
            name.len() as i32,
            0,
        )
    };
    if length <= 0 {
        return None;
    }
    // 長さは、終わりの 0 を含む
    let name = String::from_utf16_lossy(&name[..(length as usize).saturating_sub(1)]);
    super::primary_language(&name)
}
