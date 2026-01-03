use redring::app::App;

fn main() -> Result<(), winit::error::EventLoopError> {
    // ログシステムを初期化
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("RedRing アプリケーション起動");

    let event_loop = winit::event_loop::EventLoop::builder().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
