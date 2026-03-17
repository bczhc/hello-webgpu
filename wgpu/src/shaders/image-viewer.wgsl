struct Info {
    width_pixels: u32,
}

@group(0) @binding(0) var<uniform> info: Info;
@group(0) @binding(1) var<storage, read> texture_buffer: array<u32>;

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
    let pos: vec2u = vec2u(fs_pos.xy);
    let offset = (pos.x + pos.y * info.width_pixels) * 3;
    let r = texture_buffer[offset];
    let g = texture_buffer[offset + 1];
    let b = texture_buffer[offset + 2];
    return vec4f(f32(r) / 255.0, f32(g) / 255.0, f32(b) / 255.0, 1.0);
}
