const COLOR = vec4f(1,0,0,1);

struct Uniform {
    matrix: mat3x3f,
}

struct Vertex {
    @location(0) pos: vec2f,
}

@group(0) @binding(0) var<uniform> uni: Uniform;

@vertex
fn vs(v: Vertex, @builtin(vertex_index) _index: u32) -> @builtin(position) vec4f {
    let pos_transformed = (uni.matrix * vec3f(v.pos, 1.0)).xy;
    return vec4f(pos_transformed, 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4f {
    return COLOR;
}