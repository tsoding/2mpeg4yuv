use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};
use std::slice;
use crate::sim::*;
use crate::config::*;
use crate::avi;
use crate::yuv4mpeg2;

const DELTA_TIME: f32 = 1.0 / FPS as f32;
const VIDEO_DURATION: f32 = 6.0;
const VIDEO_OUTPUT_PATH: &str = "output.y4m";
const AUDIO_OUTPUT_PATH: &str = "output.pcm";

#[derive(Default)]
struct FrameBGR24 {
    pixels: Vec<u8>,
}

fn canvas_as_frame_bgr24(canvas: &[u32], frame: &mut FrameBGR24) {
    frame.pixels.clear();
    for pixel in canvas {
        let r = ((pixel >> (8*2)) & 0xFF) as u8;
        let b = ((pixel >> (8*0)) & 0xFF) as u8;
        let g = ((pixel >> (8*1)) & 0xFF) as u8;
        frame.pixels.push(b);
        frame.pixels.push(g);
        frame.pixels.push(r);
    }
}

fn save_video_frame_avi(sink: &mut impl Write, frame: &FrameBGR24) -> io::Result<()> {
    avi::write_chunk(sink, &avi::Chunk {
        id: avi::FOURCC::from_str("00dc").unwrap(),
        content: &frame.pixels,
    })
}

fn save_audio_frame_avi(sink: &mut impl Write, sound: &[f32]) -> io::Result<()> {
    avi::write_chunk(sink, &avi::Chunk {
        id: avi::FOURCC::from_str("01wb").unwrap(),
        content: unsafe {
            slice::from_raw_parts(sound.as_ptr() as *const u8, sound.len()*size_of::<f32>())
        }
    })
}

pub fn main() -> io::Result<()> {
    let frames_count: usize = (FPS as f32 * VIDEO_DURATION).floor() as usize;
    let mut canvas = vec![0; WIDTH*HEIGHT];
    let mut sound = vec![0.0; (DELTA_TIME * SOUND_SAMPLE_RATE as f32).floor() as usize];
    let mut video_sink = BufWriter::new(File::create(VIDEO_OUTPUT_PATH)?);
    let mut audio_sink = BufWriter::new(File::create(AUDIO_OUTPUT_PATH)?);
    let mut state = State::new(WIDTH as f32, HEIGHT as f32);

    let mut y4m2 = yuv4mpeg2::Container::default();

    y4m2.start(&mut video_sink, WIDTH, HEIGHT, FPS)?;

    let mut movi: Vec<u8> = Vec::new();

    let mut frame_bgr24 = Default::default();
    for frame_index in 0..frames_count {
        canvas.fill(BACKGROUND);
        state.render(&mut canvas, WIDTH);
        y4m2.frame(&mut video_sink, &canvas)?;
        canvas_as_frame_bgr24(&canvas, &mut frame_bgr24);
        save_video_frame_avi(&mut movi, &frame_bgr24)?;

        sound.fill(0.0);
        state.sound(&mut sound, SOUND_SAMPLE_RATE);
        for sample in sound.iter() {
            audio_sink.write(&sample.to_le_bytes())?;
        }
        save_audio_frame_avi(&mut movi, &sound)?;

        state.update(DELTA_TIME);

        let progress = (frame_index as f32 / frames_count as f32 * 100.0).round() as usize;
        print!("Progress {}%\r", progress);
        io::stdout().flush()?;
    }

    avi::fabricate_avi_file("output.avi", &movi, frames_count)?;

    println!("Generated {}", VIDEO_OUTPUT_PATH);
    println!("Generated {}", AUDIO_OUTPUT_PATH);
    Ok(())
}
