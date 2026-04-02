@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

const WIDTH: u32 = 800u;
const HEIGHT: u32 = 600u;

@compute
@workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= WIDTH || id.y >= HEIGHT) {
        return;
    }

    textureStore(texture, id.xy, vec4<f32>(1.0, 0.0, 0.0, 1.0));
}