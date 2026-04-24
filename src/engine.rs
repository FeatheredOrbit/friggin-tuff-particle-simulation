use std::sync::Arc;

use winit::{application::ApplicationHandler, dpi::LogicalSize, window::{Window, WindowAttributes}};

use crate::renderer::{Renderer, shader_data::{ParticleData, ParticleType}};


#[derive(Default)]
pub struct Engine<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'a>>
}

impl<'a> ApplicationHandler for Engine<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                WindowAttributes::default()
                .with_inner_size(LogicalSize::new(1200, 600))
                .with_resizable(false)
                .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
                .with_title("Friggin Tuff Particle Simulation")
            ).unwrap()
        );

        self.window = Some(window.clone());

        if self.renderer.is_none() {
            self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());

            let mut particles: Vec<ParticleData> = Vec::new();

            for x in 0..500 {
                particles.push(ParticleData::new(ParticleType::RED, 101.0 + x as f32 / 10000.0, 101.0));
            }

            for _ in 0..1 {
                particles.push(ParticleData::new(ParticleType::BLUE, 100.0, 100.0));
            }

            self.renderer.as_mut().unwrap().set_particles(particles);
        }
    }

    fn window_event
    (
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) 
    {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            winit::event::WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }

            winit::event::WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.render() {
                        Ok(()) => {},
                        Err(e) => {
                            eprintln!("{}", e);
                            event_loop.exit();
                        }
                    }
                }
            }

            _ => {}
        }
    }
}