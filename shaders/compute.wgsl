struct Uniforms {
    number_of_particles: u32
}

struct ParticleData {
    // First slot is particle type: 0 = Red || 1 = Green || 2 = Blue
    data_1: vec4<u32>
}

const RED: u32 = 0u;
const GREEN: u32 = 1u;
const BLUE: u32 = 2u;

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

@group(0) @binding(2)
var<storage, read_write> particle_data: array<ParticleData>; 

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let id = global_invocation_id.x;

    if (id >= uniforms.number_of_particles) {
        return;
    }

    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    if particle_data[id].data_1.x == RED {
        color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
    else if particle_data[id].data_1.x == GREEN {
        color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    else if particle_data[id].data_1.x == BLUE {
        color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }

    textureStore(texture, vec2<u32>(600, 300), color);
}
