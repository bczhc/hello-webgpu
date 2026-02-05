const VERTICES: array<vec2f, 6> = array(
    vec2f(-1, -1),
    vec2f(-1, 1),
    vec2f(1, 1),
    vec2f(1, 1),
    vec2f(1, -1),
    vec2f(-1, -1),
);

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var tex: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32)
  -> @builtin(position) vec4f {
    return vec4f(VERTICES[index], 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let tex = textureSample(tex, samp, vec2f(pos.x / 500, pos.y / 500));
    return tex;
}
