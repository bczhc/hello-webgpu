const VERTICES: array<vec2f, 6> = array(
    vec2f(-1, -1),
    vec2f(-1, 1),
    vec2f(1, 1),
    vec2f(1, 1),
    vec2f(1, -1),
    vec2f(-1, -1),
);

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var tex: texture_storage_2d<r32float, read_write>;

@compute
@workgroup_size(1)
fn update_texture(@builtin(global_invocation_id) global_id: vec3u) {
    let _n = samp;
    let index = global_id.x;
    let positions = array(
        vec2u(0, 0),
        vec2u(1, 0),
        vec2u(0, 1),
        vec2u(1, 1),
    );
    let red_value = 1.0 - 0.2 * f32(index);
    textureStore(tex, positions[index], vec4f(red_value, 0, 0, 1));
}

@vertex
fn vs(@builtin(vertex_index) index: u32)
  -> @builtin(position) vec4f {
    return vec4f(VERTICES[index], 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let _n = samp;
    let mipmap_level = 0u;

    var tex_pos = vec2u(0, 0);
    if pos.x >= 250 {
        tex_pos.x = 1;
    }
    if pos.y >= 250 {
        tex_pos.y = 1;
    }
    let tex_color = textureLoad(tex, tex_pos);
    return tex_color;
}
