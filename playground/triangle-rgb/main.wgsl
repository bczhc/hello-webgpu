struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs(@builtin(vertex_index) vertexIndex: u32)
-> VsOut {
    let pos = array(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );
    let colors = array(
        vec4f(1.0, 0.0, 0.0, 1.0),
        vec4f(0.0, 1.0, 0.0, 1.0),
        vec4f(0.0, 0.0, 1.0, 1.0),
    );

    return VsOut(vec4f(pos[vertexIndex], 0.0, 1.0), colors[vertexIndex]);
}

@fragment
fn fs(@location(0) color: vec4f) -> @location(0) vec4f {
    return color;
}
