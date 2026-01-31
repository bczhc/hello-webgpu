@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(1)
fn compute(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    for (var j = 0; j < 10000; j += 1) {
        data[i] = sqrt(data[i]) * 2.0;
    }
}