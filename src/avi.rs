//! AVI container
//!
//! https://learn.microsoft.com/en-us/windows/win32/directshow/avi-file-format
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use std::fs;
use std::result;
use std::mem;
use std::convert::*;
use std::fmt;
use std::io::{self, Write};
use std::slice;

type Result<T> = result::Result<T, ()>;

fn chop_u32(content: &mut &[u8]) -> u32 {
    let result = u32::from_le_bytes((*content)[0..4].try_into().unwrap());
    (*content) = &(*content)[4..];
    result
}

enum Entry<'a> {
    Chunk(Chunk<'a>),
    List(List<'a>),
}

fn chop_entry<'a>(content: &mut &'a [u8]) -> Entry<'a> {
    let id = chop_u32(content);
    if id != LIST {
        let size = chop_u32(content) as usize;
        let size_padded = pad_size(size, 2);
        let chunk = Chunk {
            id: FOURCC::from_u32(id),
            content: &(*content)[0..size],
        };
        (*content) = &(*content)[size_padded..];
        Entry::Chunk(chunk)
    } else {
        let size = chop_u32(content) as usize - size_of::<u32>();
        let r#type = FOURCC::from_u32(chop_u32(content));
        let list = List {
            r#type,
            content: &(*content)[0..size],
        };
        (*content) = &(*content)[size..];
        Entry::List(list)
    }
}

#[derive(Debug)]
struct Chunk<'a> {
    id: FOURCC,
    content: &'a [u8]
}

fn chop_chunk<'a>(content: &mut &'a [u8]) -> Chunk<'a> {
    match chop_entry(content) {
        Entry::Chunk(chunk) => chunk,
        _ => unreachable!("Not chunk"),
    }
}

#[derive(Debug)]
struct List<'a> {
    r#type: FOURCC,
    content: &'a [u8],
}

fn chop_list<'a>(content: &mut &'a [u8]) -> List<'a> {
    match chop_entry(content) {
        Entry::List(list) => list,
        _ => unreachable!("Not list"),
    }
}

const RIFF: u32 = 0x46464952;
const AVI_: u32 = 0x20495641;
const LIST: u32 = 0x5453494c;

const hdrl: u32 = 0x6c726468;
const avih: u32 = 0x68697661;   // AVI Main Header
const strh: u32 = 0x68727473;   // AVI Stream Header
const strf: u32 = 0x66727473;
const strl: u32 = 0x6c727473;
const vids: u32 = 0x73646976;
const auds: u32 = 0x73647561;
const movi: u32 = 0x69766f6d;

fn pad_size(size: usize, width: usize) -> usize {
    (size + width - 1)/width*width
}

type WORD = u16;
type DWORD = u32;
type LONG = i32;
type SHORT = i16;

struct FOURCC([u8; 4]);

impl FOURCC {
    fn from_str(s: &str) -> Option<Self> {
        s.as_bytes().try_into().ok().map(FOURCC)
    }

    const fn from_u32(x: u32) -> Self {
        Self(x.to_le_bytes())
    }

    const fn to_u32(&self) -> u32 {
        let Self(bytes) = self;
        u32::from_le_bytes(*bytes)
    }
}

impl fmt::Display for FOURCC {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        let FOURCC(bytes) = self;
        write!(fmt, "0x{bytes:x} \"{str}\"",
               bytes = u32::from_le_bytes(*bytes),
               str = str::from_utf8(bytes).unwrap())
    }
}

