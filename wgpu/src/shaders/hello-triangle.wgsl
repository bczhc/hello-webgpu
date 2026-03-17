const VERTICES = array(
    vec2f(0, 0.5),
    vec2f(-0.5, -0.5),
    vec2f(0.5, -0.5),
);

const RED = vec4f(1.0, 0, 0, 1.0);

struct VsOut {
    @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let v = vec4f(VERTICES[i], 0.0, 1.0);
    return VsOut(v);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return RED;
}