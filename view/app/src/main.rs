//! RedRing ビューアプリケーション
//! wgpuとwinitを使用したレンダリングアプリケーション

use redring::app::App;
use redring::logging;

fn main() -> Result<(), winit::error::EventLoopError> {
    // ログシステム初期化
    logging::init_logging();

    // イベントループとアプリケーションを起動
    let event_loop = winit::event_loop::EventLoop::builder().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
