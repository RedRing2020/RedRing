//! 利用者ごとのフォルダの場所と、利用者だけが読み書きできるこの PC のフォルダの、Windows の実装

use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix};

use windows_sys::Win32::Foundation::{ERROR_SUCCESS, GENERIC_EXECUTE};
use windows_sys::Win32::Security::Authorization::{
    GetNamedSecurityInfoW, SE_FILE_OBJECT, SetNamedSecurityInfoW,
};
use windows_sys::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, DACL_SECURITY_INFORMATION, EqualSid, GetAce,
    GetSecurityDescriptorDacl, IsWellKnownSid, OWNER_SECURITY_INFORMATION,
    PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES,
    WELL_KNOWN_SID_TYPE, WinBuiltinAdministratorsSid, WinCreatorOwnerRightsSid, WinCreatorOwnerSid,
    WinLocalSystemSid,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateDirectoryW, FILE_READ_ATTRIBUTES, FILE_READ_EA, FILE_TRAVERSE, GetDriveTypeW,
    GetVolumeInformationW, GetVolumePathNameW, READ_CONTROL, SYNCHRONIZE,
};
use windows_sys::Win32::System::SystemServices::{
    ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE, ACCESS_DENIED_CALLBACK_ACE_TYPE,
    ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE, ACCESS_DENIED_OBJECT_ACE_TYPE, FILE_PERSISTENT_ACLS,
};
use windows_sys::Win32::System::WindowsProgramming::{
    DRIVE_CDROM, DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOVABLE,
};

use super::PrivateFolderError;
use crate::security::{SecurityDescriptor, UserSid};

/// ほかの利用者・グループに渡っていてもよい権利（たどる・属性と権利を読むだけで、中の名前・中身を読み書きしない）
const HARMLESS_RIGHTS: u32 = FILE_TRAVERSE
    | FILE_READ_ATTRIBUTES
    | FILE_READ_EA
    | READ_CONTROL
    | SYNCHRONIZE
    | GENERIC_EXECUTE;

/// 権利を持っていてよい、この利用者のほかの SID（`SYSTEM`・`Administrators` は、どのフォルダも読み書きできる。
/// `CREATOR OWNER`・`OWNER RIGHTS` は、持ち主（読み書きできる利用者が作ったもの）の権利）
const TRUSTED_SIDS: [WELL_KNOWN_SID_TYPE; 4] = [
    WinLocalSystemSid,
    WinBuiltinAdministratorsSid,
    WinCreatorOwnerSid,
    WinCreatorOwnerRightsSid,
];

/// 持ち主であってよい、この利用者のほかの SID（管理者として作ったフォルダの持ち主は `Administrators` になる）
const TRUSTED_OWNERS: [WELL_KNOWN_SID_TYPE; 2] = [WinLocalSystemSid, WinBuiltinAdministratorsSid];

/// アプリのデータのフォルダ（`%LOCALAPPDATA%`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn data_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("LOCALAPPDATA")
}

/// 設定のフォルダ（`%APPDATA%`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn config_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("APPDATA")
}

/// パス `path` のドライブが、この PC のドライブか（UNC のパス・デバイスのパスは使わない）
pub(super) fn is_on_this_pc(path: &Path) -> bool {
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return false;
    };
    let letter = match prefix.kind() {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => letter,
        // UNC のパス（\\サーバー\共有）・デバイスのパスは使わない
        _ => return false,
    };
    let root = crate::wide(&format!("{}:\\", char::from(letter)));
    // SAFETY: `root` は終わりが 0 の UTF-16 の文字列
    let kind = unsafe { GetDriveTypeW(root.as_ptr()) };
    matches!(
        kind,
        DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_CDROM | DRIVE_RAMDISK
    )
}

