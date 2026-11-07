//! タスクトレイ（Windows。設計: CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! トレイのアイコンは、見えないウィンドウ（メッセージを受けるだけ）に結び付けて置く。メッセージは、ウィンドウを
//! 出す前は [`dispatch_messages`] のループが、ウィンドウを出した後は eframe（winit）のループが、同じスレッドで配る。
//! メニューは「ウィンドウを開く」「Windows の起動のときに起動する」「終了」。2 つ目の起動は、動いている
//! ジョブマネージャーの見えないウィンドウを、保存先から決まる名前で探し、ウィンドウを開くように頼む。

use std::cell::RefCell;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use eframe::egui::{self, ViewportCommand};
use job_runtime::store::{JobStore, StoredJobStatus};
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, AppendMenuW, CreateIcon, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyMenu, DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW,
    GetWindowThreadProcessId, HICON, MF_CHECKED, MF_SEPARATOR, MF_STRING, MSG, PostMessageW,
    RegisterClassW, RegisterWindowMessageW, SetForegroundWindow, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage, WM_APP, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP,
    WNDCLASSW,
};

use crate::icon::{ICON_SIZE, ring_rgba};
use crate::service::ServiceRequest;

/// 見えないウィンドウの種類の名前
const CLASS_NAME: &str = "RedRingJobManagerTray";
/// トレイのアイコンからのメッセージ
const WM_TRAY: u32 = WM_APP + 1;
/// ウィンドウを開く頼み（2 つ目の起動から）
const WM_OPEN_WINDOW: u32 = WM_APP + 2;
const MENU_OPEN: i32 = 1;
const MENU_AUTOSTART: i32 = 2;
const MENU_QUIT: i32 = 3;
/// 終了のときに、サービスが実行中のランナーを止め終えるのを待つ時間
const QUIT_TIMEOUT: Duration = Duration::from_secs(10);
/// Windows の起動のときに起動するプログラムを置くレジストリのキーと、値の名前
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "RedRing Job Manager";

/// トレイの文言（画面の言語で）
pub(crate) struct TrayTexts {
    pub(crate) title: String,
    pub(crate) open: String,
    pub(crate) autostart: String,
    pub(crate) quit: String,
    pub(crate) quit_confirm: String,
    pub(crate) autostart_failed: String,
}

struct TrayState {
    hwnd: HWND,
    icon: HICON,
    texts: TrayTexts,
    store: JobStore,
    requests: Sender<ServiceRequest>,
    /// Windows の起動のときに起動するコマンド（この実行ファイルの場所が分からなければ `None`）
    autostart_command: Option<String>,
    /// eframe が動いていれば、その画面
    window: Option<egui::Context>,
    /// eframe を始める前に、ウィンドウを開くように頼まれた
    open_requested: bool,
}

thread_local! {
    static STATE: RefCell<Option<TrayState>> = const { RefCell::new(None) };
}

/// エクスプローラーが起動し直したときのメッセージ（トレイのアイコンを置き直す）
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

/// トレイのアイコンを置く（このスレッドでメッセージを配ること）
pub(crate) fn create(
    store: JobStore,
    requests: Sender<ServiceRequest>,
    texts: TrayTexts,
    autostart_command: Option<String>,
) -> std::io::Result<()> {
    let class_name = wide(CLASS_NAME);
    let window_name = wide(&window_name(store.root()));
    // SAFETY: 引数はこの関数の中で生きている、終わりが 0 の UTF-16 の文字列と、既定の値だけ
    let (hwnd, icon) = unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..std::mem::zeroed()
        };
        RegisterClassW(&class);
        TASKBAR_CREATED.store(
            RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()),
            Ordering::Relaxed,
        );
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        (hwnd, create_icon())
    };
    let state = TrayState {
        hwnd,
        icon,
        texts,
        store,
        requests,
        autostart_command,
        window: None,
        open_requested: false,
    };
    add_icon(&state)?;
    STATE.with(|cell| *cell.borrow_mut() = Some(state));
    Ok(())
}

/// トレイのメッセージを配る（eframe のループがないとき）。`until_open` のときは、ウィンドウを開くように頼まれたら
/// 終える（ウィンドウを出さずに起動したとき）。そうでなければ、終了するまで配る（ウィンドウを出せなかったとき）
pub(crate) fn dispatch_messages(until_open: bool) {
    // SAFETY: すべての項目が 0 で正しい値になる C の構造体
    let mut message: MSG = unsafe { std::mem::zeroed() };
    // SAFETY: `message` はこの関数の中で生きている
    while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
        // SAFETY: 受け取ったメッセージを配る
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        // 開く頼みは、配ったメッセージ（トレイ・2 つ目の起動）の中で記録する
        let requested = STATE.with(|cell| {
            cell.borrow()
                .as_ref()
                .is_some_and(|state| state.open_requested)
        });
        if until_open && requested {
            return;
        }
    }
}

