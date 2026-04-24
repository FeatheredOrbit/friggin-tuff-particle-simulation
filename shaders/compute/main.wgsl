struct Uniforms {
    // X is number of particles.
    // Y is screen width.
    // Z is screen height.
    data_1: vec4<u32>
}

struct ParticleData {
    // X slot is particle type: 0 = Red || 1 = Green || 2 = Blue. Just cast them to u32 though.
    // Y slot is x position.
    // Z slot is y position.
    data_1: vec4<f32>
}

const RED: u32 = 0u;
const GREEN: u32 = 1u;
const BLUE: u32 = 2u;

const MAX_LOOK_DISTANCE: u32 = 100u;

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

    if (id >= uniforms.data_1.x) {
        return;
    }

    behave(id);
    draw_texture(id);
}

fn behave(id: u32) {

    for (var iter_id = 0u; iter_id < uniforms.data_1.x; iter_id++) {

        if (iter_id == id) { continue; }

        if u32(particle_data[id].data_1.x) == RED {
            red(id, iter_id);
        }

    }  

    bound_check(id);

    // Check if the particle has gone out of bounds during behaviour, if so, bring it back.
    particle_data[id].data_1.y = clamp(particle_data[id].data_1.y, 0.0, f32(uniforms.data_1.y) - 1.0);
    particle_data[id].data_1.z = clamp(particle_data[id].data_1.z, 0.0, f32(uniforms.data_1.z) - 1.0);  

}

// Called if particle under id is red.

// Red runs away from blue.
// Red runs away from red.
fn red(id: u32, iter_id: u32) {
    let other_particle = particle_data[iter_id];

    // Move away from blue.
    if u32(other_particle.data_1.x) == BLUE {
        let dx = other_particle.data_1.y - particle_data[id].data_1.y;
        let dy = other_particle.data_1.z - particle_data[id].data_1.z;

        let length = sqrt(dx * dx + dy * dy);

        if length != 0.0 {
            let dir_x = dx / length;
            let dir_y = dy / length;

            particle_data[id].data_1.y -= dir_x * 0.01;
            particle_data[id].data_1.z -= dir_y * 0.01;
        }
    }

    // Move away from red.
    if u32(other_particle.data_1.x) == RED {
        let dx = other_particle.data_1.y - particle_data[id].data_1.y;
        let dy = other_particle.data_1.z - particle_data[id].data_1.z;

        let length = sqrt(dx * dx + dy * dy);

        if length != 0.0 {
            let dir_x = dx / length;
            let dir_y = dy / length;

            particle_data[id].data_1.y -= dir_x * 0.01;
            particle_data[id].data_1.z -= dir_y * 0.01;
        }
    }
}

// Particles should be pushed away from the walls of the screen.
fn bound_check(id: u32) {
    // Check for left wall.
    {
        let dx = 0.0 - particle_data[id].data_1.y;
        let dy = 0.0;

        let length = sqrt(dx * dx + dy * dy);

        if length != 0.0 {
            let dir_x = dx / length;
            let dir_y = dy / length;

            particle_data[id].data_1.y -= dir_x * 0.01;
            particle_data[id].data_1.z -= dir_y * 0.01;
        }
    }

    // Check for right wall.
    {
        let dx = f32(uniforms.data_1.y) - particle_data[id].data_1.y;
        let dy = 0.0;

        let length = sqrt(dx * dx + dy * dy);

        if length != 0.0 {
            let dir_x = dx / length;
            let dir_y = dy / length;

            particle_data[id].data_1.y -= dir_x * 0.01;
            particle_data[id].data_1.z -= dir_y * 0.01;
        }
    }

    // Check for ceiling.
    {
        let dx = 0.0;
        let dy = 0.0 - particle_data[id].data_1.z;

        let length = sqrt(dx * dx + dy * dy);

        if length != 0.0 {
            let dir_x = dx / length;
            let dir_y = dy / length;

            particle_data[id].data_1.y -= dir_x * 0.01;
            particle_data[id].data_1.z -= dir_y * 0.01;
        }
    }

    // Check for floor.
    {
        let dx = 0.0;
        let dy = f32(uniforms.data_1.z) - particle_data[id].data_1.z;

        let length = sqrt(dx * dx + dy * dy);

        if length != 0.0 {
            let dir_x = dx / length;
            let dir_y = dy / length;

            particle_data[id].data_1.y -= dir_x * 0.01;
            particle_data[id].data_1.z -= dir_y * 0.01;
        }
    }
}

fn draw_texture(id: u32) {
    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    if u32(particle_data[id].data_1.x) == RED {
        color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
    else if u32(particle_data[id].data_1.x) == GREEN {
        color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    else if u32(particle_data[id].data_1.x) == BLUE {
        color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }

    textureStore(texture, vec2<u32>(u32(particle_data[id].data_1.y), u32(particle_data[id].data_1.z)), color);
}
