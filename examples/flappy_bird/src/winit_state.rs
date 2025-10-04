use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::Window,
};
#[derive(Debug)]
pub struct WinitState {
    pub events_loop: EventLoop<()>,
    pub window: Window,
}

impl WinitState {
    /// Constructs a new `EventsLoop` and `Window` pair.
    ///
    /// The specified title and size are used, other elements are default.
    /// ## Failure
    /// It's possible for the window creation to fail. This is unlikely.
    pub fn create<T: Into<String>>(
        title: T,
        width : u32,height:u32,
    ) -> (winit::window::WindowAttributes, EventLoop<()>) {
        let events_loop = EventLoop::new().unwrap();
        
        #[cfg(target_arch = "wasm32")]
        let window_attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(width, height));
            
        #[cfg(not(target_arch = "wasm32"))]
        let window_attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height));
        
        (window_attributes, events_loop)
    }
}