/// eframe の画面を結び付ける（以後、「ウィンドウを開く」はこの画面を出す）
pub(crate) fn attach_window(context: egui::Context) {
    STATE.with(|cell| {
        if let Some(state) = cell.borrow_mut().as_mut() {
            state.window = Some(context);
        }
    });
}

/// 保存先 `store_root` のジョブマネージャーが動いていれば、ウィンドウを開くように頼む。頼めたら `true`
pub(crate) fn open_running(store_root: &Path) -> bool {
    let class_name = wide(CLASS_NAME);
    let window_name = wide(&window_name(store_root));
    // SAFETY: 引数は終わりが 0 の UTF-16 の文字列と、見つけたウィンドウ
    unsafe {
        let hwnd = FindWindowW(class_name.as_ptr(), window_name.as_ptr());
        if hwnd.is_null() {
            return false;
        }
        // 起動した利用者の操作で動いているこのプロセスから、前に出す権利を渡す
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        AllowSetForegroundWindow(process_id);
        PostMessageW(hwnd, WM_OPEN_WINDOW, 0, 0) != 0
    }
}

/// 見えないウィンドウの名前（保存先ごとに 1 つのジョブマネージャーのため、保存先から決める）
///
/// 同じフォルダの書き方（大文字・小文字、末尾の区切りなど）で名前が変わらないように、実際のパスに直す。
fn window_name(store_root: &Path) -> String {
    let root = std::fs::canonicalize(store_root).unwrap_or_else(|_| store_root.to_path_buf());
    format!("RedRing Job Manager: {}", root.display())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TRAY => {
            match (lparam & 0xffff) as u32 {
                WM_LBUTTONUP => open_window(),
                WM_RBUTTONUP => show_menu(hwnd),
                _ => {}
            }
            0
        }
        WM_OPEN_WINDOW => {
            open_window();
            0
        }
        _ if message != 0 && message == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            STATE.with(|cell| {
                if let Ok(state) = cell.try_borrow()
                    && let Some(state) = state.as_ref()
                {
                    let _ = add_icon(state);
                }
            });
            0
        }
        // SAFETY: 受け取った引数をそのまま渡す
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

/// ウィンドウを開く（eframe が動いていれば画面を出し、まだなら eframe を始めるように頼む）
fn open_window() {
    STATE.with(|cell| {
        let Ok(mut state) = cell.try_borrow_mut() else {
            return;
        };
        let Some(state) = state.as_mut() else {
            return;
        };
        match &state.window {
            Some(context) => {
                context.send_viewport_cmd(ViewportCommand::Visible(true));
                context.send_viewport_cmd(ViewportCommand::Minimized(false));
                context.send_viewport_cmd(ViewportCommand::Focus);
                context.request_repaint();
            }
            None => state.open_requested = true,
        }
    });
}

/// トレイのメニューを出し、選んだ項目を行う
fn show_menu(hwnd: HWND) {
    // メニュー・確かめの画面は中でメッセージを配るため、状態を借りたままにしない
    let Some((open, autostart, quit_label, command)) = STATE.with(|cell| {
        let state = cell.try_borrow().ok()?;
        let state = state.as_ref()?;
        Some((
            wide(&state.texts.open),
            wide(&state.texts.autostart),
            wide(&state.texts.quit),
            state.autostart_command.clone(),
        ))
    }) else {
        return;
    };
    // 書いたコマンドが、このジョブマネージャーのコマンドと同じときだけチェックを付ける（ほかの保存先・前の場所の
    // ジョブマネージャーのコマンドは、選ぶとこのコマンドで書き直す）
    let autostart_enabled = command.is_some() && registered_autostart_command() == command;
    // SAFETY: 作ったメニューを、この関数の中で消す。文字列は終わりが 0 の UTF-16
    let chosen = unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }
        AppendMenuW(menu, MF_STRING, MENU_OPEN as usize, open.as_ptr());
        let checked = if autostart_enabled { MF_CHECKED } else { 0 };
        AppendMenuW(
            menu,
            MF_STRING | checked,
            MENU_AUTOSTART as usize,
            autostart.as_ptr(),
        );
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(menu, MF_STRING, MENU_QUIT as usize, quit_label.as_ptr());
        let mut cursor = POINT { x: 0, y: 0 };
        GetCursorPos(&mut cursor);
        // 前に出しておかないと、メニューの外を押してもメニューが閉じない
        SetForegroundWindow(hwnd);
        let chosen = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
            cursor.x,
            cursor.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
        chosen
    };
    match chosen {
        MENU_OPEN => open_window(),
        MENU_AUTOSTART => {
            let changed = match (autostart_enabled, &command) {
                (true, _) => disable_autostart(),
                (false, Some(command)) => enable_autostart(command),
                (false, None) => Err(std::io::Error::other("cannot find the executable")),
            };
            if let Err(error) = changed {
                show_message(|texts| {
                    texts
                        .autostart_failed
                        .replace("{reason}", &error.to_string())
                });
            }
        }
        MENU_QUIT => quit(),
        _ => {}
    }
}

