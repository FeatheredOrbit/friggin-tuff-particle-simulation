use winit::event_loop::EventLoop;

use crate::engine::Engine;

mod engine;
mod renderer;
mod constants;
mod shader_data;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut Engine::default()).unwrap();
}
