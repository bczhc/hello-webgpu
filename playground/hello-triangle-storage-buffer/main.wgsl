// size: 4
struct StaticStorage {
    scale: f32,
}

// size: 32
struct ChangingStorage {
    color: vec4f,
    offset: vec2f,
    _pad: vec2f,
}

@group(0) @binding(0)
var<storage, read> static_storage: StaticStorage;

@group(0) @binding(1)
var<storage, read> changing_storage: ChangingStorage;

@vertex
fn vs(@builtin(vertex_index) index: u32)
-> @builtin(position) vec4f {
    let pos = array(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );

    let pos2 = (pos[index] * static_storage.scale)
        + changing_storage.offset;

    return vec4f(pos2, 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4f {
    return changing_storage.color;
}