import shader from "./main.wgsl?raw";

const GRID_X_NUM = 2;
const GRID_Y_NUM = 2;

let pixels = [
    255, 0, 0, 255, 0, 255, 0, 255,
    0, 0, 255, 255, 0, 255, 255, 128,
];
let textureData = new Uint8Array(GRID_X_NUM * GRID_Y_NUM * 4);
for (let i = 0; i < pixels.length; i++) {
    textureData[i] = pixels[i];
}

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

    let texture = device.createTexture({
        format: 'rgba8unorm',
        size: [GRID_X_NUM, GRID_Y_NUM],
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    });
    let sampler = device.createSampler({
        magFilter: 'nearest',
    });

    let bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
            {binding: 0, resource: sampler},
            {binding: 1, resource: texture},
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
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        device.queue.writeTexture({
            texture
        }, textureData, {
            bytesPerRow: GRID_X_NUM * 4,
            offset: 0,
        }, [GRID_X_NUM, GRID_Y_NUM]);
        pass.draw(6, 1);
        pass.end();

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render();
})();
