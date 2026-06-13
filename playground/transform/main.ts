import shader from "./main.wgsl?raw";
import {mat3} from "wgpu-matrix";

const CANVAS_WIDTH = 500;
const CANVAS_HEIGHT = 500;

let mat = {
    projectionToClippedSpace() {
        let m = mat3.create();
        mat3.identity(m);
        let m3 = mat3.create();
        let m2 = mat3.create();
        let m1 = mat3.create();
        mat3.scaling([1, -1], m3);
        mat3.translation([-1, -1], m2);
        mat3.scaling([2. / CANVAS_WIDTH, 2. / CANVAS_HEIGHT], m1);
        mat3.multiply(m, m3, m);
        mat3.multiply(m, m2, m);
        mat3.multiply(m, m1, m);
        console.log(m);
        return m;
    }
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
    console.log("Used texture format: ", textureFormat);
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

    let vboData = Float32Array.from(
        [
            0, 0,
            50, 0,
            0, 200,
            50, 200,
            50, 50,
            200, 0,
            200, 50,
            150, 0,
            0, 150,
        ]
    );
    let iboData = Uint32Array.from([
        0, 2, 3, 0, 1, 3, 1, 5, 6, 1, 4, 6, 0, 7, 8,
    ])

    let matrix = mat.projectionToClippedSpace();
    mat3.translate(matrix, [250, 250], matrix);
    mat3.rotate(matrix, Math.PI / 4., matrix);

    let final = mat3.create();
    let s = mat3.create();
    mat3.scaling([0.5, 1.], s);
    mat3.multiply(s, matrix, final);

    let uniformData = Float32Array.from(final);

    let vbo = device.createBuffer({
        size: vboData.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });
    let ibo = device.createBuffer({
        size: iboData.byteLength,
        usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });
    let uniform = device.createBuffer({
        size: uniformData.byteLength,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        mappedAtCreation: false,
    });
    device.queue.writeBuffer(uniform, 0, uniformData, 0, uniformData.length);
    device.queue.writeBuffer(ibo, 0, iboData, 0, iboData.length);
    device.queue.writeBuffer(vbo, 0, vboData, 0, vboData.length);

    let pipeline = device.createRenderPipeline({
        layout: 'auto',
        vertex: {
            module: shaderModule,
            buffers: [
                {
                    arrayStride: 2 * 4,
                    stepMode: "vertex",
                    attributes: [
                        {shaderLocation: 0, offset: 0, format: 'float32x2'}
                    ]
                }
            ]
        },
        fragment: {
            module: shaderModule,
            targets: [
                {format: textureFormat}
            ]
        }
    })

    let bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
            {binding: 0, resource: uniform},
        ]
    })

    function render() {
        let encoder = device.createCommandEncoder();

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
        pass.setBindGroup(0, bindGroup);
        pass.setVertexBuffer(0, vbo);
        pass.setIndexBuffer(ibo, 'uint32', 0);
        pass.drawIndexed(iboData.length);
        pass.end()

        let commandBuffer = encoder.finish();
        device.queue.submit([commandBuffer]);
    }

    render();
})();
