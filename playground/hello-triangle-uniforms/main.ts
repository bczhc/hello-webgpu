import shader from "./main.wgsl?raw";

const rand = (min: number, max: number) => {
    if (min === undefined) {
        min = 0;
        max = 1;
    } else if (max === undefined) {
        max = min;
        min = 0;
    }
    return min + Math.random() * (max - min);
};

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

    function createEntities() {
        let entities = [];
        let scale = 2.0;
        for (let i = 0; i < 10; i++) {
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
            uniformBuffer.color.set([rand(0, 1), rand(0, 1), rand(0, 1), 1]);
            uniformBuffer.scale.set([scale]);

            entities.push({
                uniformGpuBuffer,
                uniformBuffer,
                bindGroup: device.createBindGroup({
                    layout: pipeline.getBindGroupLayout(0),
                    entries: [{binding: 0, resource: uniformGpuBuffer}],
                })
            });

            scale -= 0.2;
        }
        return entities;
    }

    let entities = createEntities();

    function render() {
        let encoder = device.createCommandEncoder();

        let pass = encoder.beginRenderPass({
            colorAttachments: [
                {
                    view: context!!.getCurrentTexture(),
                    loadOp: 'clear',
                    storeOp: 'store',
                    clearValue: [0.3, 0.3, 0.3, 1]
                },
            ],
        });
        pass.setPipeline(pipeline);

        // pick different uniform buffer on each draw
        for (let entity of entities) {
            device.queue.writeBuffer(entity.uniformGpuBuffer, 0, entity.uniformBuffer.buffer);
            pass.setBindGroup(0, entity.bindGroup);
            pass.draw(3);
        }

        pass.end()

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render();
})();
