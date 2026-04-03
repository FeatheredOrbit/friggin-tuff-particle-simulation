#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Uniforms {
    pub number_of_particles: u32
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
    // First slot is particle type: 0 = Red || 1 = Green || 2 = Blue
    pub data_1: [u32; 4]
}
impl ParticleData {
    pub fn new(particle_type: ParticleType) -> Self {
        Self {
            data_1: [particle_type as u32, 0, 0, 0]
        }
    }
}
