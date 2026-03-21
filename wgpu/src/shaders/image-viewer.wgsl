struct Info {
    image_size: vec2u,
    out_size: vec2u,
}

@group(0) @binding(0) var<uniform> info: Info;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var texture: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
    let quad_vertices = array(
        vec2f(-1, -1),
        vec2f(-1, 1),
        vec2f(1, 1),
        vec2f(1, 1),
        vec2f(1, -1),
        vec2f(-1, -1),
    );
    return vec4f(quad_vertices[vi], 0, 1);
}

@fragment
fn fs(@builtin(position) fs_pos: vec4f) -> @location(0) vec4f {
    let u = fs_pos.x / f32(info.out_size.x);
    let v = fs_pos.y / f32(info.out_size.y);

    let color = textureSample(texture, samp, vec2f(u, v));
    return color;
}
