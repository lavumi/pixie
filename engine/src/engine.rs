use instant::Instant;
use wgpu::SurfaceError;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::Window,
    application::ApplicationHandler,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use std::sync::Arc;
use specs::{World, WorldExt};

use crate::application::Application;
use crate::renderer::*;
use crate::config::SCREEN_SIZE;

pub struct Engine<A: Application> {
    app: A,
    world: World,
    rs: RenderState,

    window: Arc<Window>,
    size: PhysicalSize<u32>,

    prev_mouse_position: PhysicalPosition<f64>,
    prev_time: Instant,
}

impl<A: Application> ApplicationHandler<()> for Engine<A> {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // Engine resumed
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if window_id == self.window.id() {
            // Try application input first
            if !self.app.handle_input(&mut self.world, &event) {
                match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        if key_event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
                            && key_event.state == ElementState::Pressed {
                            event_loop.exit();
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.resize(physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        let new_size = self.window.inner_size();
                        self.resize(new_size);
                    }
                    WindowEvent::RedrawRequested => {
                        let elapsed_time = self.prev_time.elapsed().as_millis() as f32 / 1000.0;
                        self.prev_time = Instant::now();

                        if elapsed_time > 0.2 {
                            return;
                        }
                        self.update(elapsed_time);
                        match self.render() {
                            Ok(_) => {}
                            Err(SurfaceError::Lost | SurfaceError::Outdated) => self.rs.resize(self.size),
                            Err(SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            Err(SurfaceError::Other) => log::warn!("Surface error: other"),
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl<A: Application> Engine<A> {
    pub async fn new(
        mut app: A,
        window_attributes: winit::window::WindowAttributes,
        event_loop: &EventLoop<()>) -> Self {
        let window = Arc::new(event_loop
            .create_window(window_attributes)
            .unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            // Canvas setup for web
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wgpu-wasm")?;
                    if let Some(canvas) = window.canvas() {
                        let canvas = web_sys::Element::from(canvas);
                        canvas.set_id("wasm-canvas");
                        dst.append_child(&canvas).ok()?;
                    }
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let size = winit::dpi::PhysicalSize::new(SCREEN_SIZE[0], SCREEN_SIZE[1]);
        let prev_mouse_position = PhysicalPosition::new(0.0, 0.0);
        let prev_time = Instant::now();

        // Initialize World
        let mut world = World::new();

        // Initialize application
        app.init(&mut world);

        // Initialize renderer
        let mut rs = RenderState::new(window.clone()).await;
        rs.init_resources().await;

        Self {
            app,
            world,
            rs,
            window,
            size,
            prev_mouse_position,
            prev_time,
        }
    }

    pub fn get_render_state_mut(&mut self) -> &mut RenderState {
        &mut self.rs
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.rs.resize(new_size);
    }

    fn update(&mut self, dt: f32) {
        self.app.update(&mut self.world, dt);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // 1. Update camera
        let camera_uniform = self.app.get_camera_uniform(&self.world);
        self.rs.update_camera_buffer(camera_uniform);

        // 2. Update meshes
        let instances = self.app.get_tile_instances(&self.world);
        self.rs.update_mesh_instance(instances);

        // 3. Update text
        let text_instances = self.app.get_text_instances(&self.world);
        self.rs.update_text_instance(text_instances);

        self.rs.render()
    }
}
