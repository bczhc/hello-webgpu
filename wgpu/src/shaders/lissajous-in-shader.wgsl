struct Uniform {
    a: f32,
    b: f32,
    t: f32,
    segments: u32,
    scale: f32,
    // it will automatically pad 3*f32 here
    color: vec4f,
}

@group(0) @binding(0) var<uniform> myUniform: Uniform;

@vertex fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
    // range of t: 0 to 1 (inclusive)
    let t = f32(vi) / f32(myUniform.segments - 1);
    let r = t * 2.0 * 3.14159265358979323846264338327950288;
    let point = vec2f(
        cos(myUniform.a * r * myUniform.t / 10.0), sin(myUniform.a * r * myUniform.t / 11.0 + myUniform.t)
    ) * myUniform.scale;
    return vec4f(point, 0.0, 1.0);
}

@fragment fn fs() -> @location(0) vec4f {
    return myUniform.color;
}
