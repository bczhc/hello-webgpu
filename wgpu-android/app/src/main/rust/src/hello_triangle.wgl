struct VertexInput {
    @location(0) position: vec2f,
    @location(1) color: vec3f,
}

struct Uniform {
    elapsed: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
}

@group(0) @binding(0) var<uniform> myUniform: Uniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let r = myUniform.elapsed * 2.0 * 3.1415926536;
    let cos_a = cos(r);
    let sin_a = sin(r);

    // 旋转位置
    let rotated_position = vec2f(
        in.position.x * cos_a - in.position.y * sin_a,
        in.position.x * sin_a + in.position.y * cos_a
    );

    out.clip_position = vec4f(rotated_position, 0.0, 1.0);
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}