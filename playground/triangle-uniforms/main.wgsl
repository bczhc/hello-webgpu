// size: 32, align: 16
struct Uniform {
    color: vec4f,
    scale: f32,
}

@group(0) @binding(0)
var<uniform> u: Uniform;

@vertex
fn vs(@builtin(vertex_index) index: u32)
-> @builtin(position) vec4f {
    let pos = array(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );

    return vec4f(pos[index] * u.scale, 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4f {
    return u.color;
}