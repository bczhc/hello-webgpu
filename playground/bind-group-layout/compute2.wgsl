@group(0) @binding(2) var<storage, read_write> result: array<u32>;

@compute @workgroup_size(1)
fn compute(@builtin(global_invocation_id) global_id: vec3u) {
    let i = global_id.x;
    result[i] += 1;
}
