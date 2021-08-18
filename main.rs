mod sim;

use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};
use sim::*;

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
const FPS: usize = 60;
const DELTA_TIME: f32 = 1.0 / FPS as f32;
const VIDEO_DURATION: f32 = 16.0;
const VIDEO_OUTPUT_PATH: &str = "output.y4m";
const AUDIO_OUTPUT_PATH: &str = "output.pcm";
const BACKGROUND: u32 = 0x181818;
const SOUND_SAMPLE_RATE: usize = 48000;

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

fn main() -> io::Result<()> {
    let frames_count: usize = (FPS as f32 * VIDEO_DURATION).floor() as usize;
    let mut canvas = vec![0; WIDTH*HEIGHT];
    let mut sound = vec![0.0; (DELTA_TIME * SOUND_SAMPLE_RATE as f32).floor() as usize];
    let mut video_sink = BufWriter::new(File::create(VIDEO_OUTPUT_PATH)?);
    let mut audio_sink = BufWriter::new(File::create(AUDIO_OUTPUT_PATH)?);
    let mut state = State::new(WIDTH as f32, HEIGHT as f32);

    writeln!(&mut video_sink, "YUV4MPEG2 W{} H{} F{}:1 Ip A1:1 C444", WIDTH, HEIGHT, FPS)?;

    let mut frame = Frame::default();
    for frame_index in 0..frames_count {
        canvas.fill(BACKGROUND);
        state.render(&mut canvas, WIDTH);
        canvas_as_frame(&canvas, &mut frame);
        save_frame(&mut video_sink, &frame)?;

        sound.fill(0.0);
        state.update(DELTA_TIME, &mut sound, SOUND_SAMPLE_RATE);
        for sample in sound.iter() {
            audio_sink.write(&sample.to_le_bytes());
        }

        let progress = (frame_index as f32 / frames_count as f32 * 100.0).round() as usize;
        print!("Progress {}%\r", progress);
        io::stdout().flush()?;
    }

    println!("Generated {}", VIDEO_OUTPUT_PATH);
    println!("Generated {}", AUDIO_OUTPUT_PATH);
    Ok(())
}

// TODO: Real-Time generation version
