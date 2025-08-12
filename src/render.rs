use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};
use crate::sim::*;
use crate::config::*;
use crate::avi;
use crate::yuv4mpeg2;

const DELTA_TIME: f32 = 1.0 / FPS as f32;
const VIDEO_DURATION: f32 = 6.0;
const VIDEO_OUTPUT_PATH: &str = "output.y4m";
const AUDIO_OUTPUT_PATH: &str = "output.pcm";

pub fn main() -> io::Result<()> {
    let frames_count: usize = (FPS as f32 * VIDEO_DURATION).floor() as usize;
    let mut canvas = vec![0; WIDTH*HEIGHT];
    let mut sound = vec![0.0; (DELTA_TIME * SOUND_SAMPLE_RATE as f32).floor() as usize];
    let mut video_sink = BufWriter::new(File::create(VIDEO_OUTPUT_PATH)?);
    let mut audio_sink = BufWriter::new(File::create(AUDIO_OUTPUT_PATH)?);
    let mut state = State::new(WIDTH as f32, HEIGHT as f32);

    let mut y4m2 = yuv4mpeg2::Container::default();
    let mut avi = avi::Container::default();

    y4m2.start(&mut video_sink, WIDTH, HEIGHT, FPS)?;
    avi.start(WIDTH, HEIGHT, FPS);
    for frame_index in 0..frames_count {
        canvas.fill(BACKGROUND);
        state.render(&mut canvas, WIDTH);

        sound.fill(0.0);
        state.sound(&mut sound, SOUND_SAMPLE_RATE);

        y4m2.frame(&mut video_sink, &canvas)?;
        for sample in sound.iter() {
            audio_sink.write(&sample.to_le_bytes())?;
        }
        avi.frame(&canvas, &sound)?;

        state.update(DELTA_TIME);

        let progress = (frame_index as f32 / frames_count as f32 * 100.0).round() as usize;
        print!("Progress {}%\r", progress);
        io::stdout().flush()?;
    }

    avi.finish("output.avi")?;

    println!("Generated {}", VIDEO_OUTPUT_PATH);
    println!("Generated {}", AUDIO_OUTPUT_PATH);
    Ok(())
}