impl fmt::Debug for FOURCC {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        fmt::Display::fmt(self, fmt)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RECT {
    left: SHORT,
    top: SHORT,
    right: SHORT,
    bottom: SHORT,
}

#[derive(Debug)]
#[repr(C)]
struct AVIMainHeader {
    // fcc: FOURCC,
    // cb: DWORD,
    dwMicroSecPerFrame: DWORD,
    dwMaxBytesPerSec: DWORD,
    dwPaddingGranularity: DWORD,
    dwFlags: DWORD,
    dwTotalFrames: DWORD,
    dwInitialFrames: DWORD,
    dwStreams: DWORD,
    dwSuggestedBufferSize: DWORD,
    dwWidth: DWORD,
    dwHeight: DWORD,
    dwReserved: [DWORD; 4],
}

#[derive(Debug)]
#[repr(C)]
struct AVIStreamHeader {
    fccType: FOURCC,
    fccHandler: FOURCC,
    dwFlags: DWORD,
    wPriority: WORD,
    wLanguage: WORD,
    dwInitialFrames: DWORD,
    dwScale: DWORD,
    dwRate: DWORD,
    dwStart: DWORD,
    dwLength: DWORD,
    dwSuggestedBufferSize: DWORD,
    dwQuality: DWORD,
    dwSampleSize: DWORD,
    rcFrame: RECT,
}

#[derive(Debug)]
#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: DWORD,
    biWidth: LONG,
    biHeight: LONG,
    biPlanes: WORD,
    biBitCount: WORD,
    biCompression: DWORD,
    biSizeImage: DWORD,
    biXPelsPerMeter: LONG,
    biYPelsPerMeter: LONG,
    biClrUsed: DWORD,
    biClrImportant: DWORD,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(1))]
struct WAVEFORMATEX {
    wFormatTag: WORD,
    nChannels: WORD,
    nSamplesPerSec: DWORD,
    nAvgBytesPerSec: DWORD,
    nBlockAlign: WORD,
    wBitsPerSample: WORD,
    cbSize: WORD,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(1))]
struct GUID {
    Data1: u32,
    Data2: u16,
    Data3: u16,
    Data4: [u8; 8],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(1))]
struct WAVEFORMATEXTENSIBLE {
    Format: WAVEFORMATEX,
    Samples: WORD,//SamplesUnion,
    dwChannelMask: DWORD,
    SubFormat: GUID,
}

#[derive(Debug)]
#[repr(C)]
enum SamplesUnion {
    wValidBitsPerSample(WORD),
    wSamplesPerBlock(WORD),
    wReserved(WORD),
}

struct Indent<T: fmt::Debug> {
    value: T,
    indent: usize,
}

impl<T: fmt::Debug> fmt::Debug for Indent<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        let Indent {value, indent} = self;
        if fmt.alternate() {
            for line in format!("{value:#?}").lines() {
                writeln!(fmt, "{empty:indent$}{line}", empty = "")?;
            }
        } else {
            write!(fmt, "{empty:indent$}{value:?}", empty = "")?;
        }
        Ok(())
    }
}

struct Padding(usize);

impl fmt::Display for Padding {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        let Padding(padding) = self;
        write!(fmt, "{empty:padding$}", empty = "")
    }
}

fn transmute_struct_to_chunk<'a, T>(id: FOURCC, s: &'a T) -> Chunk<'a> {
    let content = unsafe {
        slice::from_raw_parts(mem::transmute::<&'a T, *const u8>(s), size_of::<T>())
    };
    Chunk {id, content}
}

fn transmute_chunk_to_struct<'a, T>(chunk: &Chunk<'a>) -> &'a T {
    let expected = size_of::<T>();
    let actual = chunk.content.len() as usize;
    assert!(
        expected <= actual,
        "Unexpected chunk size. Expected size of structure {}, but got {}",
        expected, actual
    );
    unsafe {
        mem::transmute::<*const u8, &'a T>(chunk.content.as_ptr())
    }
}

fn parse_strl(mut content: &[u8], level: usize) {
    let chunk = chop_chunk(&mut content);
    assert!(chunk.id.to_u32() == strh);
    let header = transmute_chunk_to_struct::<AVIStreamHeader>(&chunk);
    print!("{:#?}", Indent {value: header, indent: level*2,});

    let chunk = chop_chunk(&mut content);
    dump_chunk(&chunk, level);
    assert!(chunk.id.to_u32() == strf);
    match header.fccType.to_u32() {
        vids => {
            let header = transmute_chunk_to_struct::<BITMAPINFOHEADER>(&chunk);
            print!("{:#?}", Indent {value: header, indent: level*2});
        }
        auds => {
            let header = transmute_chunk_to_struct::<WAVEFORMATEXTENSIBLE>(&chunk);
            print!("{:#?}", Indent {value: header, indent: level*2});
            println!("{}sizeof(WAVEFORMATEX) == {}", Padding(level*2), size_of::<WAVEFORMATEXTENSIBLE>());
        }
        _ => unreachable!("{}", header.fccType)
    }
}

