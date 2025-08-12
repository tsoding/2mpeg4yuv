//! YUV4MPEG2 container
use std::io::{self, Write};

struct YCbCr {
    y: u8,
    cb: u8,
    cr: u8,
}

impl YCbCr {
    fn from_rgb(pixel: u32) -> Self {
        let rf = ((pixel >> (8*2)) & 0xFF) as f32;
        let gf = ((pixel >> (8*1)) & 0xFF) as f32;
        let bf = ((pixel >> (8*0)) & 0xFF) as f32;
        let y  = (16.0  +  65.738*rf/256.0 + 129.057*gf/256.0 +  25.064*bf/256.0) as u8;
        let cb = (128.0 -  37.945*rf/256.0 -  74.494*gf/256.0 + 112.439*bf/256.0) as u8;
        let cr = (128.0 + 112.439*rf/256.0 -  94.154*gf/256.0 -  18.285*bf/256.0) as u8;
        Self {y, cb, cr}
    }
}


#[derive(Default)]
struct Frame {
    y_plane: Vec<u8>,
    cb_plane: Vec<u8>,
    cr_plane: Vec<u8>,
}

impl Frame {
    fn from_canvas(&mut self, canvas: &[u32]) {
        self.y_plane.clear();
        self.cb_plane.clear();
        self.cr_plane.clear();
        for pixel in canvas {
            let YCbCr{y, cb, cr} = YCbCr::from_rgb(*pixel);
            self.y_plane.push(y);
            self.cb_plane.push(cb);
            self.cr_plane.push(cr);
        }
    }
}

#[derive(Default)]
pub struct Container {
    frame: Frame,
}

impl Container {
    /// Prepare the metadata header for the YUV4MPEG2 container
    pub fn start(&mut self, sink: &mut impl Write, width: usize, height: usize, fps: usize) -> io::Result<()> {
        writeln!(sink, "YUV4MPEG2 W{} H{} F{}:1 Ip A1:1 C444", width, height, fps)
    }

    /// Emit a frame into YUV4MPEG2 container
    pub fn frame(&mut self, sink: &mut impl Write, canvas: &[u32]) -> io::Result<()> {
        self.frame.from_canvas(canvas);
        writeln!(sink, "FRAME")?;
        sink.write(&self.frame.y_plane)?;
        sink.write(&self.frame.cb_plane)?;
        sink.write(&self.frame.cr_plane)?;
        Ok(())
    }
}
