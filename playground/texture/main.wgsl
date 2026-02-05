const CANVAS_WIDTH = 500;
const CANVAS_HEIGHT = 500;

struct Vertex {
    @location(0) pos: vec2f,
}

struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) tex_coord: vec2f,
}

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var<storage, read> t_info: Params;
@group(0) @binding(3) var overlay_tex: texture_2d<f32>;

// size: 24
struct Params {
    offset: vec2f,
    scale: f32,
    sampling_mode: u32,
    // offset: 16
    enable_overlay: u32,
}

@vertex
fn vs(@builtin(vertex_index) _index: u32, vertex: Vertex)
 -> VsOut {
    let pos = vec4f(vertex.pos * t_info.scale + t_info.offset, 0.0, 1.0);
    return VsOut(pos, vertex.pos);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4f {
    let samp_coord = select(
        vec2f(
            in.tex_coord.x / 2 + 0.5,
            -in.tex_coord.y / 2+ 0.5
        ),
        vec2f(in.pos.x / CANVAS_WIDTH, in.pos.y / CANVAS_HEIGHT),
        t_info.sampling_mode == 2
    );
    let tex1 = textureSample(tex, samp, samp_coord);
    let tex2 = textureSample(overlay_tex, samp, samp_coord);
    if t_info.enable_overlay == 1 {
        return vec4f(tex1.xyz * 1 + tex2.xyz * 0.5, 1);
    } else {
        return tex1;
    }
}
