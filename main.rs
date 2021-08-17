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
    let cr = (128.0 + 112.439*rf/256.0 -  94.154*gf/256.0 -  18.285*bf/256.0) as u8;
    YCbCr {y, cb, cr}
}

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const RECT_WIDTH: usize = 50 * 2;
const RECT_HEIGHT: usize = 50 * 2;
const RECT_VEL: f32 = 1000.0;
const RECTS_CAP: usize = 100;
const RECT_AREA_THRESHOLD: f32 = 10.0;
const FPS: usize = 60;
const DELTA_TIME: f32 = 1.0 / FPS as f32;
const VIDEO_DURATION: f32 = 16.0;
const OUTPUT_FILE_PATH: &str = "output.y4m";
const BACKGROUND: u32 = 0x181818;
const SPLIT_REDUCE_FACTOR: f32 = 0.90;

fn hsl2rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let mut r = (((0.0 + h*6.0).rem_euclid(6.0) - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let mut g = (((4.0 + h*6.0).rem_euclid(6.0) - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let mut b = (((2.0 + h*6.0).rem_euclid(6.0) - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let t = 1.0-(2.0*l-1.0).abs();
    r = l + s * (r - 0.5) * t;
    g = l + s * (g - 0.5) * t;
    b = l + s * (b - 0.5) * t;
    (r, g, b)
}

fn fill_gay_rect_rba(canvas: &mut [u32], canvas_stride: usize, rect: (i32, i32, u32, u32)) {
    let w = canvas_stride as i32;
    let h = canvas.len() as i32 / w;
    let (rx, ry, rw, rh) = rect;

    for dy in 0..rh {
        for dx in 0..rw {
            let x = rx + dx as i32;
            let y = ry + dy as i32;

            if (0..w).contains(&x) && (0..h).contains(&y) {
                let u = x as f32 / w as f32;
                let v = y as f32 / h as f32;
                let (rf, gf, bf) = hsl2rgb((u + v) * 2.0, 1.0, 0.80);
                canvas[(y as usize)*canvas_stride + x as usize] =
                    ((rf * 255.0) as u32) << (8*2) |
                    ((gf * 255.0) as u32) << (8*1) |
                    ((bf * 255.0) as u32) << (8*0);
            }
        }
    }
}

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

fn save_frame(sink: &mut impl Write, frame: &Frame) -> io::Result<()> {
    writeln!(sink, "FRAME")?;
    sink.write(&frame.y_plane)?;
    sink.write(&frame.cb_plane)?;
    sink.write(&frame.cr_plane)?;
    Ok(())
}

struct Rect {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    w: f32,
    h: f32,
}

#[derive(Copy, Clone)]
enum Orient {
    Vert,
    Horz
}

impl Rect {
    fn hitbox(&self) -> (i32, i32, u32, u32) {
        (self.x as i32, self.y as i32, self.w as u32, self.h as u32)
    }

    fn area(&self) -> f32 {
        self.w * self.h
    }

    fn split(self, orient: Orient) -> (Rect, Rect) {
        use Orient::*;
        match orient {
            Vert => {
                let left = Rect {
                    x: self.x,
                    y: self.y,
                    dx: self.dx,
                    dy: -self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                };
                let right =  Rect {
                    x: self.x,
                    y: self.y,
                    dx: -self.dx,
                    dy: -self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                };
                (left, right)
            },
            Horz => {
                let left = Rect {
                    x: self.x,
                    y: self.y,
                    dx: -self.dx,
                    dy: self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                };
                let right =  Rect {
                    x: self.x,
                    y: self.y,
                    dx: -self.dx,
                    dy: -self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                };
                (left, right)
            },
        }
    }

    fn update(&mut self) -> Option<Orient> {
        let nx = self.x + self.dx * RECT_VEL * DELTA_TIME;
        let ny = self.y + self.dy * RECT_VEL * DELTA_TIME;

        if nx + self.w as f32 >= WIDTH as f32 || nx <= 0.0 {
            return Some(Orient::Horz);
        }

        if ny + self.h as f32 >= HEIGHT as f32 || ny <= 0.0 {
            return Some(Orient::Vert);
        }

        self.x = nx;
        self.y = ny;

        None
    }
}

fn main() -> io::Result<()> {
    let frames_count: usize = (FPS as f32 * VIDEO_DURATION).floor() as usize;
    let mut canvas = vec![0; WIDTH*HEIGHT];
    let mut sink = BufWriter::new(File::create(OUTPUT_FILE_PATH)?);
    let mut rects = Vec::<Rect>::new();
    let mut to_split = Vec::<(usize, Orient)>::new();
    rects.push(Rect {
        x: 30.0, y: 100.0,
        dx: 0.7, dy: 0.8,
        w: RECT_WIDTH as f32, h: RECT_HEIGHT as f32
    });

    writeln!(&mut sink, "YUV4MPEG2 W{} H{} F{}:1 Ip A1:1 C444", WIDTH, HEIGHT, FPS)?;

    let mut frame = Frame::default();
    fill_rect_rba(&mut canvas, WIDTH, (0, 0, WIDTH as u32, HEIGHT as u32), BACKGROUND);
    for frame_index in 0..frames_count {
        for rect in rects.iter() {
            fill_gay_rect_rba(&mut canvas, WIDTH, rect.hitbox());
        }
        canvas_as_frame(&canvas, &mut frame);
        save_frame(&mut sink, &frame)?;
        for rect in rects.iter() {
            fill_rect_rba(&mut canvas, WIDTH, rect.hitbox(), BACKGROUND);
        }

        for (index, rect) in rects.iter_mut().enumerate() {
            if let Some(orient) = rect.update() {
                to_split.push((index, orient));
            }
        }

        for (index, orient) in to_split.iter().rev() {
            let rect = rects.remove(*index);
            let (left, right) = rect.split(*orient);

            if rects.len() < RECTS_CAP && left.area() >= RECT_AREA_THRESHOLD {
                rects.push(left);
            }
            if rects.len() < RECTS_CAP && right.area() >= RECT_AREA_THRESHOLD {
                rects.push(right);
            }
        }
        to_split.clear();

        let progress = (frame_index as f32 / frames_count as f32 * 100.0).round() as usize;
        print!("Progress {}%\r", progress);
        io::stdout().flush()?;
    }

    println!("Generated {}", OUTPUT_FILE_PATH);
    Ok(())
}
