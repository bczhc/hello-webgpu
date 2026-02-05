@group(0) @binding(0) var<storage, read_write> buf1: array<u32>;
@group(0) @binding(1) var<storage, read_write> buf2: array<u32>;
@group(0) @binding(2) var<storage, read_write> result: array<u32>;

@compute @workgroup_size(256)
fn compute(@builtin(global_invocation_id) global_id: vec3u) {
    let i = global_id.x;
    if i >= arrayLength(&result) {
        return;
    }
    result[i] = buf1[i] + buf2[i];
}
