export const rand = (min: number, max: number) => {
    if (min === undefined) {
        min = 0;
        max = 1;
    } else if (max === undefined) {
        max = min;
        min = 0;
    }
    return min + Math.random() * (max - min);
};

export function joinedPrimitivesIndexBuffer(vertexCount: number) {
    if (vertexCount < 3) throw 'requires: vertexCount >= 3';
    let tmp = [0, 1, 2];
    let count = vertexCount - 2;
    let buffer = new Uint32Array(count * 3);
    for (let i = 0; i < count; i++) {
        buffer.set(tmp, 3 * i);
        tmp = tmp.map(x => x + 1);
    }
    return buffer;
}

export interface ImageDataResult {
    data: Uint8Array;
    width: number;
    height: number;
}

export const getImageRawData = (url: string): Promise<ImageDataResult> => {
    return new Promise((resolve, reject) => {
        const img = new Image();
        // 处理跨域问题（如果图片来自其他域名）
        img.crossOrigin = 'anonymous';

        img.onload = () => {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');

            if (!ctx) {
                reject(new Error('无法创建 Canvas 上下文'));
                return;
            }

            canvas.width = img.width;
            canvas.height = img.height;

            // 将图片绘制到画布
            ctx.drawImage(img, 0, 0);

            // 获取像素数据 (RGBA)
            const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

            resolve({
                data: new Uint8Array(imageData.data.buffer), // 转化为 Uint8Array
                width: img.width,
                height: img.height
            });
        };

        img.onerror = reject;
        img.src = url;
    });
};
