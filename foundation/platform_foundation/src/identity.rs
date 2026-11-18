//! この PC の名前と、サインインしている利用者の名前（共有のファイルを書いている PC・利用者を見分けるため）

#[cfg(not(windows))]
mod other;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
use other as platform;
#[cfg(windows)]
use windows as platform;

#[cfg(test)]
mod tests;

/// この PC の名前
///
/// Windows では環境の変数 `COMPUTERNAME`、ほかの OS では環境の変数 `HOSTNAME`、なければ `/etc/hostname` の中身から
/// 決める。分からないときは `None`。
pub fn computer_name() -> Option<String> {
    platform::computer_name()
}

/// サインインしている利用者の名前
///
/// Windows では環境の変数 `USERNAME`、ほかの OS では環境の変数 `USER` から決める。分からないときは `None`。
pub fn user_name() -> Option<String> {
    platform::user_name()
}

/// 環境の変数 `name` の値（空白だけのとき・ないときは `None`）
fn variable(name: &str) -> Option<String> {
    non_empty(std::env::var(name).ok()?)
}

/// 前後の空白を除いた `value`（空なら `None`）
fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
