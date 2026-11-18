//! 動かす環境（OS など）によって違う処理を、環境に依らない API で使うための薄いラッパー
//!
//! 環境ごとの違いは、このクレートの中に閉じる。対応しない環境での動きは、API ごとにドキュメントコメントに書く。

pub mod autostart;
pub mod file;
pub mod folder;
pub mod identity;
pub mod ipc;
pub mod locale;
pub mod process;

/// 終わりが 0 の UTF-16 の文字列（Windows の API に渡す）
#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
