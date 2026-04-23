struct Info {
    image_size: vec2u,
    out_size: vec2u,
    uv_offset: vec2f,
    _pad1: vec2u,
    no_scale: u32,
    proportional: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> info: Info;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var texture: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
    let quad_vertices = array(
        vec2f(-1, -1),
        vec2f(-1, 1),
        vec2f(1, 1),
        vec2f(1, 1),
        vec2f(1, -1),
        vec2f(-1, -1),
    );
    return vec4f(quad_vertices[vi], 0, 1);
}

@fragment
fn fs(@builtin(position) fs_pos: vec4f) -> @location(0) vec4f {
    const BLACK = vec4f(0, 0, 0, 1);

    let s = samp;

    let u = fs_pos.x / f32(info.out_size.x);
    let v = fs_pos.y / f32(info.out_size.y);

    var uv = vec2f(u, v);
    if info.proportional == 1 {
        let input_aspect = f32(info.image_size.x) / f32(info.image_size.y);
        let output_aspect = f32(info.out_size.x) / f32(info.out_size.y);
        if input_aspect > output_aspect {
            let s = output_aspect / input_aspect;
            uv.y = (uv.y - 0.5) / s + 0.5;
        } else {
            let s = input_aspect / output_aspect;
            uv.x = (uv.x - 0.5) / s + 0.5;
        }
    }

    var color: vec4f;
    if info.no_scale == 1 {
        color = textureLoad(texture, vec2u(fs_pos.xy), 0);
    } else {
        let sampled_uv = uv + info.uv_offset;
        if sampled_uv.x < 0 || sampled_uv.x > 1 || sampled_uv.y < 0 || sampled_uv.y > 1 {
            return BLACK;
        }
        color = textureSample(texture, s, sampled_uv);
    }
    return color;
}
