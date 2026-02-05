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
    textureStore(tex, tex_pos, tex_color * 0.9);
    let tex_color_updated = textureLoad(tex, tex_pos);
    return tex_color_updated;
}
