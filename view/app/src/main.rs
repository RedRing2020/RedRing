//! RedRing ビューアプリケーション
//! wgpuとwinitを使用したレンダリングアプリケーション

// リリースのビルドではコンソールを開かない（ログはファイルへ、知らせは画面の下の行と OS の知らせの画面へ出す）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use redring::app::App;
use redring::fatal_error_dialog;
use redring::logging;

fn main() -> Result<(), winit::error::EventLoopError> {
    // ログシステム初期化
    logging::init_logging();
    fatal_error_dialog::install_panic_logging();

    // イベントループとアプリケーションを起動
    let event_loop = winit::event_loop::EventLoop::builder().build()?;
    let mut app = App::default();
    // パニックで終わるときは、イベントループを抜けた後に、メインのスレッドで知らせの画面を出す
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        event_loop.run_app(&mut app)
    })) {
        Ok(result) => result,
        Err(payload) => {
            fatal_error_dialog::show_panic_dialog(payload.as_ref());
            std::panic::resume_unwind(payload)
        }
    }
}
