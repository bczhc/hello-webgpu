struct Vertex {
    // ===== vertex =====
    @location(0) pos: vec2f,
    @location(1) color: vec3f,

    // ===== instance =====
    @location(2) offset: vec2f,
    @location(3) scale: f32,
}

struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs(@builtin(vertex_index) _index: u32, v: Vertex) -> VsOut {
    return VsOut(
        vec4f(v.pos * v.scale + v.offset, 0.0, 1.0),
        vec4f(v.color, 1.0),
    );
}

@fragment
fn fs(@location(0) color: vec4f) -> @location(0) vec4f {
    return color;
}
