struct Uniforms {
    // X is number of particles.
    // Y is screen width.
    // Z is screen height.
    data_1: vec4<u32>
}

@group(0) @binding(0)
var texture_read: texture_storage_2d<rgba8unorm, read>;

@group(0) @binding(1)
var texture_write: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

@compute
@workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let id_x = global_invocation_id.x;
    let id_y = global_invocation_id.y;

    if id_x >= uniforms.data_1.y || id_y >= uniforms.data_1.z { return; }

    var color = textureLoad(texture_read, vec2<u32>(id_x, id_y));

    // Fade out alpha.
    if color.w <= 0.04 {
        color.w = 0.0;
    }
    else {
        color.w *= 0.95;
    }

    textureStore(texture_write, vec2<u32>(id_x, id_y), color);
}