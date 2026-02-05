import shader from "./main.wgsl?raw";

(async () => {
    let adapter = (await navigator.gpu.requestAdapter())!!;
    let device = await adapter.requestDevice();

    let pipeline = device.createComputePipeline({
        layout: 'auto',
        compute: {
            module: device.createShaderModule({
                code: shader,
            })
        },
    });

    let input = [1, 2, 3, 4, 5];

    let workBufferData = new Float32Array(input);
    let workBuffer = device.createBuffer({
        size: workBufferData.byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC,
        mappedAtCreation: true,
    });
    new Float32Array(workBuffer.getMappedRange()).set(input);
    workBuffer.unmap();

    let resultBuffer = device.createBuffer({
        size: workBufferData.byteLength,
        usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });

    let bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
            {binding: 0, resource: workBuffer},
        ]
    });

    function computeCommandBuffer() {
        let encoder = device.createCommandEncoder();
        let pass = encoder.beginComputePass({});
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        pass.dispatchWorkgroups(workBufferData.length);
        pass.end();

        encoder.copyBufferToBuffer(workBuffer, resultBuffer, workBufferData.byteLength);
        return encoder.finish();
    }

    let cb1 = computeCommandBuffer();
    let cb2 = computeCommandBuffer();
    device.queue.submit([cb1, cb2]);

    await resultBuffer.mapAsync(GPUMapMode.READ);
    let result = new Float32Array(resultBuffer.getMappedRange()).slice();
    resultBuffer.unmap();

    console.log('input', workBufferData);
    console.log('output', result);
    document.querySelector('#span-input')!!.innerHTML = `Input: ${workBufferData.toString()}`
    document.querySelector('#span-output')!!.innerHTML = `Output: ${result.toString()}`;
})();