fn parse_hdrl(mut content: &[u8], level: usize) {
    let chunk = chop_chunk(&mut content);
    assert!(chunk.id.to_u32() == avih);
    let header = transmute_chunk_to_struct::<AVIMainHeader>(&chunk);
    print!("{:#?}", Indent {value: header, indent: level*2});

    'strl: while content.len() > 0 {
        match chop_entry(&mut content) {
            Entry::List(list) => {
                if list.r#type.to_u32() != strl {
                    break 'strl
                }
                dump_list(&list, level);
                parse_strl(list.content, level + 1);
            }
            _ => break 'strl
        }
    }
}

fn dump_all_chunks_with_id(mut content: &[u8], id: u32, level: usize, file_path: &str) {
    let mut buffer: Vec<u8> = Default::default();

    let mut chunk_count = 0;
    while content.len() > 0 {
        let chunk = chop_chunk(&mut content);
        if chunk.id.to_u32() == id {
            let _ = buffer.write(chunk.content);
            chunk_count += 1;
        }
    }

    fs::write(file_path, &buffer).unwrap();
    println!("{}Generated {file_path}. {chunk_count} chunks with {id}", Padding(level*2), id = FOURCC::from_u32(id));
}

fn parse_movi(content: &[u8], level: usize) {
    const _00dc: u32 = 0x63643030;
    const _01wb: u32 = 0x62773130;
    dump_all_chunks_with_id(content, _00dc, level, "extracted_video.bin");
    dump_all_chunks_with_id(content, _01wb, level, "extracted_audio.bin");
}

fn parse_avi(mut content: &[u8], level: usize) {
    let list = chop_list(&mut content);
    assert!(list.r#type.to_u32() == hdrl);
    dump_list(&list, level);
    parse_hdrl(list.content, level + 1);

    'search_movi: loop {
        let entry = chop_entry(&mut content);
        dump_entry(&entry, level);
        match entry {
            Entry::List(list) => {
                if list.r#type.to_u32() == movi {
                    parse_movi(list.content, level + 1);
                    break 'search_movi;
                }
            }
            Entry::Chunk(_) => {}
        }
    }
}

fn dump_list<'a>(list: &List<'a>, level: usize) {
    println!("{}LIST listSize = {list_size}, listType = {list_type}",
             Padding(level*2),
             list_size = list.content.len(),
             list_type = list.r#type);
}

fn dump_chunk<'a>(chunk: &Chunk<'a>, level: usize) {
    println!("{}CHUNK ckSize = {chunk_size}, ckID = {chunk_id}",
             Padding(level*2),
             chunk_size = chunk.content.len(),
             chunk_id = chunk.id);
}

fn dump_entry<'a>(entry: &Entry<'a>, level: usize) {
    match entry {
        Entry::List(list) => dump_list(list, level),
        Entry::Chunk(chunk) => dump_chunk(chunk, level),
    }
}

fn dump_avi_as_tree(mut content: &[u8], level: usize) {
    while content.len() > 0 {
        match chop_entry(&mut content) {
            Entry::Chunk(chunk) => {
                dump_chunk(&chunk, level);
            }
            Entry::List(list) => {
                dump_list(&list, level);
                dump_avi_as_tree(list.content, level + 1);
            }
        }
    }
}

fn hack_avi_file(file_path: &str) -> Result<()> {
    let bytes = fs::read(file_path).map_err(|err| {
        eprintln!("ERROR: could not open file {file_path}: {err}");
    })?;
    let mut content = bytes.as_slice();

    println!("Read {file_path} {size} bytes", size = content.len());
    let riff = chop_u32(&mut content);
    assert!(riff == RIFF);
    let file_size = chop_u32(&mut content);
    assert!(file_size as usize == content.len());
    let file_type = chop_u32(&mut content);
    assert!(file_type == AVI_);

    // dump_avi_as_tree(content, 0);
    parse_avi(content, 0);

    Ok(())
}

