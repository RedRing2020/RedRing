//! サインインしたときに起動するプログラムの、Windows の実装（利用者ごとのレジストリの `Run` キー）

use std::io;

use crate::wide;

use windows_sys::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};

/// サインインしたときに起動するプログラムを置くレジストリのキー
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub(super) fn registered_command(name: &str) -> Option<String> {
    let key = wide(RUN_KEY);
    let value = wide(name);
    // 大きさを問い合わせてから読む（読む間に書き替えられて大きくなっていれば、問い合わせ直す）
    loop {
        let mut bytes = 0u32;
        // SAFETY: 大きさだけを問い合わせる
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut bytes,
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let mut buffer = vec![0u16; (bytes as usize).div_ceil(2)];
        // SAFETY: `buffer` の大きさを `bytes` で渡す
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut bytes,
            )
        };
        match status {
            ERROR_SUCCESS => {
                let length = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                return Some(String::from_utf16_lossy(&buffer[..length]));
            }
            ERROR_MORE_DATA => continue,
            _ => return None,
        }
    }
}

pub(super) fn register(name: &str, command: &str) -> io::Result<()> {
    let key = wide(RUN_KEY);
    let value = wide(name);
    let data = wide(command);
    // SAFETY: `data` の大きさ（終わりの 0 を含む）をバイトで渡す
    let status = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            REG_SZ,
            data.as_ptr().cast(),
            (data.len() * 2) as u32,
        )
    };
    registry_result(status)
}

pub(super) fn unregister(name: &str) -> io::Result<()> {
    let key = wide(RUN_KEY);
    let value = wide(name);
    // SAFETY: 引数は終わりが 0 の UTF-16 の文字列
    let status = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), value.as_ptr()) };
    registry_result(status)
}

fn registry_result(status: u32) -> io::Result<()> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status as i32))
    }
}