/// 実際のパス `path` のフォルダが、利用者だけが読み書きできるこの PC のフォルダか
pub(super) fn check_private_folder(path: &Path) -> Result<(), PrivateFolderError> {
    if !is_on_this_pc(path) {
        return Err(PrivateFolderError::NotOnThisPc);
    }
    if !holds_permissions(path).map_err(PrivateFolderError::Io)? {
        return Err(PrivateFolderError::NoPermissions);
    }
    let user = UserSid::current().map_err(PrivateFolderError::Io)?;
    let mut owner: PSID = std::ptr::null_mut();
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    let name = wide_path(path);
    // SAFETY: `name` は終わりが 0 の UTF-16 の文字列。持ち主と権利の一覧は `descriptor` の中を指し、確かめ終えてから
    // 放す
    let status = unsafe {
        GetNamedSecurityInfoW(
            name.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(PrivateFolderError::Io(io::Error::from_raw_os_error(
            status as i32,
        )));
    }
    let descriptor = SecurityDescriptor::from_raw(descriptor);
    // SAFETY: 持ち主と権利の一覧は、`descriptor` が生きている間だけ読む
    let private = unsafe { is_owner_trusted(owner, &user) && is_dacl_private(dacl, &user) };
    drop(descriptor);
    if private {
        Ok(())
    } else {
        Err(PrivateFolderError::OpenToOthers)
    }
}

/// フォルダ `path` を、利用者だけが読み書きできるフォルダとして作る
pub(super) fn create_private_folder(path: &Path) -> io::Result<()> {
    let security = SecurityDescriptor::from_sddl(&private_sddl()?)?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: security.as_ptr(),
        bInheritHandle: 0,
    };
    let name = wide_path(path);
    // SAFETY: `name` は終わりが 0 の UTF-16 の文字列。属性はこの関数の中で生きている
    if unsafe { CreateDirectoryW(name.as_ptr(), &attributes) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// フォルダ `path` の権利を、利用者だけが読み書きできるようにする
pub(super) fn make_private(path: &Path) -> io::Result<()> {
    set_dacl(path, &private_sddl()?)
}

/// 利用者だけが読み書きできるフォルダの権利（この利用者・`SYSTEM`・`Administrators` だけにすべての権利を渡し、中の
/// ファイル・フォルダにも引き継ぐ。上のフォルダの権利は引き継がない）
fn private_sddl() -> io::Result<String> {
    let user = UserSid::current()?;
    Ok(format!(
        "D:P(A;OICI;FA;;;{})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)",
        user.text
    ))
}

/// フォルダ `path` の権利の一覧を、SDDL の文字列 `sddl` の一覧にする（上のフォルダの権利は引き継がない）
fn set_dacl(path: &Path, sddl: &str) -> io::Result<()> {
    let security = SecurityDescriptor::from_sddl(sddl)?;
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl: *mut ACL = std::ptr::null_mut();
    // SAFETY: 作った記述子から、その中の権利の一覧を指すポインタを読む
    if unsafe {
        GetSecurityDescriptorDacl(security.as_ptr(), &mut present, &mut dacl, &mut defaulted)
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let name = wide_path(path);
    // SAFETY: `name` は終わりが 0 の UTF-16 の文字列。権利の一覧は `security` が生きている間だけ使う
    let status = unsafe {
        SetNamedSecurityInfoW(
            name.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dacl,
            std::ptr::null(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(())
}

/// パス `path` のボリュームのファイルシステムが、権利を持てるか（FAT32・exFAT は持てない）
fn holds_permissions(path: &Path) -> io::Result<bool> {
    let name = wide_path(path);
    let mut volume = vec![0u16; name.len().max(260) + 1];
    // SAFETY: `name` は終わりが 0 の UTF-16 の文字列。`volume` の長さを渡す
    if unsafe { GetVolumePathNameW(name.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut flags = 0u32;
    // SAFETY: `volume` は終わりが 0 の UTF-16 の文字列。読むのはファイルシステムの性質だけ
    if unsafe {
        GetVolumeInformationW(
            volume.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut flags,
            std::ptr::null_mut(),
            0,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(flags & FILE_PERSISTENT_ACLS != 0)
}

/// 持ち主 `owner` が、この利用者 `user`・`SYSTEM`・`Administrators` か
///
/// # Safety
///
/// `owner` は、生きているセキュリティ記述子の中の SID を指す。
unsafe fn is_owner_trusted(owner: PSID, user: &UserSid) -> bool {
    // SAFETY: どちらの SID も生きている
    !owner.is_null() && unsafe { is_user_or(owner, user, &TRUSTED_OWNERS) }
}

/// 権利の一覧 `dacl` が、この利用者 `user`（と、`TRUSTED_SIDS`）のほかに、たどる・属性を読む以外の権利を渡していないか
///
/// 一覧がなければ、だれでも読み書きできる。拒む権利は見ない（渡す権利だけで判断する）。渡す権利のうち、ふつうの形で
/// ないもの（対象の種類・条件を持つもの）は、ほかの利用者に渡しているとみなす。
///
/// # Safety
///
/// `dacl` は、生きているセキュリティ記述子の中の権利の一覧を指す（または空）。
unsafe fn is_dacl_private(dacl: *mut ACL, user: &UserSid) -> bool {
    if dacl.is_null() {
        return false;
    }
    // SAFETY: 権利の一覧は生きている
    let count = unsafe { (*dacl).AceCount };
    for index in 0..u32::from(count) {
        let mut ace: *mut core::ffi::c_void = std::ptr::null_mut();
        // SAFETY: 番号は一覧の数の中
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 {
            return false;
        }
        // SAFETY: どの権利も、始めに `ACE_HEADER` を持つ
        let header = unsafe { *ace.cast::<ACE_HEADER>() };
        let kind = u32::from(header.AceType);
        if [
            ACCESS_DENIED_ACE_TYPE,
            ACCESS_DENIED_OBJECT_ACE_TYPE,
            ACCESS_DENIED_CALLBACK_ACE_TYPE,
            ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE,
        ]
        .contains(&kind)
        {
            continue;
        }
        if kind != ACCESS_ALLOWED_ACE_TYPE {
            return false;
        }
        let allowed = ace.cast::<ACCESS_ALLOWED_ACE>();
        // SAFETY: 渡す権利の形。SID は `SidStart` から始まる
        let (mask, sid) = unsafe {
            (
                (*allowed).Mask,
                std::ptr::addr_of_mut!((*allowed).SidStart).cast::<core::ffi::c_void>(),
            )
        };
        // SAFETY: SID は権利の一覧の中にある
        if mask & !HARMLESS_RIGHTS != 0 && !unsafe { is_user_or(sid, user, &TRUSTED_SIDS) } {
            return false;
        }
    }
    true
}

/// SID `sid` が、この利用者 `user` か、`others` のどれかか
///
/// # Safety
///
/// `sid` は生きている SID を指す。
unsafe fn is_user_or(sid: PSID, user: &UserSid, others: &[WELL_KNOWN_SID_TYPE]) -> bool {
    // SAFETY: どちらの SID も生きている
    let same = unsafe { EqualSid(sid, user.sid()) } != 0;
    // SAFETY: SID は生きている
    same || others
        .iter()
        .any(|&kind| unsafe { IsWellKnownSid(sid, kind) } != 0)
}

/// パス `path` の、終わりが 0 の UTF-16 の文字列
fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// 試験のため、フォルダ `path` の権利の一覧を SDDL の文字列 `sddl`（`{user}` を、この利用者の SID にする）にする
#[cfg(test)]
pub(super) fn set_test_dacl(path: &Path, sddl: &str) -> io::Result<()> {
    set_dacl(path, &sddl.replace("{user}", &UserSid::current()?.text))
}
