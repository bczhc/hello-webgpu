import shaderCode from './main.wgsl?raw';
import {rand} from "../utils";

(async () => {
    let canvas = document.querySelector('canvas')!!;
    let context = canvas.getContext('webgpu')!!;

    let adapter = (await navigator.gpu.requestAdapter())!!;
    let device = (await adapter.requestDevice());
    let preferredFormat = navigator.gpu.getPreferredCanvasFormat();
    context.configure({
        device,
        format: preferredFormat,
    });

    let shaderModule = device.createShaderModule({
        code: shaderCode,
        label: 'shader 1',
    });

    let pipeline = device.createRenderPipeline({
        layout: 'auto',
        vertex: {
            module: shaderModule,
            buffers: [
                {
                    // slot 0
                    stepMode: 'vertex',
                    arrayStride: 5 * 4,
                    attributes: [
                        {
                            shaderLocation: 0,
                            offset: 0,
                            format: 'float32x2',
                        },
                        {
                            shaderLocation: 1,
                            offset: 2 * 4,
                            format: 'float32x3',
                        }
                    ]
                },
                {
                    // slot 1
                    stepMode: 'instance',
                    arrayStride: 3 * 4,
                    attributes: [
                        {
                            shaderLocation: 2,
                            offset: 0,
                            format: 'float32x2'
                        },
                        {
                            shaderLocation: 3,
                            offset: 2 * 4,
                            format: 'float32',
                        }
                    ]
                }
            ]
        },
        fragment: {
            module: shaderModule,
            targets: [{format: preferredFormat}]
        },
    });

    const OBJECT_COUNT = 10;
    const BASIC_VERTEX_DATA = [
        0, 0.5, 1, 0, 0,
        -0.5, -0.5, 0, 1, 0,
        0.5, -0.5, 0, 0, 1,
    ];
    // let vertexData = new Float32Array(BASIC_VERTEX_DATA.length * OBJECT_COUNT);
    // for (let i = 0; i < OBJECT_COUNT; i++) {
    //     let cloned = BASIC_VERTEX_DATA.slice();
    //     for (let j = 0; j < cloned.length; j++) {
    //         cloned[j] += rand(-1, 1);
    //     }
    //     vertexData.set(cloned.slice(0, 5), 15 * i);
    //     vertexData.set(cloned.slice(5, 10), 15 * i + 5);
    //     vertexData.set(cloned.slice(10, 15), 15 * i + 10);
    // }
    let vertexData = new Float32Array(BASIC_VERTEX_DATA);

    let vertexBuffer = device.createBuffer({
        size: vertexData.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });

    // row layout: (offset: vec2f, scale: f32)
    let instanceVertexData = new Float32Array(3 * OBJECT_COUNT);
    for (let i = 0; i < OBJECT_COUNT; i++) {
        let data = [rand(-1, 1), rand(-1, 1), rand(0.2, 1)];
        instanceVertexData.set(data, 3 * i);
    }

    let instanceVertexBuffer = device.createBuffer({
        size: instanceVertexData.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });

    function render() {
        let encoder = device.createCommandEncoder();

        let pass = encoder.beginRenderPass({
            colorAttachments: [{
                view: context.getCurrentTexture(),
                loadOp: 'clear',
                storeOp: 'store',
                clearValue: [0.3, 0.3, 0.3, 1] /* gray */,
            }],
        });

        pass.setPipeline(pipeline);
        device.queue.writeBuffer(vertexBuffer, 0, vertexData);
        device.queue.writeBuffer(instanceVertexBuffer, 0, instanceVertexData);
        pass.setVertexBuffer(0, vertexBuffer);
        pass.setVertexBuffer(1, instanceVertexBuffer);
        pass.draw(3, OBJECT_COUNT);
        pass.end();
        let commandBuffer = encoder.finish();

        device.queue.submit([commandBuffer]);
    }

    render();
})();
