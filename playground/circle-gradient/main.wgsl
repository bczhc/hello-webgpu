struct Vertex {
    @location(0) pos: vec2f,
}

struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs(@builtin(vertex_index) index: u32, vertex: Vertex)
 -> VsOut {
    let yellow = vec4f(1,1,0,1);
    let green = vec4f(0,1,0,1);
    var out: VsOut;
    out.pos = vec4f(vertex.pos, 0, 1);
    out.color = select(green, yellow, index % 2 == 1);
    return out;
}

@fragment
fn fs(@location(0) color: vec4f) -> @location(0) vec4f {
    return color;
}