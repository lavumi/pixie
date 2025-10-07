use instant::Instant;
use wgpu::SurfaceError;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::Window,
    application::ApplicationHandler,
    dpi::PhysicalSize,
};
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
// use winit::dpi::PhysicalPosition; // no longer needed
use std::sync::Arc;
use std::collections::HashMap;
use specs::{World, WorldExt};

use crate::application::Application;
use crate::renderer::*;
use crate::dispatcher::UnifiedDispatcher;
use crate::resources::DeltaTime;
use pollster::block_on;

pub struct Engine<A: Application> {
    app: A,
    world: World,
    rs: Option<RenderState>,
    dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    window: Option<Arc<Window>>,
    size: PhysicalSize<u32>,
    prev_time: Instant,
    accumulator: f32,
    fixed_dt: f32,

    // bootstrap config for ActiveEventLoop window creation
    title: String,
    initial_width: u32,
    initial_height: u32,
    textures_to_load: Option<HashMap<String, &'static [u8]>>,
}

impl<A: Application> ApplicationHandler<()> for Engine<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            #[cfg(target_arch = "wasm32")]
            let window_attributes = Window::default_attributes()
                .with_title(self.title.clone())
                .with_inner_size(PhysicalSize::new(self.initial_width, self.initial_height));

            #[cfg(not(target_arch = "wasm32"))]
            let window_attributes = Window::default_attributes()
                .with_title(self.title.clone())
                .with_inner_size(LogicalSize::new(self.initial_width, self.initial_height));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            #[cfg(target_arch = "wasm32")]
            {
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

            let mut rs = block_on(RenderState::new(window.clone(), self.initial_width, self.initial_height));
            block_on(rs.init_resources());

            if let Some(textures) = self.textures_to_load.take() {
                for (name, image_bytes) in textures {
                    rs.load_texture_atlas(&name, image_bytes);
                }
            }

            self.size = window.inner_size();
            self.window = Some(window);
            self.rs = Some(rs);
        }
        if let Some(w) = &self.window { w.request_redraw(); }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(w) = &self.window { w.request_redraw(); }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = &self.window { if window_id == window.id() {
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
                        let new_size = window.inner_size();
                        self.resize(new_size);
                    }
                    WindowEvent::RedrawRequested => {
                        let mut elapsed_time = self.prev_time.elapsed().as_millis() as f32 / 1000.0;
                        self.prev_time = Instant::now();

                        if elapsed_time > 0.2 { elapsed_time = 0.2; }

                        // Accumulate time for fixed-step updates
                        self.accumulator += elapsed_time;
                        while self.accumulator >= self.fixed_dt {
                            // Provide fixed dt to systems and step dispatcher
                            {
                                let mut delta = self.world.write_resource::<DeltaTime>();
                                *delta = DeltaTime(self.fixed_dt);
                            }
                            self.dispatcher.run_now(&mut self.world);
                            self.world.maintain();
                            self.accumulator -= self.fixed_dt;
                        }

                        // Variable-step update for non-physics
                        self.update(elapsed_time);

                        match self.render() {
                            Ok(_) => {}
                            Err(SurfaceError::Lost | SurfaceError::Outdated) => if let Some(rs) = &mut self.rs { rs.resize(self.size); },
                            Err(SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            Err(SurfaceError::Other) => log::warn!("Surface error: other"),
                        }
                    }
                    _ => {}
                }
            }
        }}
    }
}

impl<A: Application> Engine<A> {
    pub async fn new<T: Into<String>>(
        mut app: A,
        title: T,
        width: u32,
        height: u32,
        dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    ) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new().unwrap();

        let size = PhysicalSize::new(width, height);
        let prev_time = Instant::now();
        let accumulator = 0.0f32;
        let fixed_dt = 1.0 / 60.0; // 60 Hz physics

        // Initialize World
        let mut world = World::new();

        // Initialize application
        app.init(&mut world);



        let engine = Self {
            app,
            world,
            rs: None,
            dispatcher,
            window: None,
            size,
            prev_time,
            accumulator,
            fixed_dt,
            title: title.into(),
            initial_width: width,
            initial_height: height,
            textures_to_load: None,
        };
        
        (engine, event_loop)
    }

    /// Load multiple textures at once
    pub fn load_textures(&mut self, textures: HashMap<String, &'static [u8]>) {
        for (name, image_bytes) in textures {
            if let Some(rs) = &mut self.rs { rs.load_texture_atlas(&name, image_bytes); }
        }
    }

    /// Start the engine - creates and runs in one go
    pub async fn start<T: Into<String>>(
        app: A,
        title: T,
        width: u32,
        height: u32,
        textures: Option<HashMap<String, &'static [u8]>>,
        dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    ) {
        let event_loop = EventLoop::new().unwrap();

        // Initialize World
        let mut world = World::new();

        // Initialize application
        let mut app = app;
        app.init(&mut world);

        let mut engine = Self {
            app,
            world,
            rs: None,
            dispatcher,
            window: None,
            size: PhysicalSize::new(width, height),
            prev_time: Instant::now(),
            accumulator: 0.0,
            fixed_dt: 1.0 / 60.0,
            title: title.into(),
            initial_width: width,
            initial_height: height,
            textures_to_load: textures,
        };

        event_loop.run_app(&mut engine).unwrap();
    }

    /// Run the engine with the provided event loop
    pub fn run_with_event_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run_app(&mut self).unwrap();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        if let Some(rs) = &mut self.rs { rs.resize(new_size); }
    }

    fn update(&mut self, dt: f32) {
        self.app.update(&mut self.world, dt);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let rs = match &mut self.rs { Some(rs) => rs, None => return Ok(()) };
        // 1. Update camera
        let camera_uniform = self.app.get_camera_uniform(&self.world);
        rs.update_camera_buffer(camera_uniform);

        // 2. Update meshes
        let instances = self.app.get_tile_instances(&self.world);
        rs.update_mesh_instance(instances);

        // 3. Update text
        let text_instances = self.app.get_text_instances(&self.world);
        rs.update_text_instance(text_instances);

        rs.render()
    }
}
