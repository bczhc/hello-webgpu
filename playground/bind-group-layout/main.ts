import computeShader1 from "./compute1.wgsl?raw";
import computeShader2 from "./compute2.wgsl?raw";

(async () => {
    let adapter = (await navigator.gpu.requestAdapter())!!;
    let device = await adapter.requestDevice();

    // Cases when bindGroupLayout needs to be created manually:
    // 1. custom formats
    // 2. dynamicOffset: true
    // 3. one bindGroup getting used by multiple pipelines
    let bindGroupLayout = device.createBindGroupLayout({
        entries: [
            {
                binding: 0,
                visibility: GPUShaderStage.COMPUTE,
                buffer: {
                    type: "storage",
                    hasDynamicOffset: true,
                    minBindingSize: 0,
                },
            },
            {
                binding: 1,
                visibility: GPUShaderStage.COMPUTE,
                buffer: {
                    type: "storage",
                    hasDynamicOffset: true,
                    minBindingSize: 0,
                },
            },
            {
                binding: 2,
                visibility: GPUShaderStage.COMPUTE,
                buffer: {
                    type: "storage",
                    hasDynamicOffset: true,
                    minBindingSize: 0,
                },
            },
        ]
    });

    let pipelineLayout = device.createPipelineLayout({
        bindGroupLayouts: [bindGroupLayout],
    })

    let pipeline1 = device.createComputePipeline({
        layout: pipelineLayout,
        compute: {
            module: device.createShaderModule({
                code: computeShader1,
            })
        },
    });
    let pipeline2 = device.createComputePipeline({
        layout: pipelineLayout,
        compute: {
            module: device.createShaderModule({
                code: computeShader2,
            })
        }
    });

    // When dynamicOffset is enabled, all buffer bindings should be 256 byte aligned.
    let bufferData = new Uint32Array(256 * 3);
    let bindGroupOffsets = [0, 256, 512];
    bufferData.set([1, 2, 3], 0);
    bufferData.set([3, 2, 1], 64);

    let buffer = device.createBuffer({
        size: bufferData.byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC,
        mappedAtCreation: false,
    });

    let bindGroup = device.createBindGroup({
        layout: bindGroupLayout,
        entries: [
            {binding: 0, resource: {buffer: buffer, size: 256}},
            {binding: 1, resource: {buffer: buffer, size: 256}},
            {binding: 2, resource: {buffer: buffer, size: 256}},
        ]
    });

    let resultBufferData = new Uint32Array(3);
    let resultBuffer = device.createBuffer({
        size: resultBufferData.byteLength,
        usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });

    device.queue.writeBuffer(buffer, 0, bufferData);

    function computeCommandBuffer() {
        let encoder = device.createCommandEncoder();
        let pass = encoder.beginComputePass({});
        pass.setBindGroup(0, bindGroup, bindGroupOffsets);

        pass.setPipeline(pipeline1);
        pass.dispatchWorkgroups(1);

        pass.setPipeline(pipeline2);
        pass.dispatchWorkgroups(3);

        pass.end();

        encoder.copyBufferToBuffer(buffer, 512, resultBuffer, 0, 3 * 4);
        return encoder.finish();
    }

    let commandBuffer = computeCommandBuffer();
    device.queue.submit([commandBuffer]);

    await resultBuffer.mapAsync(GPUMapMode.READ, 0, 3 * 4);
    let mappedResult = resultBuffer.getMappedRange(0, 3 * 4);
    let resultCopied = new Uint32Array(mappedResult).slice();
    resultBuffer.unmap()

    document.querySelector('#span-input1')!!.innerHTML = `Input: ${bufferData.slice(0, 3)}`
    document.querySelector('#span-input2')!!.innerHTML = `Input: ${bufferData.slice(64, 64 + 3)}`
    document.querySelector('#span-output')!!.innerHTML = `Output: ${resultCopied}`;
})();