/// 終了する（計算中のジョブがあれば確かめ、ランナーを止めてから終える）
fn quit() {
    let Some((store, requests, title, confirm)) = STATE.with(|cell| {
        let state = cell.try_borrow().ok()?;
        let state = state.as_ref()?;
        Some((
            state.store.clone(),
            state.requests.clone(),
            state.texts.title.clone(),
            state.texts.quit_confirm.clone(),
        ))
    }) else {
        return;
    };
    let running = store.load_jobs().is_ok_and(|(jobs, _)| {
        jobs.iter()
            .any(|job| job.status == StoredJobStatus::Running)
    });
    if running {
        let answer = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title(&title)
            .set_description(&confirm)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        if answer != rfd::MessageDialogResult::Yes {
            return;
        }
    }
    let (done_tx, done_rx) = mpsc::channel();
    if requests
        .send(ServiceRequest::Quit { done: done_tx })
        .is_ok()
    {
        let _ = done_rx.recv_timeout(QUIT_TIMEOUT);
    }
    STATE.with(|cell| {
        if let Ok(state) = cell.try_borrow()
            && let Some(state) = state.as_ref()
        {
            remove_icon(state);
        }
    });
    std::process::exit(0);
}

/// トレイの文言で作った誤りの説明を、画面に出す
fn show_message(message: impl FnOnce(&TrayTexts) -> String) {
    let Some((title, description)) = STATE.with(|cell| {
        let state = cell.try_borrow().ok()?;
        let state = state.as_ref()?;
        Some((state.texts.title.clone(), message(&state.texts)))
    }) else {
        return;
    };
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(&title)
        .set_description(&description)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

fn notify_icon_data(state: &TrayState) -> NOTIFYICONDATAW {
    // SAFETY: すべての項目が 0 で正しい値になる C の構造体
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = state.hwnd;
    data.uID = 1;
    data
}

fn add_icon(state: &TrayState) -> std::io::Result<()> {
    let mut data = notify_icon_data(state);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = state.icon;
    let tip = wide(&state.texts.title);
    let length = tip.len().min(data.szTip.len()) - 1;
    data.szTip[..length].copy_from_slice(&tip[..length]);
    // SAFETY: `data` はこの関数の中で生きている
    if unsafe { Shell_NotifyIconW(NIM_ADD, &data) } == 0 {
        return Err(std::io::Error::other("cannot add the tray icon"));
    }
    Ok(())
}

fn remove_icon(state: &TrayState) {
    let data = notify_icon_data(state);
    // SAFETY: `data` はこの関数の中で生きている
    unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
}

/// 赤い輪のアイコン（32 ビットの色と、すべて見せるマスク）
///
/// # Safety
/// Windows の API を呼ぶ
unsafe fn create_icon() -> HICON {
    let rgba = ring_rgba();
    let bgra: Vec<u8> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| [b, g, r, a])
        .collect();
    // マスクの行は 2 バイトの倍数にそろえる（色の透明度で見せ方を決めるため、すべて 0）
    let mask = vec![0u8; (ICON_SIZE.div_ceil(16) * 2 * ICON_SIZE) as usize];
    // SAFETY: 色とマスクは、大きさに合うバイトの列
    unsafe {
        CreateIcon(
            GetModuleHandleW(std::ptr::null()),
            ICON_SIZE as i32,
            ICON_SIZE as i32,
            1,
            32,
            mask.as_ptr(),
            bgra.as_ptr(),
        )
    }
}

/// Windows の起動のときに起動するように書いたコマンド（書いていなければ `None`）
fn registered_autostart_command() -> Option<String> {
    let key = wide(RUN_KEY);
    let value = wide(RUN_VALUE);
    let mut buffer = vec![0u16; 1024];
    let mut bytes = (buffer.len() * 2) as u32;
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
    (status == ERROR_SUCCESS).then(|| {
        let length = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..length])
    })
}

fn enable_autostart(command: &str) -> std::io::Result<()> {
    let key = wide(RUN_KEY);
    let value = wide(RUN_VALUE);
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

fn disable_autostart() -> std::io::Result<()> {
    let key = wide(RUN_KEY);
    let value = wide(RUN_VALUE);
    // SAFETY: 引数は終わりが 0 の UTF-16 の文字列
    let status = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), value.as_ptr()) };
    registry_result(status)
}

fn registry_result(status: u32) -> std::io::Result<()> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(status as i32))
    }
}

/// 終わりが 0 の UTF-16 の文字列
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
