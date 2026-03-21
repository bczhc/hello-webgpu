use image::{Rgb, RgbImage};

fn main() {
    let width = 2560;
    let height = 1440;
    let path = "/home/bczhc/a.png";

    let mut img = RgbImage::from_fn(width, height, |_, _| Rgb([255, 255, 255]));

    let black = Rgb([0, 0, 0]);
    let red = Rgb([255, 0, 0]);
    let green = Rgb([0, 255, 0]);
    let blue = Rgb([0, 0, 255]);
    let yellow = Rgb([255, 255, 0]);
    let cyan = Rgb([0, 255, 255]);
    let magenta = Rgb([255, 0, 255]);

    // 1. 绘制大叉叉 (精准单像素)
    for i in 0..width {
        let j = (i * height) / width;
        if j < height { img.put_pixel(i, j, black); }
        let j2 = height - 1 - (i * height) / width;
        if j2 < height { img.put_pixel(i, j2, black); }
    }

    // 2. 遍历全图绘制规律性彩色模式
    // 我们将屏幕分成若干区域，在不干扰大叉叉的情况下填充规律图案
    for x in 0..width {
        for y in 0..height {
            // 避开大叉叉附近的像素（保留5像素宽度的缓冲带）
            let d1 = (x as i32 * height as i32 / width as i32 - y as i32).abs();
            let d2 = ((height as i32 - 1 - (x as i32 * height as i32 / width as i32)) - y as i32).abs();
            if d1 < 3 || d2 < 3 { continue; }

            // --- 规律模式分布 ---

            // 左上区域：红绿 2x2 交错 (检测 4 像素对齐)
            if x > 100 && x < 600 && y > 450 && y < 850 {
                if (x / 2 + y / 2) % 2 == 0 {
                    img.put_pixel(x, y, red);
                } else {
                    img.put_pixel(x, y, green);
                }
            }

            // 右上区域：蓝黄 3x3 交错 (检测非 2 幂次缩放)
            if x > width - 600 && x < width - 100 && y > 450 && y < 850 {
                if (x / 3 + y / 3) % 2 == 0 {
                    img.put_pixel(x, y, blue);
                } else {
                    img.put_pixel(x, y, yellow);
                }
            }

            // 中间顶部：RGB 三色横向细条纹 (1像素宽)
            if x > width / 2 - 250 && x < width / 2 + 250 && y > 50 && y < 250 {
                match x % 3 {
                    0 => img.put_pixel(x, y, red),
                    1 => img.put_pixel(x, y, green),
                    _ => img.put_pixel(x, y, blue),
                }
            }

            // 中间底部：CMY 三色纵向细条纹 (1像素高)
            if x > width / 2 - 250 && x < width / 2 + 250 && y > height - 250 && y < height - 50 {
                match y % 3 {
                    0 => img.put_pixel(x, y, cyan),
                    1 => img.put_pixel(x, y, magenta),
                    _ => img.put_pixel(x, y, yellow),
                }
            }

            // 剩下的空白处加点微小的 1x1 灰度噪点点阵，增加测试密度
            if x % 20 == 0 && y % 20 == 0 {
                img.put_pixel(x, y, Rgb([180, 180, 180]));
            }
        }
    }

    // 3. 保存
    match img.save(path) {
        Ok(_) => println!("测试图像已生成: {}", path),
        Err(e) => eprintln!("错误: {}", e),
    }
}