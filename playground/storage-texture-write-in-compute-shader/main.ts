import shader from "./main.wgsl?raw";

;
(async () => {
    let canvas = document.querySelector('canvas')!!;
    let context = canvas.getContext('webgpu')!!;

    let preferredFormat = navigator.gpu.getPreferredCanvasFormat();
    let adapter = await navigator.gpu.requestAdapter();
    let device = await adapter!!.requestDevice();

    context.configure({
        device,
        format: preferredFormat,
    });

    let shaderModule = device.createShaderModule({
        code: shader,
        label: 'shader module 1',
    });

    let computePipeline = device.createComputePipeline({
        layout: 'auto',
        compute: {
            module: shaderModule,
        }
    });
    let renderPipeline = device.createRenderPipeline({
        layout: 'auto',
        vertex: {
            module: shaderModule,
            buffers: []
        },
        fragment: {
            module: shaderModule,
            targets: [
                {format: preferredFormat,}
            ]
        }
    });

    let sampler = device.createSampler({
        magFilter: 'nearest',
    });
    let storageTexture = device.createTexture({
        size: [2, 2],
        format: 'r32float',
        usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_DST,
    });

    function createBindGroup(pipeline: GPURenderPipeline | GPUComputePipeline) {
        return device.createBindGroup({
            layout: pipeline.getBindGroupLayout(0),
            entries: [
                {binding: 0, resource: sampler},
                {binding: 1, resource: storageTexture},
            ]
        });
    }
    let computeBindGroup = createBindGroup(computePipeline);
    let renderBindGroup = createBindGroup(renderPipeline);

    function addComputePass(encoder: GPUCommandEncoder) {
        let pass = encoder.beginComputePass();
        pass.setPipeline(computePipeline);
        pass.setBindGroup(0, computeBindGroup);
        pass.dispatchWorkgroups(1);
        pass.end()
    }

    function addRenderPass(encoder: GPUCommandEncoder) {
        let pass = encoder.beginRenderPass({
            colorAttachments: [
                {
                    view: context.getCurrentTexture(),
                    loadOp: 'clear',
                    storeOp: 'store',
                    clearValue: [.3, .3, .3, 1.0],
                }
            ]
        });
        device.queue.writeTexture({
            texture: storageTexture,
        }, (() => {
            return new Float32Array(2 * 2);
        })(), {
            bytesPerRow: 2 * 4,
            offset: 0
        }, [2, 2]);
        pass.setPipeline(renderPipeline);
        pass.setBindGroup(0, renderBindGroup);
        pass.draw(6, 1);
        pass.end();
    }

    function render() {
        let encoder = device.createCommandEncoder();
        addComputePass(encoder);
        addRenderPass(encoder);
        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render();
})();
