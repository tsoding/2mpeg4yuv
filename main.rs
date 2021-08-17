use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};

#[allow(dead_code)]
fn write_ppm_image(sink: &mut impl Write, pixels: &[u32], stride: usize) -> io::Result<()> {
    let w = stride;
    let h = pixels.len() / stride;
    write!(sink, "P6\n{} {} 255\n", w, h)?;
    for pixel in pixels {
        // 0xRRGGBB
        let r = ((pixel >> 8*2) & 0xFF) as u8;
        let g = ((pixel >> 8*1) & 0xFF) as u8;
        let b = ((pixel >> 8*0) & 0xFF) as u8;
        sink.write(&[r, g, b])?;
    }
    Ok(())
}

#[allow(dead_code)]
fn generate_uv_pattern(pixels: &mut [u32], stride: usize) {
    let w = stride;
    let h = pixels.len() / w;
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32;
            let v = y as f32 / h as f32;
            let r = (u*255.0) as u32;
            let g = (v*255.0) as u32;
            pixels[y*w + x] = (r << 8*2) | (g << 8*1);
        }
    }
}

fn different_kind_of_pattern(pixels: &mut [u32], stride: usize, luma: u8) {
    let w = stride;
    let h = pixels.len() / w;
    for y in 0..h {
        for x in 0..w {
            let cr = x as u32 & 0xFF;
            let cb = y as u32 & 0xFF;
            pixels[y*stride + x] = ((luma as u32) << (8*2)) | (cr << (8*1)) | (cb << (8*0));
        }
    }
}

fn write_yuv4mpeg2_frame_c444(sink: &mut impl Write, pixels: &[u32]) -> io::Result<()> {
    writeln!(sink, "FRAME")?;
    for pixel in pixels {
        let y  = ((pixel >> 8*2) & 0xFF) as u8;
        let cr = ((pixel >> 8*1) & 0xFF) as u8;
        let cb = ((pixel >> 8*0) & 0xFF) as u8;
        sink.write(&[y, cr, cb])?;
    }
    Ok(())
}

fn rgb_to_ycrcb((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let rf = r as f32;
    let gf = g as f32;
    let bf = b as f32;
    let y  = 16.0  + (  65.738*rf + 129.057*gf +  25.064*bf)/256.0;
    let cb = 128.0 + (- 37.945*rf -  74.494*gf + 112.439*bf)/256.0;
    let cr = 128.0 + 112.439*rf + ( -  94.154*gf -  18.285*bf)/256.0;
    return (y as u8, cb as u8, cr as u8)
}

fn main() -> io::Result<()> {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 600;
    const FPS: usize = 30;
    const DURATION: f32 = 2.0;
    let frames_count: usize = (FPS as f32 * DURATION).floor() as usize;

    let mut pixels: [u32; WIDTH*HEIGHT] = [0; WIDTH*HEIGHT];
    let output_file_path = "output.y4m";
    let mut sink = BufWriter::new(File::create(output_file_path)?);
    writeln!(&mut sink, "YUV4MPEG2 W{} H{} F{}:1 Ip A1:1 C444", WIDTH, HEIGHT, FPS)?;

    let (red_y, red_cb, red_cr) = rgb_to_ycrcb((0, 0, 255));
    for frame in 0..frames_count {
        writeln!(&mut sink, "FRAME")?;
        for _ in 0..WIDTH*HEIGHT {
            sink.write(&[red_y]);
        }
        for _ in 0..WIDTH*HEIGHT {
            sink.write(&[red_cr]);
        }
        for _ in 0..WIDTH*HEIGHT {
            sink.write(&[red_cb]);
        }

        let progress = (frame as f32 / frames_count as f32 * 100.0).round() as usize;
        print!("Progress {}%\r", progress);
        io::stdout().flush()?;
    }

    println!("Generated {}", output_file_path);
    Ok(())
}
