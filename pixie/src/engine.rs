use hecs::World;
use instant::Instant;
use std::fmt;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, dpi::PhysicalSize, event::*,
    event_loop::EventLoop, window::Window,
};

use crate::application::Application;
use crate::dispatcher::UnifiedDispatcher;
use crate::renderer::*;
use crate::resources::{DeltaTime, ResourceContainer};
use crate::{TextureAtlasAsset, TextureAtlasRegistry};
#[cfg(not(target_arch = "wasm32"))]
use pollster::block_on;

#[derive(Debug)]
pub enum EngineError {
    Atlas(crate::AtlasError),
    EventLoop(winit::error::EventLoopError),
    Render(RenderError),
    Startup(String),
    Window(winit::error::OsError),
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atlas(error) => error.fmt(formatter),
            Self::EventLoop(error) => write!(formatter, "event loop failed: {error}"),
            Self::Render(error) => error.fmt(formatter),
            Self::Startup(message) => formatter.write_str(message),
            Self::Window(error) => write!(formatter, "window creation failed: {error}"),
        }
    }
}

impl std::error::Error for EngineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Atlas(error) => Some(error),
            Self::EventLoop(error) => Some(error),
            Self::Render(error) => Some(error),
            Self::Startup(_) => None,
            Self::Window(error) => Some(error),
        }
    }
}

impl From<crate::AtlasError> for EngineError {
    fn from(value: crate::AtlasError) -> Self {
        Self::Atlas(value)
    }
}

impl From<winit::error::EventLoopError> for EngineError {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoop(value)
    }
}

impl From<RenderError> for EngineError {
    fn from(value: RenderError) -> Self {
        Self::Render(value)
    }
}

impl From<winit::error::OsError> for EngineError {
    fn from(value: winit::error::OsError) -> Self {
        Self::Window(value)
    }
}

pub struct Engine<A: Application> {
    app: A,
    world: World,
    resources: ResourceContainer,
    rs: Option<RenderState>,
    render_extractor: RenderWorldExtractor,
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
    fatal_error: Option<EngineError>,
    #[cfg(target_arch = "wasm32")]
    wasm_pending_rs:
        Option<std::rc::Rc<std::cell::RefCell<Option<Result<RenderState, RenderError>>>>>,
}

