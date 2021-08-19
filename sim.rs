const SPLIT_REDUCE_FACTOR: f32 = 0.90;
const RECT_VEL: f32 = 1000.0;
const RECT_WIDTH: usize = 100;
const RECT_HEIGHT: usize = 100;
const RECTS_CAP: usize = 100;
const RECT_NOTE_STEP: i32 = 3;
const RECT_AREA_THRESHOLD: f32 = 1853.0;
    // RECT_WIDTH as f32 * RECT_HEIGHT as f32 * (SPLIT_REDUCE_FACTOR.powf(10.0 * 2.0));
const BEEP_DURATION: f32 = 0.2;
const BEEP_FREQ: f32 = 440.0;
const BEEP_VOLUME: f32 = 0.05;

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

fn fill_gay_rectangle_rba(canvas: &mut [u32], canvas_stride: usize, rect: (i32, i32, u32, u32)) {
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

struct Rect {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    w: f32,
    h: f32,
    note: i32,
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

    fn bounce(mut self, orient: Orient) -> Rect {
        use self::Orient::*;
        match orient {
            Vert => self.dy *= -1.0,
            Horz => self.dx *= -1.0,
        }
        self
    }

    fn split(self, orient: Orient) -> (Rect, Rect) {
        use self::Orient::*;
        match orient {
            Vert => {
                let left = Rect {
                    x: self.x,
                    y: self.y,
                    dx: self.dx,
                    dy: -self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                    note: self.note + RECT_NOTE_STEP,
                };
                let right =  Rect {
                    x: self.x,
                    y: self.y,
                    dx: -self.dx,
                    dy: -self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                    note: self.note + RECT_NOTE_STEP,
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
                    note: self.note + RECT_NOTE_STEP,
                };
                let right =  Rect {
                    x: self.x,
                    y: self.y,
                    dx: -self.dx,
                    dy: -self.dy,
                    w: self.w * SPLIT_REDUCE_FACTOR,
                    h: self.h * SPLIT_REDUCE_FACTOR,
                    note: self.note + RECT_NOTE_STEP,
                };
                (left, right)
            },
        }
    }

    fn update(&mut self, delta_time: f32, width: f32, height: f32) -> Option<Orient> {
        let nx = self.x + self.dx * RECT_VEL * delta_time;
        let ny = self.y + self.dy * RECT_VEL * delta_time;

        if nx + self.w as f32 >= width as f32 || nx <= 0.0 {
            return Some(Orient::Horz);
        }

        if ny + self.h as f32 >= height as f32 || ny <= 0.0 {
            return Some(Orient::Vert);
        }

        self.x = nx;
        self.y = ny;

        None
    }
}

struct Beep {
    freq: f32,
    duration: f32,
}

#[derive(Default)]
struct Beeper {
    beeps: Vec<Beep>,
    time: f32,
}

impl Beeper {
    fn beep(&mut self, freq: f32, duration: f32) {
        self.beeps.push(Beep{freq, duration})
    }

    fn update(&mut self, samples: &mut [f32], sample_rate: usize) {
        use std::f32::consts::PI;

        let sample_step = 1.0 / sample_rate as f32;
        for sample in samples {
            *sample = 0.0;
            for beep in self.beeps.iter_mut() {
                if beep.duration > 0.0 {
                    let p = beep.duration / BEEP_DURATION;
                    let fader = if p >= 0.95 {
                        1.0 - (p - 0.95)/0.05
                    } else if p <= 0.05 {
                        p/0.05
                    } else {
                        1.0
                    };

                    *sample += (2.0 * PI * beep.freq * self.time).sin() * BEEP_VOLUME * fader;
                    beep.duration -= sample_step;
                }
            }

            self.time += sample_step;
        }
        self.beeps.retain(|beep| beep.duration > 0.0);
    }
}

pub struct State {
    rects: Vec<Rect>,
    to_split: Vec<(usize, Orient)>,
    width: f32,
    height: f32,
    beeper: Beeper,
}

fn freq_of_note(note: i32) -> f32 {
    440.0 * (2.0f32).powf(1.0 / 12.0).powf(note as f32)
}

impl State {
    pub fn new(width: f32, height: f32) -> Self {
        let mut rects = Vec::new();
        rects.push(Rect {
            x: 30.0, y: 100.0,
            dx: 0.7, dy: 0.8,
            w: RECT_WIDTH as f32, h: RECT_HEIGHT as f32,
            note: -24,
        });
        Self {
            rects,
            to_split: Vec::new(),
            width,
            height,
            beeper: Beeper::default()
        }
    }

    pub fn render(&self, canvas: &mut [u32], canvas_stride: usize) {
        for rect in self.rects.iter() {
            fill_gay_rectangle_rba(canvas, canvas_stride, rect.hitbox());
        }
    }

    pub fn sound(&mut self, sample: &mut [f32], sample_rate: usize) {
        self.beeper.update(sample, sample_rate);
    }

    pub fn update(&mut self, delta_time: f32) {
        for (index, rect) in self.rects.iter_mut().enumerate() {
            if let Some(orient) = rect.update(delta_time, self.width, self.height) {
                self.to_split.push((index, orient));
            }
        }

        for (index, orient) in self.to_split.iter().rev() {
            let rect = self.rects.remove(*index);

            self.beeper.beep(freq_of_note(rect.note), BEEP_DURATION);

            let (left, right) = rect.split(*orient);

            if self.rects.len() < RECTS_CAP && left.area() >= RECT_AREA_THRESHOLD {
                self.rects.push(left);
            }
            if self.rects.len() < RECTS_CAP && right.area() >= RECT_AREA_THRESHOLD {
                self.rects.push(right);
            }
        }
        self.to_split.clear();

        // TODO: stereo sound depending on the location of the collision
    }
}
