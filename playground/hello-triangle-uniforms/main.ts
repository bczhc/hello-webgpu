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
    let uniformGpuBuffer = device.createBuffer({
        size: 32,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });
    let uniformBuffer = (() => {
        let b = new ArrayBuffer(32);
        return {
            buffer: b,
            color: new Float32Array(b, 0, 4),
            scale: new Float32Array(b, 16, 1),
        }
    })();

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

    function render(scale: number, color: Array<number>, loadOp: GPULoadOp) {
        uniformBuffer.color.set(color);
        uniformBuffer.scale.set([scale]);
        let bindGroup = device.createBindGroup({
            layout: pipeline.getBindGroupLayout(0),
            entries: [
                {binding: 0, resource: uniformGpuBuffer},
            ]
        });
        device.queue.writeBuffer(uniformGpuBuffer, 0, uniformBuffer.buffer);

        let encoder = device.createCommandEncoder();

        let pass = encoder.beginRenderPass({
            colorAttachments: [
                {
                    view: context!!.getCurrentTexture(),
                    loadOp,
                    storeOp: 'store',
                    clearValue: [0.3, 0.3, 0.3, 1]
                },
            ],
        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        pass.draw(3);
        pass.end()

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render(2, [1, 1, 0, 1], 'clear');
    render(1, [1, 0, 0, 1], 'load');
})();
