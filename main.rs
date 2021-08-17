use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};

struct YCbCr {
    y: u8,
    cb: u8,
    cr: u8,
}

fn rgb_to_ycrcb(pixel: u32) -> YCbCr {
    let rf = ((pixel >> (8*2)) & 0xFF) as f32;
    let gf = ((pixel >> (8*1)) & 0xFF) as f32;
    let bf = ((pixel >> (8*0)) & 0xFF) as f32;
    let y  = (16.0  +  65.738*rf/256.0 + 129.057*gf/256.0 +  25.064*bf/256.0) as u8;
    let cb = (128.0 -  37.945*rf/256.0 -  74.494*gf/256.0 + 112.439*bf/256.0) as u8;
    let cr = (128.0 + 112.439*rf       -  94.154*gf/256.0 -  18.285*bf/256.0) as u8;
    YCbCr {y, cb, cr}
}

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const RECT_WIDTH: usize = 50;
const RECT_HEIGHT: usize = 60;
const FPS: usize = 30;
const DURATION: f32 = 2.0;
const OUTPUT_FILE_PATH: &str = "output.y4m";
const BACKGROUND: u32 = 0x181818;
const FOREGROUND: u32 = 0xAA9999;

fn fill_rect_rba(canvas: &mut [u32], canvas_stride: usize, rect: (i32, i32, u32, u32), color: u32) {
    let w = canvas_stride as i32;
    let h = canvas.len() as i32 / w;
    let (rx, ry, rw, rh) = rect;

    for dy in 0..rh {
        for dx in 0..rw {
            let x = rx + dx as i32;
            let y = ry + dy as i32;

            if (0..w).contains(&x) && (0..h).contains(&y) {
                canvas[(y as usize)*canvas_stride + x as usize] = color;
            }
        }
    }
}

#[derive(Default)]
struct Frame {
    y_plane: Vec<u8>,
    cb_plane: Vec<u8>,
    cr_plane: Vec<u8>,
}

fn canvas_as_frame(canvas: &[u32], frame: &mut Frame) {
    frame.y_plane.clear();
    frame.cb_plane.clear();
    frame.cr_plane.clear();
    for pixel in canvas {
        let YCbCr{y, cb, cr} = rgb_to_ycrcb(*pixel);
        frame.y_plane.push(y);
        frame.cb_plane.push(cb);
        frame.cr_plane.push(cr);
    }
}

// TODO: is this correct? are we saving the planes in the correct order?
fn save_frame(sink: &mut impl Write, frame: &Frame) -> io::Result<()> {
    writeln!(sink, "FRAME")?;
    sink.write(&frame.y_plane)?;
    sink.write(&frame.cr_plane)?;
    sink.write(&frame.cb_plane)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let frames_count: usize = (FPS as f32 * DURATION).floor() as usize;
    let mut canvas: [u32; WIDTH*HEIGHT] = [0; WIDTH*HEIGHT];
    let mut sink = BufWriter::new(File::create(OUTPUT_FILE_PATH)?);
    let mut rect_x: i32 = 0;
    let mut rect_y: i32 = 0;

    writeln!(&mut sink, "YUV4MPEG2 W{} H{} F{}:1 Ip A1:1 C444", WIDTH, HEIGHT, FPS)?;

    let mut frame = Frame::default();
    fill_rect_rba(&mut canvas, WIDTH, (0, 0, WIDTH as u32, HEIGHT as u32), BACKGROUND);
    for frame_index in 0..frames_count {
        fill_rect_rba(&mut canvas, WIDTH, (rect_x, rect_y, RECT_WIDTH as u32, RECT_HEIGHT as u32), FOREGROUND);
        canvas_as_frame(&canvas, &mut frame);
        save_frame(&mut sink, &frame)?;
        fill_rect_rba(&mut canvas, WIDTH, (rect_x, rect_y, RECT_WIDTH as u32, RECT_HEIGHT as u32), BACKGROUND);

        rect_x += 1;
        rect_y += 1;

        let progress = (frame_index as f32 / frames_count as f32 * 100.0).round() as usize;
        print!("Progress {}%\r", progress);
        io::stdout().flush()?;
    }

    println!("Generated {}", OUTPUT_FILE_PATH);
    Ok(())
}
