//! 名前付きパイプの口（設計: JOB_IPC_DESIGN.md 3.1）
//!
//! パイプの名前は利用者の SID から決め、接続できるのをその利用者だけにし（持ち主もその利用者にする）、ネットワーク
//! からの接続を拒む。最初のインスタンスは `FILE_FLAG_FIRST_PIPE_INSTANCE` で作る（ほかのプロセスが先に作っていれば
//! 開かない）。つなぐ側は、パイプの持ち主が自分であることを確かめる。つなぐ側の接続は非同期の読み書き（overlapped）で
//! 開き、読み書きに時間の上限を設けられる。

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::time::Duration;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_BROKEN_PIPE, ERROR_INSUFFICIENT_BUFFER,
    ERROR_IO_PENDING, ERROR_NO_DATA, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, ERROR_SUCCESS,
    GetLastError, HANDLE, INVALID_HANDLE_VALUE, LocalFree, WAIT_OBJECT_0,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, GetSecurityInfo,
    SDDL_REVISION_1, SE_KERNEL_OBJECT,
};
use windows_sys::Win32::Security::{
    EqualSid, GetTokenInformation, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
    SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX, ReadFile,
    SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT, WriteFile,
};
use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeServerProcessId, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, GetCurrentProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
};
use windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;

use super::MAX_CONNECTIONS;

/// パイプの名前の始まり
const PIPE_PREFIX: &str = r"\\.\pipe\RedRing.JobManager.";
/// パイプの送り・受けの大きさ
const PIPE_BUFFER_BYTES: u32 = 64 * 1024;
/// 同時につないでいる数が上限のときに、空くのを待つ間隔
const BUSY_INTERVAL: Duration = Duration::from_millis(50);

/// パイプの接続（つなぐ側は、非同期の読み書きで時間の上限を設けられる）
#[derive(Debug)]
pub(super) struct Stream {
    file: File,
    /// つなぐ側の、非同期の読み書き（受ける側は `None` で、同期に読み書きする）
    overlapped: Option<OverlappedIo>,
}

/// 非同期の読み書きの、終わりを知らせるイベントと、時間の上限
#[derive(Debug)]
struct OverlappedIo {
    event: Event,
    timeout: Option<Duration>,
}

/// 閉じるイベントのハンドル
struct Event(HANDLE);

// SAFETY: イベントのハンドルは、どのスレッドからでも使える
unsafe impl Send for Event {}
// SAFETY: イベントのハンドルは、複数のスレッドから同時に使ってよい
unsafe impl Sync for Event {}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Event")
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        // SAFETY: 作ったイベントを閉じる
        unsafe { CloseHandle(self.0) };
    }
}

impl OverlappedIo {
    fn new() -> io::Result<Self> {
        // SAFETY: 手で戻す（manual reset）、名前のないイベントを作る
        let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
        if event.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            event: Event(event),
            timeout: None,
        })
    }

    /// `file` で、`buffer`（`length` バイト）を読む・書く。時間の上限を超えたら取り消して `TimedOut`。読むときに
    /// 相手が閉じていれば 0
    fn transfer(
        &self,
        file: &File,
        buffer: *mut u8,
        length: usize,
        read: bool,
    ) -> io::Result<usize> {
        let handle = file.as_raw_handle();
        let length = length.min(u32::MAX as usize) as u32;
        // SAFETY: すべての項目が 0 で正しい値になる C の構造体
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = self.event.0;
        // SAFETY: バッファと OVERLAPPED は、読み書きが終わる（または取り消して終わりを待つ）まで生きている
        let started = unsafe {
            if read {
                ReadFile(
                    handle,
                    buffer,
                    length,
                    std::ptr::null_mut(),
                    &mut overlapped,
                )
            } else {
                WriteFile(
                    handle,
                    buffer,
                    length,
                    std::ptr::null_mut(),
                    &mut overlapped,
                )
            }
        };
        if started == 0 {
            // SAFETY: 直前の呼び出しの誤りを読む
            let error = unsafe { GetLastError() };
            if error != ERROR_IO_PENDING {
                return Self::finish_error(error, read);
            }
        }
        let milliseconds = self.timeout.map_or(INFINITE, |timeout| {
            timeout.as_millis().min(u128::from(INFINITE - 1)) as u32
        });
        let mut transferred = 0u32;
        // SAFETY: 始めた読み書きのイベントを待つ
        if unsafe { WaitForSingleObject(self.event.0, milliseconds) } != WAIT_OBJECT_0 {
            // 時間の上限を超えた・待てなかったときは、始めた読み書きを取り消し、終わるまで待つ（バッファと OVERLAPPED
            // を放す前に）
            // SAFETY: 始めた読み書きのハンドルと OVERLAPPED
            unsafe {
                CancelIoEx(handle, &overlapped);
                GetOverlappedResult(handle, &overlapped, &mut transferred, 1);
            }
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the job manager did not answer in time",
            ));
        }
        // SAFETY: 終わった読み書きの結果を読む
        if unsafe { GetOverlappedResult(handle, &overlapped, &mut transferred, 0) } == 0 {
            // SAFETY: 直前の呼び出しの誤りを読む
            return Self::finish_error(unsafe { GetLastError() }, read);
        }
        Ok(transferred as usize)
    }

    fn finish_error(error: u32, read: bool) -> io::Result<usize> {
        if read && error == ERROR_BROKEN_PIPE {
            Ok(0)
        } else {
            Err(io::Error::from_raw_os_error(error as i32))
        }
    }
}

