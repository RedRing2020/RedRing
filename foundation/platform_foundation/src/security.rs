//! 利用者の SID とセキュリティ記述子（Windows）

use std::io;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, GetLastError, HANDLE, LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, PSID, TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// この利用者の SID
pub(crate) struct UserSid {
    /// `TOKEN_USER` を入れた領域（SID を指すため、比べ終えるまで持つ）
    token_user: Vec<u64>,
    /// SID の文字列（`S-1-5-…`）
    pub(crate) text: String,
}

impl UserSid {
    pub(crate) fn current() -> io::Result<Self> {
        let mut token: HANDLE = std::ptr::null_mut();
        // SAFETY: 自分のプロセスの擬似ハンドルから、問い合わせだけのトークンを開く
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = Self::from_token(token);
        // SAFETY: 開いたトークンを閉じる
        unsafe { CloseHandle(token) };
        result
    }

    fn from_token(token: HANDLE) -> io::Result<Self> {
        let mut length = 0u32;
        // SAFETY: 大きさだけを問い合わせる
        let sized =
            unsafe { GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut length) };
        // SAFETY: 直前の呼び出しの誤りを読む
        if sized == 0 && unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER {
            return Err(io::Error::last_os_error());
        }
        let mut token_user = vec![0u64; (length as usize).div_ceil(8)];
        // SAFETY: `token_user` は `length` バイト以上あり、8 バイトにそろっている
        if unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                token_user.as_mut_ptr().cast(),
                length,
                &mut length,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut sid = Self {
            token_user,
            text: String::new(),
        };
        let mut text: *mut u16 = std::ptr::null_mut();
        // SAFETY: SID は `token_user` の中にある。書いた文字列は、読み終えてから放す
        unsafe {
            if ConvertSidToStringSidW(sid.sid(), &mut text) == 0 {
                return Err(io::Error::last_os_error());
            }
            let length = (0..).take_while(|&i| *text.add(i) != 0).count();
            sid.text = String::from_utf16_lossy(std::slice::from_raw_parts(text, length));
            LocalFree(text.cast());
        }
        Ok(sid)
    }

    pub(crate) fn sid(&self) -> PSID {
        // SAFETY: `token_user` には、`GetTokenInformation` が書いた `TOKEN_USER` がある
        unsafe { (*self.token_user.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }
}

/// `LocalFree` で放す、セキュリティ記述子
pub(crate) struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

// SAFETY: 作った後は読むだけで、どのスレッドからでも使える
unsafe impl Send for SecurityDescriptor {}

impl SecurityDescriptor {
    /// SDDL の文字列 `text` から作る
    pub(crate) fn from_sddl(text: &str) -> io::Result<Self> {
        let text = crate::wide(text);
        let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        // SAFETY: `text` は終わりが 0 の UTF-16 の文字列
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }

    /// OS が `LocalFree` で放す領域に書いた記述子 `descriptor` を持つ（放してよいときだけ使う）
    pub(crate) fn from_raw(descriptor: PSECURITY_DESCRIPTOR) -> Self {
        Self(descriptor)
    }

    pub(crate) fn as_ptr(&self) -> PSECURITY_DESCRIPTOR {
        self.0
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: OS が `LocalFree` で放す領域に書いたものを放す
        unsafe { LocalFree(self.0) };
    }
}
