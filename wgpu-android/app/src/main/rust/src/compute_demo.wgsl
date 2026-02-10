override WORKGROUP_SIZE: u32;

@group(0) @binding(0)
var<storage, read_write> data: array<u32>;

@compute @workgroup_size(WORKGROUP_SIZE)
fn compute(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    if i >= arrayLength(&data) { return; }
    
    data[i] += 1;
}