impl Stream {
    pub(super) fn set_io_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        match self.overlapped.as_mut() {
            Some(io) => {
                io.timeout = timeout;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "only a connecting side can set a timeout",
            )),
        }
    }
}

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match &self.overlapped {
            Some(io) => io.transfer(&self.file, buffer.as_mut_ptr(), buffer.len(), true),
            None => self.file.read(buffer),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match &self.overlapped {
            // 書くときはバッファを読むだけ
            Some(io) => io.transfer(&self.file, buffer.as_ptr().cast_mut(), buffer.len(), false),
            None => self.file.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// パイプの名前
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Address(String);

impl Address {
    pub(super) fn for_current_user() -> io::Result<Self> {
        Ok(Self(format!("{PIPE_PREFIX}{}", UserSid::current()?.text)))
    }

    pub(super) fn with_name(name: &str) -> io::Result<Self> {
        Ok(Self(format!(
            "{PIPE_PREFIX}{}.{name}",
            UserSid::current()?.text
        )))
    }

    fn wide(&self) -> Vec<u16> {
        self.0.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

/// この利用者の SID
struct UserSid {
    /// `TOKEN_USER` を入れた領域（SID を指すため、比べ終えるまで持つ）
    token_user: Vec<u64>,
    text: String,
}

impl UserSid {
    fn current() -> io::Result<Self> {
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

    fn sid(&self) -> PSID {
        // SAFETY: `token_user` には、`GetTokenInformation` が書いた `TOKEN_USER` がある
        unsafe { (*self.token_user.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }
}

/// `LocalFree` で放す、セキュリティ記述子
struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

// SAFETY: 作った後は読むだけで、どのスレッドからでも使える
unsafe impl Send for SecurityDescriptor {}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: `ConvertStringSecurityDescriptorToSecurityDescriptorW` が作ったものを放す
        unsafe { LocalFree(self.0) };
    }
}

/// 開いたパイプ（次の接続を待つインスタンスを持つ）
pub(super) struct Listener {
    name: Vec<u16>,
    security: SecurityDescriptor,
    /// 次の接続を待つインスタンス（最初のインスタンス。受けたら、次の接続のときに作る）
    next: Option<File>,
}

impl std::fmt::Debug for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listener")
            .field("name", &String::from_utf16_lossy(&self.name))
            .finish_non_exhaustive()
    }
}

impl Listener {
    pub(super) fn bind(address: &Address) -> io::Result<Self> {
        let sid = UserSid::current()?;
        // 持ち主をこの利用者にし、この利用者だけにすべての権利を渡す（親の権利を引き継がない）
        let text: Vec<u16> = format!("O:{0}D:P(A;;GA;;;{0})", sid.text)
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
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
        let mut listener = Self {
            name: address.wide(),
            security: SecurityDescriptor(descriptor),
            next: None,
        };
        listener.next = Some(listener.create_instance(true).map_err(|error| {
            // 最初のインスタンスを作れないのは、ほかのプロセスが同じ名前のパイプを開いているとき
            if error.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) {
                io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "the job manager endpoint is already open",
                )
            } else {
                error
            }
        })?);
        Ok(listener)
    }

    fn create_instance(&self, first: bool) -> io::Result<File> {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.security.0,
            bInheritHandle: 0,
        };
        let first_flag = if first {
            FILE_FLAG_FIRST_PIPE_INSTANCE
        } else {
            0
        };
        // SAFETY: 名前は終わりが 0 の UTF-16 の文字列。属性はこの関数の中で生きている
        let handle = unsafe {
            CreateNamedPipeW(
                self.name.as_ptr(),
                PIPE_ACCESS_DUPLEX | first_flag,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                MAX_CONNECTIONS as u32,
                PIPE_BUFFER_BYTES,
                PIPE_BUFFER_BYTES,
                0,
                &attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: 作ったハンドルを、閉じる役目ごと渡す
        Ok(unsafe { File::from_raw_handle(handle) })
    }

    /// 次の接続を待つインスタンス（なければ作る。同時につないでいる数が上限のときは、空くまで待つ）
    fn waiting_instance(&mut self) -> io::Result<File> {
        if let Some(instance) = self.next.take() {
            return Ok(instance);
        }
        loop {
            match self.create_instance(false) {
                Ok(instance) => return Ok(instance),
                Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) => {
                    std::thread::sleep(BUSY_INTERVAL);
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// 接続を待って受ける。受けたら、次の接続を待つインスタンスをすぐに作る（パイプの名前を、インスタンスのない
    /// 間にほかのプロセスに取られないように）
    pub(super) fn accept(&mut self) -> io::Result<Stream> {
        loop {
            let instance = self.waiting_instance()?;
            // SAFETY: 自分で作ったパイプのインスタンスで、接続を待つ（同期）
            let connected =
                unsafe { ConnectNamedPipe(instance.as_raw_handle(), std::ptr::null_mut()) } != 0;
            // SAFETY: 直前の呼び出しの誤りを読む
            let error = if connected {
                0
            } else {
                unsafe { GetLastError() }
            };
            // 待ち始める前につながっていたときも、接続を受けたとする
            if connected || error == ERROR_PIPE_CONNECTED {
                self.next = self.create_instance(false).ok();
                return Ok(Stream {
                    file: instance,
                    overlapped: None,
                });
            }
            // つないですぐに切った接続は、新しいインスタンスを作ってから捨て、待ち直す
            let replacement = self.create_instance(false)?;
            drop(instance);
            self.next = Some(replacement);
            if error != ERROR_NO_DATA {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
        }
    }
}

pub(super) fn connect(address: &Address) -> io::Result<Stream> {
    // パイプの側に、こちらの身元を確かめる以上の権利（なりすまし）を渡さない。時間の上限を設けられるよう、非同期の
    // 読み書きで開く
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .security_qos_flags(SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION)
        .custom_flags(FILE_FLAG_OVERLAPPED)
        .open(&address.0)
        .map_err(|error| {
            if error.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) {
                io::Error::new(io::ErrorKind::ResourceBusy, error)
            } else {
                error
            }
        })?;
    if !owned_by_current_user(&file)? {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the job manager endpoint is owned by another user",
        ));
    }
    Ok(Stream {
        file,
        overlapped: Some(OverlappedIo::new()?),
    })
}

pub(super) fn allow_server_foreground(stream: &Stream) {
    let mut process_id = 0u32;
    // SAFETY: つないだパイプのハンドルから、パイプを開いたプロセスの番号を読む
    if unsafe { GetNamedPipeServerProcessId(stream.file.as_raw_handle(), &mut process_id) } != 0 {
        // SAFETY: 番号を渡すだけ（渡せなくても、ウィンドウは開く）
        unsafe { AllowSetForegroundWindow(process_id) };
    }
}

/// つないだパイプの持ち主が、この利用者か
fn owned_by_current_user(file: &File) -> io::Result<bool> {
    let sid = UserSid::current()?;
    let mut owner: PSID = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SAFETY: つないだパイプのハンドルの、持ち主だけを読む。読んだ記述子は、比べ終えてから放す
    let status = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_KERNEL_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &mut owner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let descriptor = SecurityDescriptor(descriptor);
    // SAFETY: どちらの SID も、持ち主の領域が生きている間に比べる
    let same = unsafe { EqualSid(owner, sid.sid()) } != 0;
    drop(descriptor);
    Ok(same)
}
