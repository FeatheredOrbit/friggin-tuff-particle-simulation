struct Uniforms {
    number_of_particles: u32
}

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= uniforms.number_of_particles) {
        return;
    }

    textureStore(texture, vec2<u32>(600, 300), vec4<f32>(1.0, 0.0, 0.0, 1.0));
}