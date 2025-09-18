use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::graphic::Graphic;

pub struct App<'a> {
    window: &'a Window,
    graphic: Graphic<'a>,
}

impl<'a> App<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let graphic = Graphic::new(window).await;
        Self { window, graphic }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.graphic.render(),
            _ => {}
        }
    }
}