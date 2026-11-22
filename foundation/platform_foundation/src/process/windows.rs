//! Windows の子プロセスの扱い（コンソールのウィンドウを出さない指定と、ジョブオブジェクトの組）

use std::io;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

pub(super) fn hide_console_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

/// ハンドルを閉じると、入れたプロセスを OS が止めるジョブオブジェクト
#[derive(Debug)]
pub(super) struct Group(HANDLE);

// SAFETY: ジョブオブジェクトのハンドルは、どのスレッドから使ってもよい
unsafe impl Send for Group {}
// SAFETY: 同上（ハンドルを書き換えない）
unsafe impl Sync for Group {}

impl Group {
    pub(super) fn new() -> io::Result<Self> {
        // SAFETY: 名前のない、既定の権限のジョブオブジェクトを作る
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        // 設定に失敗したときも、作ったハンドルを閉じる
        let group = Self(handle);
        // SAFETY: すべて 0 の制限の情報は、正しい値（制限なし）
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: 作ったハンドルに、大きさを合わせた制限の情報を渡す
        let set = unsafe {
            SetInformationJobObject(
                group.0,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if set == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(group)
    }

    pub(super) fn add(&self, child: &Child) -> io::Result<()> {
        // SAFETY: 組と、起動した子プロセスのハンドル（`child` を持つ間は正しい）を渡す
        if unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        // SAFETY: 作ったハンドルを 1 度だけ閉じる
        unsafe { CloseHandle(self.0) };
    }
}
