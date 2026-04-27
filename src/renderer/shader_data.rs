#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Uniforms {
    // First slot is number of particles.
    // Second slot is screen width.
    // Third slot is screen height.
    // W is window scale factor, needs to be sent as bits though.
    pub data_1: [u32; 4]
}

#[allow(unused)]
#[repr(u32)]
pub enum ParticleType {
    RED = 0,
    GREEN = 1,
    BLUE = 2
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ParticleData {
    // First slot is particle type: 0 = Red || 1 = Green || 2 = Blue.
    // Second slot is x position.
    // Third slot is y position.
    pub data_1: [f32; 4]
}
impl ParticleData {
    pub fn new(particle_type: ParticleType, x_pos: f32, y_pos: f32) -> Self {

        Self {
            // Yeah, ugly ash.
            data_1: [particle_type as u32 as f32, x_pos, y_pos, 0.0]
        }
    }
}