impl<A: Application> ApplicationHandler<()> for Engine<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(self.title.clone())
                .with_inner_size(LogicalSize::new(self.initial_width, self.initial_height));

            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => Arc::new(window),
                Err(error) => {
                    self.exit_with_error(event_loop, error.into());
                    return;
                }
            };

            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowExtWebSys;
                let canvas_attached =
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
                        });
                if canvas_attached.is_none() {
                    self.exit_with_error(
                        event_loop,
                        EngineError::Startup(
                            "failed to append the rendering canvas to '#wgpu-wasm'".to_string(),
                        ),
                    );
                    return;
                }

                use std::cell::RefCell;
                use std::rc::Rc;

                let pending: Rc<RefCell<Option<Result<RenderState, RenderError>>>> =
                    Rc::new(RefCell::new(None));
                let pending_clone = Rc::clone(&pending);
                let window_clone = window.clone();
                let w = self.initial_width;
                let h = self.initial_height;
                wasm_bindgen_futures::spawn_local(async move {
                    let result = async {
                        let mut rs = RenderState::new(window_clone, w, h).await?;
                        rs.init_resources().await?;
                        Ok(rs)
                    }
                    .await;
                    *pending_clone.borrow_mut() = Some(result);
                });

                self.wasm_pending_rs = Some(pending);
                self.size = PhysicalSize::new(self.initial_width, self.initial_height);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut rs = match block_on(RenderState::new(
                    window.clone(),
                    self.initial_width,
                    self.initial_height,
                )) {
                    Ok(rs) => rs,
                    Err(error) => {
                        self.exit_with_error(event_loop, error.into());
                        return;
                    }
                };
                if let Err(error) = block_on(rs.init_resources()) {
                    self.exit_with_error(event_loop, error.into());
                    return;
                }
                if let Err(error) = Self::upload_pending_atlases(&mut self.resources, &mut rs) {
                    self.exit_with_error(event_loop, error.into());
                    return;
                }
                self.rs = Some(rs);
                self.size = window.inner_size();
            }

            self.window = Some(window);
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = &self.window {
            if window_id == window.id() {
                // Try application input first
                if !self
                    .app
                    .handle_input(&mut self.world, &mut self.resources, &event)
                {
                    match event {
                        WindowEvent::CloseRequested => event_loop.exit(),
                        WindowEvent::KeyboardInput {
                            event: key_event, ..
                        } if key_event.physical_key
                            == winit::keyboard::PhysicalKey::Code(
                                winit::keyboard::KeyCode::Escape,
                            )
                            && key_event.state == ElementState::Pressed =>
                        {
                            event_loop.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            self.resize(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            let new_size = window.inner_size();
                            self.resize(new_size);
                        }
                        WindowEvent::RedrawRequested => {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // If async renderer finished, take it and finalize
                                if self.rs.is_none() {
                                    let pending_result = self
                                        .wasm_pending_rs
                                        .as_ref()
                                        .and_then(|holder| holder.borrow_mut().take());
                                    if let Some(result) = pending_result {
                                        let mut rs = match result {
                                            Ok(rs) => rs,
                                            Err(error) => {
                                                self.exit_with_error(event_loop, error.into());
                                                return;
                                            }
                                        };
                                        if let Err(error) = Self::upload_pending_atlases(
                                            &mut self.resources,
                                            &mut rs,
                                        ) {
                                            self.exit_with_error(event_loop, error.into());
                                            return;
                                        }
                                        self.rs = Some(rs);
                                    }
                                }
                            }
                            let mut elapsed_time =
                                self.prev_time.elapsed().as_millis() as f32 / 1000.0;
                            self.prev_time = Instant::now();

                            if elapsed_time > 0.2 {
                                elapsed_time = 0.2;
                            }

                            // Accumulate time for fixed-step updates
                            self.accumulator += elapsed_time;
                            while self.accumulator >= self.fixed_dt {
                                // Check if app wants to run fixed updates
                                if self.app.should_run_fixed(&self.world, &self.resources) {
                                    // Provide fixed dt to systems and step dispatcher
                                    self.resources.insert(DeltaTime(self.fixed_dt));
                                    self.app.fixed_update(
                                        &mut self.world,
                                        &mut self.resources,
                                        self.fixed_dt,
                                    );
                                    self.dispatcher
                                        .run_now(&mut self.world, &mut self.resources);
                                }
                                self.accumulator -= self.fixed_dt;
                            }

                            // Variable-step update for non-physics
                            self.update(elapsed_time);

                            match self.render() {
                                Ok(_) => {}
                                Err(RenderError::Surface(
                                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                )) => {
                                    if let Some(rs) = &mut self.rs {
                                        rs.resize(self.size);
                                    }
                                }
                                Err(RenderError::Surface(wgpu::SurfaceError::OutOfMemory)) => {
                                    self.exit_with_error(
                                        event_loop,
                                        RenderError::Surface(wgpu::SurfaceError::OutOfMemory)
                                            .into(),
                                    );
                                }
                                Err(RenderError::Surface(wgpu::SurfaceError::Timeout)) => {
                                    log::warn!("Surface timeout")
                                }
                                Err(RenderError::Surface(wgpu::SurfaceError::Other)) => {
                                    log::warn!("Surface error: other")
                                }
                                Err(error) => {
                                    self.exit_with_error(event_loop, error.into());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl<A: Application> Engine<A> {
    fn exit_with_error(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        error: EngineError,
    ) {
        log::error!("{error}");
        self.fatal_error = Some(error);
        event_loop.exit();
    }

    #[cfg(not(target_arch = "wasm32"))]
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

        // Initialize World and Resources
        let mut world = World::new();
        let mut resources = ResourceContainer::new();

        // Insert engine-managed resources first
        let aspect_ratio = width as f32 / height as f32;
        resources.insert(crate::resources::Camera::init_orthographic(
            20.0,
            aspect_ratio,
        ));
        resources.insert(DeltaTime(0.0));
        resources.insert(TextureAtlasRegistry::default());

        // Initialize application (can adjust camera via resources)
        app.init(&mut world, &mut resources);

        let engine = Self {
            app,
            world,
            resources,
            rs: None,
            render_extractor: RenderWorldExtractor::with_capacity(8, 128),
            dispatcher,
            window: None,
            size,
            prev_time,
            accumulator,
            fixed_dt,
            title: title.into(),
            initial_width: width,
            initial_height: height,
            fatal_error: None,
            #[cfg(target_arch = "wasm32")]
            wasm_pending_rs: None,
        };

        (engine, event_loop)
    }

    /// Start the engine - creates and runs in one go
    pub async fn start<T: Into<String>>(
        app: A,
        title: T,
        width: u32,
        height: u32,
        texture_atlases: Vec<TextureAtlasAsset>,
        dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    ) -> Result<(), EngineError> {
        let event_loop = EventLoop::new()?;

        // Initialize World and Resources
        let mut world = World::new();
        let mut resources = ResourceContainer::new();

        // Insert engine-managed resources first
        let aspect_ratio = width as f32 / height as f32;
        resources.insert(crate::resources::Camera::init_orthographic(
            20.0,
            aspect_ratio,
        ));
        resources.insert(DeltaTime(0.0));
        let mut atlas_registry = TextureAtlasRegistry::default();
        for asset in texture_atlases {
            atlas_registry.register(asset)?;
        }
        resources.insert(atlas_registry);

        // Initialize application (can adjust camera via resources)
        let mut app = app;
        app.init(&mut world, &mut resources);

        let mut engine = Self {
            app,
            world,
            resources,
            rs: None,
            render_extractor: RenderWorldExtractor::with_capacity(8, 128),
            dispatcher,
            window: None,
            size: PhysicalSize::new(width, height),
            prev_time: Instant::now(),
            accumulator: 0.0,
            fixed_dt: 1.0 / 60.0,
            title: title.into(),
            initial_width: width,
            initial_height: height,
            fatal_error: None,
            #[cfg(target_arch = "wasm32")]
            wasm_pending_rs: None,
        };

        event_loop.run_app(&mut engine)?;
        if let Some(error) = engine.fatal_error {
            return Err(error);
        }
        Ok(())
    }

    /// Run the engine with the provided event loop
    pub fn run_with_event_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run_app(&mut self).unwrap();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        if let Some(rs) = &mut self.rs {
            rs.resize(new_size);
        }
    }

    fn update(&mut self, dt: f32) {
        self.resources.insert(DeltaTime(dt));
        self.app.update(&mut self.world, &mut self.resources, dt);
    }

    fn upload_pending_atlases(
        resources: &mut ResourceContainer,
        render_state: &mut RenderState,
    ) -> Result<(), RenderError> {
        let registry = resources
            .get_mut::<TextureAtlasRegistry>()
            .expect("TextureAtlasRegistry resource not found");
        if let Some(error) = registry.take_error() {
            return Err(error.into());
        }
        let pending = registry.take_pending();

        for asset in pending {
            render_state.load_texture_atlas(asset.id(), asset.bytes())?;
            resources
                .get_mut::<TextureAtlasRegistry>()
                .expect("TextureAtlasRegistry resource not found")
                .mark_loaded(asset.id().clone());
        }
        Ok(())
    }

    fn render(&mut self) -> Result<(), RenderError> {
        let rs = match &mut self.rs {
            Some(rs) => rs,
            None => return Ok(()),
        };
        Self::upload_pending_atlases(&mut self.resources, rs)?;
        let frame = self
            .render_extractor
            .extract(&self.world, &self.resources)?;
        rs.render_frame(&frame)
    }
}
