// left, right
var<immediate> imm: array<f32, 2>;

@vertex
fn vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f {
    let vertices: array<vec2f, 4> = array(
        vec2f(-1, 1),
        vec2f(-1, -1),
        vec2f(1, -1),
        vec2f(1, 1),
    );
    return vec4f(vertices[vid], 0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    const YELLOW = vec4f(1, 1, 0, 1);
    const T = vec4f(0);
    let left = imm[0];
    let right = imm[1];

    for (var i = 0; i < 500; i += 1) {}

    if pos.x >= left && pos.x <= right {
        return YELLOW;
    } else {
        return T;
    }
}