fn hardcoded_avi_main_header(dwTotalFrames: DWORD) -> AVIMainHeader {
    AVIMainHeader {
        dwMicroSecPerFrame: 16666,
        dwMaxBytesPerSec: 86592000,
        dwPaddingGranularity: 0,
        dwFlags: 2320,
        dwTotalFrames,
        dwInitialFrames: 0,
        dwStreams: 2,
        dwSuggestedBufferSize: 1048576,
        dwWidth: 800,
        dwHeight: 600,
        dwReserved: [0, 0, 0, 0],
    }
}

fn hardcoded_avi_stream_header_vids(dwLength: DWORD) -> AVIStreamHeader {
    AVIStreamHeader {
        fccType: FOURCC::from_u32(vids),
        fccHandler: FOURCC::from_u32(0x0),
        dwFlags: 0,
        wPriority: 0,
        wLanguage: 0,
        dwInitialFrames: 0,
        dwScale: 1,
        dwRate: 60,
        dwStart: 0,
        dwLength,
        dwSuggestedBufferSize: 1440000,
        dwQuality: 4294967295,
        dwSampleSize: 0,
        rcFrame: RECT {
            left: 0,
            top: 0,
            right: 800,
            bottom: 600,
        },
    }
}

const HARDCODED_BITMAPINFOHEADER: BITMAPINFOHEADER = BITMAPINFOHEADER {
    biSize: 40,
    biWidth: 800,
    biHeight: -600,
    biPlanes: 1,
    biBitCount: 24,
    biCompression: 0,
    biSizeImage: 1440000,
    biXPelsPerMeter: 0,
    biYPelsPerMeter: 0,
    biClrUsed: 0,
    biClrImportant: 0,
};

fn hardcoded_avi_stream_header_auds(dwLength: DWORD) -> AVIStreamHeader {
    AVIStreamHeader {
        fccType: FOURCC::from_u32(auds),
        fccHandler: FOURCC::from_u32(0x1),
        dwFlags: 0,
        wPriority: 0,
        wLanguage: 0,
        dwInitialFrames: 0,
        dwScale: 1,
        dwRate: 48000,
        dwStart: 0,
        dwLength, // original 288000
        dwSuggestedBufferSize: 3200, // original 4096
        dwQuality: 4294967295,
        dwSampleSize: 4,
        rcFrame: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    }
}

const HARDCODED_WAVEFORMATEXTENSIBLE: WAVEFORMATEXTENSIBLE = WAVEFORMATEXTENSIBLE {
    Format: WAVEFORMATEX {
        wFormatTag: 65534,
        nChannels: 1,
        nSamplesPerSec: 48000,
        nAvgBytesPerSec: 192000,
        nBlockAlign: 4,
        wBitsPerSample: 32,
        cbSize: 22,
    },
    Samples: 32,
    dwChannelMask: 4,
    SubFormat: GUID {
        Data1: 3,
        Data2: 0,
        Data3: 16,
        Data4: [128, 0, 0, 170, 0, 56, 155, 113],
    },
};

fn write_list<'a>(sink: &mut impl io::Write, list: &List<'a>) -> io::Result<()> {
    sink.write(&LIST.to_le_bytes())?;
    let list_size = (list.content.len() + size_of::<FOURCC>()) as u32;
    sink.write(&list_size.to_le_bytes())?;
    sink.write(&list.r#type.to_u32().to_le_bytes())?;
    sink.write(list.content)?;
    Ok(())
}

fn write_chunk<'a>(sink: &mut impl io::Write, chunk: &Chunk<'a>) -> io::Result<()> {
    sink.write(&chunk.id.to_u32().to_le_bytes())?;
    let chunk_size = chunk.content.len() as u32;
    sink.write(&chunk_size.to_le_bytes())?;
    sink.write(chunk.content)?;

    if chunk_size%2 != 0 {
        sink.write(&[0])?;
    }

    Ok(())
}

fn fabrivate_video_strl(dwLength: DWORD) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    write_chunk(&mut result, &transmute_struct_to_chunk(FOURCC::from_u32(strh), &hardcoded_avi_stream_header_vids(dwLength)))?;
    write_chunk(&mut result, &transmute_struct_to_chunk(FOURCC::from_u32(strf), &HARDCODED_BITMAPINFOHEADER))?;
    Ok(result)
}

