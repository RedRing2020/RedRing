//! この PC のメモリの、Windows 以外の実装

/// 物理メモリのページの数とページの大きさの積（Linux・macOS。ほかの OS は `None`）
#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
pub(super) fn physical_memory_bytes() -> Option<u64> {
    // SAFETY: 引数の名前を読むだけの、状態を変えない関数
    let pages = unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) };
    // SAFETY: 同上
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let pages = u64::try_from(pages).ok().filter(|&pages| pages > 0)?;
    let page_size = u64::try_from(page_size).ok().filter(|&size| size > 0)?;
    pages.checked_mul(page_size)
}

/// 物理メモリの大きさ（この OS では分からない）
#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
pub(super) fn physical_memory_bytes() -> Option<u64> {
    None
}
