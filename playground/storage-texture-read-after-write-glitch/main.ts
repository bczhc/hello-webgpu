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

    let pipeline = device.createRenderPipeline({
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

    let bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
            {binding: 0, resource: sampler},
            {binding: 1, resource: storageTexture},
        ]
    });

    function render() {
        let encoder = device.createCommandEncoder();
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
            let data = new Float32Array(2 * 2);
            data.set([1, 0.8, 0.5, 0.3]);
            return data;
        })(), {
            bytesPerRow: 2 * 4,
            offset: 0
        }, [2, 2]);
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        pass.draw(6, 1);
        pass.end();

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render();
})();
