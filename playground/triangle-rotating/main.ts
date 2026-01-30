import shader from "./main.wgsl?raw";

(async () => {
    let canvas = document.querySelector('canvas')!!;
    let context = canvas.getContext('webgpu');

    let adapter = (await navigator.gpu.requestAdapter())!!;
    let device = await adapter.requestDevice();
    if (!device || !context) {
        alert("WebGPU is not supported");
        return;
    }

    let textureFormat = navigator.gpu.getPreferredCanvasFormat();
    context.configure({
        device,
        format: textureFormat,
    });

    function createShaderModule() {
        return device.createShaderModule({
            label: 'shader1',
            code: shader,
        })
    }

    let shaderModule = createShaderModule();

    let pipeline = device.createRenderPipeline({
        layout: 'auto',
        vertex: {
            module: shaderModule,
        },
        fragment: {
            module: shaderModule,
            targets: [
                {format: textureFormat}
            ]
        }
    })

    let rotation = 0;

    function render() {
        let encoder = device.createCommandEncoder();

        let uniformData = new Float32Array([rotation]);
        let uniformBuffer = device.createBuffer({
            size: uniformData.byteLength,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
            mappedAtCreation: false,
        });
        let uniformBindGroup = device.createBindGroup({
            layout: pipeline.getBindGroupLayout(0),
            entries: [
                {
                    binding: 0,
                    resource: uniformBuffer,
                }
            ]
        });

        let pass = encoder.beginRenderPass({
            colorAttachments: [
                {
                    view: context!!.getCurrentTexture(),
                    loadOp: 'clear',
                    storeOp: 'store',
                    clearValue: [0.3, 0.3, 0.3, 1],
                },
            ],
        });
        pass.setPipeline(pipeline);
        device.queue.writeBuffer(uniformBuffer, 0, uniformData);
        pass.setBindGroup(0, uniformBindGroup);
        pass.draw(3);
        pass.end()

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
        rotation += -0.05;

        requestAnimationFrame(render);
    }

    requestAnimationFrame(render);
})();
