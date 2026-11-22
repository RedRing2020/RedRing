use super::*;

#[cfg(any(windows, target_os = "linux", target_vendor = "apple"))]
#[test]
fn physical_memory_is_read_on_supported_systems() {
    // 試験を動かす PC は、少なくとも 256 MB の物理メモリを持つ
    let bytes = physical_memory_bytes().expect("the physical memory size is known");
    assert!(bytes >= 256 * 1024 * 1024, "{bytes}");
}
