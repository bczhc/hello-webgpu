const COLOR = vec4f(1,0,0,1);
const SQRT3 = sqrt(3.0);

@group(0) @binding(0)
var<uniform> rotation: f32;

@vertex
fn vs(@builtin(vertex_index) index: u32)
-> @builtin(position) vec4f {
    let pos = array(
        vec2f(0.0, 0.5),
        vec2f(SQRT3 / 4.0, -0.25),
        vec2f(-SQRT3 / 4.0, -0.25),
    );

    let r = rotation;
    let r_mat = mat2x2(cos(r), sin(r), -sin(r), cos(r));

    return vec4f(r_mat * pos[index], 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4f {
    return COLOR;
}