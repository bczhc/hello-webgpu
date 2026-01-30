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
        label: 'pipeline 1',
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
        for (let i = 0; i < 50; i++) {
            let staticBuffer = device.createBuffer({
                size: 4,
                mappedAtCreation: false,
                usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
            });
            let changingBuffer = device.createBuffer({
                size: 32,
                mappedAtCreation: false,
                usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
            });
            let staticBufferData = new Float32Array(1);
            let changingBufferData = new Float32Array(8);

            let offset = [rand(-1, 1), rand(-1, 1)];
            let color = [
                rand(0, 1),
                rand(0, 1),
                rand(0, 1),
                1
            ];
            staticBufferData.set([0.4 /* scale */]);
            changingBufferData.set(color, 0);
            changingBufferData.set(offset, 4);

            let bindGroup = device.createBindGroup({
                layout: pipeline.getBindGroupLayout(0),
                label: 'bind group 0',
                entries: [
                    {
                        binding: 0,
                        resource: staticBuffer,
                    },
                    {
                        binding: 1,
                        resource: changingBuffer,
                    }
                ]
            });

            entities.push({
                bindGroup,
                staticBuffer,
                staticBufferData,
                changingBuffer,
                changingBufferData,
            });
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

        // pick different storage buffer on each draw
        for (let entity of entities) {
            device.queue.writeBuffer(entity.staticBuffer, 0, entity.staticBufferData);
            device.queue.writeBuffer(entity.changingBuffer, 0, entity.changingBufferData);
            pass.setBindGroup(0, entity.bindGroup);
            pass.draw(3);
        }

        pass.end()

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render();
})();