fn fabrivate_audio_strl(dwLength: DWORD) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    write_chunk(&mut result, &transmute_struct_to_chunk(FOURCC::from_u32(strh), &hardcoded_avi_stream_header_auds(dwLength)))?;
    write_chunk(&mut result, &transmute_struct_to_chunk(FOURCC::from_u32(strf), &HARDCODED_WAVEFORMATEXTENSIBLE))?;
    Ok(result)
}

fn fabricate_hdrl(frames_count: usize) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    write_chunk(
        &mut result,
        &transmute_struct_to_chunk(
            FOURCC::from_u32(avih),
            &hardcoded_avi_main_header(frames_count as DWORD)
        )
    )?;
    write_list(&mut result, &List {
        r#type: FOURCC::from_u32(strl),
        content: &fabrivate_video_strl(frames_count as DWORD)?,
    })?;
    write_list(&mut result, &List {
        r#type: FOURCC::from_u32(strl),
        content: &fabrivate_audio_strl((frames_count*800) as DWORD)?,
    })?;
    Ok(result)
}

fn fabricate_avi(movi_bytes: &[u8], frames_count: usize) -> io::Result<Vec<u8>> {
    let mut avi: Vec<u8> = Vec::new();
    avi.write(&AVI_.to_le_bytes())?;
    write_list(&mut avi, &List {
        r#type: FOURCC::from_u32(hdrl),
        content: &fabricate_hdrl(frames_count)?,
    })?;
    write_list(&mut avi, &List {
        r#type: FOURCC::from_u32(movi),
        content: movi_bytes,
    })?;
    Ok(avi)
}

fn fabricate_avi_file(file_path: &str, movi_bytes: &[u8], frames_count: usize) -> io::Result<()> {
    let mut riff: Vec<u8> = Vec::new();

    write_chunk(&mut riff, &Chunk {
        id: FOURCC::from_u32(RIFF),
        content: &fabricate_avi(movi_bytes, frames_count)?,
    })?;

    println!("Generating {file_path}...");
    fs::write(file_path, &riff)
}

#[derive(Default)]
struct FrameBGR24 {
    pixels: Vec<u8>,
}

impl FrameBGR24 {
    fn from_canvas(&mut self, canvas: &[u32]) {
        self.pixels.clear();
        for pixel in canvas {
            let r = ((pixel >> (8*2)) & 0xFF) as u8;
            let b = ((pixel >> (8*0)) & 0xFF) as u8;
            let g = ((pixel >> (8*1)) & 0xFF) as u8;
            self.pixels.push(b);
            self.pixels.push(g);
            self.pixels.push(r);
        }
    }
}

#[derive(Default)]
pub struct Container {
    frame_bgr24: FrameBGR24,
    movi: Vec<u8>,
    frame_count: usize,
    width: usize,
    height: usize,
    fps: usize,
}

impl Container {
    pub fn start(&mut self, width: usize, height: usize, fps: usize) {
        self.frame_count = 0;
        self.movi.clear();
        self.width = width;
        self.height = height;
        self.fps = fps;
    }

    pub fn frame(&mut self, canvas: &[u32], sound: &[f32]) -> io::Result<()> {
        self.frame_count += 1;
        self.frame_bgr24.from_canvas(canvas);
        write_chunk(&mut self.movi, &Chunk {
            id: FOURCC::from_str("00dc").unwrap(),
            content: &self.frame_bgr24.pixels,
        })?;
        write_chunk(&mut self.movi, &Chunk {
            id: FOURCC::from_str("01wb").unwrap(),
            content: unsafe {
                slice::from_raw_parts(sound.as_ptr() as *const u8, sound.len()*size_of::<f32>())
            }
        })
    }

    pub fn finish(&mut self, file_path: &str) -> io::Result<()> {
        fabricate_avi_file(file_path, &self.movi, self.frame_count)
    }
}

pub fn main() -> Result<()> {
    // fabricate_avi_file("./output.fab.avi", &[], 0).unwrap();
    hack_avi_file("./output.avi")?;

    Ok(())
}
