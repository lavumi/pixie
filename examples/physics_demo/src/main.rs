#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(error) = pollster::block_on(physics_demo::start()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
