//! この PC のメモリ

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

/// この PC の物理メモリの大きさ（バイト）
///
/// Windows では `GlobalMemoryStatusEx` の物理メモリの合計、Linux・macOS では物理メモリのページの数とページの大きさ
/// の積。分からない（ほかの OS を含む）ときは `None`。
pub fn physical_memory_bytes() -> Option<u64> {
    platform::physical_memory_bytes()
}
