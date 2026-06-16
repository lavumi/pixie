#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(error) = pollster::block_on(prey_predator::start()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
