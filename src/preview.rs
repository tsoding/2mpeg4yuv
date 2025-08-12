#![allow(non_camel_case_types)]

// TODO: do not compile preview on non-linux platforms.
// Use some conditional compilation magic or what not.

// TODO: try to port this module to windows
// At least see what needs to be done to make this work on windows
// and create corresponding TODOs

use std::ffi::{c_void, CString, CStr};
use std::ptr::{null, null_mut};
use std::os::raw::{c_char, c_int, c_float, c_uint, c_double};
use std::str;
use super::config::*;

type GLFWwindow = c_void;
type GLFWmonitor = c_void;
type GLFWerrorfun = extern "C" fn(c_int, *const c_char);
type GLFWkeyfun = extern "C" fn (window: *mut GLFWwindow, key: c_int, scancode: c_int, action: c_int, mods: c_int);

extern "C" fn glfw_error_callback(code: c_int, description: *const c_char) {
    unsafe {
        // TODO: it is important that the program compiled with -C panic=abort
        //
        // Because if we are unwinding from within the C runtime, something bad may happen.
        // Is it possible to predicate on the kind of panic at compile time? Something like
        // "if panic is not abort fail the compilation in here"?
        panic!("GLFW ERROR {}: {}", code, CStr::from_ptr(description).to_str().unwrap());
    }
}

extern "C" fn glfw_keyboard_callback(window: *mut GLFWwindow, key: c_int, _scancode: c_int, _action: c_int, _mods: c_int) {
    unsafe {
        if key == 81 {
            glfwSetWindowShouldClose(window, 1);
        }
    }
}

const GLFW_CONTEXT_VERSION_MAJOR: c_int = 0x00022002;
const GLFW_CONTEXT_VERSION_MINOR: c_int = 0x00022003;

#[link(name = "glfw")]
extern "C" {
    fn glfwInit() -> c_int;
    fn glfwTerminate();

    fn glfwCreateWindow(width: c_int, height: c_int, title: *const c_char, monitor: *mut GLFWmonitor, share: *mut GLFWwindow) -> *mut GLFWwindow;
    fn glfwWindowShouldClose(window: *mut GLFWwindow) -> c_int;
    fn glfwPollEvents();
    fn glfwMakeContextCurrent(window: *mut GLFWwindow);
    fn glfwWindowHint(hint: c_int, value: c_int);
    fn glfwSetErrorCallback(cbfun: GLFWerrorfun) -> GLFWerrorfun;
    fn glfwSwapBuffers(window: *mut GLFWwindow);
    fn glfwSetKeyCallback(window: *mut GLFWwindow, cbfun: GLFWkeyfun) -> GLFWkeyfun;
    fn glfwSetWindowShouldClose(window: *mut GLFWwindow, value: c_int);
    fn glfwGetTime() -> c_double;
    fn glfwSwapInterval(interval: c_int);
}

const GL_COLOR_BUFFER_BIT: GLbitfield = 0x00004000;
const GL_FRAGMENT_SHADER: GLenum = 0x8B30;
const GL_VERTEX_SHADER: GLenum = 0x8B31;
const GL_COMPILE_STATUS: GLenum = 0x8B81;
const GL_LINK_STATUS: GLenum = 0x8B82;
const GL_TRIANGLE_STRIP: GLenum = 0x0005;
const GL_TEXTURE_2D: GLenum = 0x0DE1;
const GL_TEXTURE_MAG_FILTER: GLenum = 0x2800;
const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;
const GL_NEAREST: GLint = 0x2600;
const GL_RGBA: GLenum = 0x1908;
const GL_UNSIGNED_BYTE: GLenum = 0x1401;

type GLclampf = c_float;
type GLbitfield = c_uint;
type GLuint = c_uint;
type GLint = c_int;
type GLenum = c_uint;
type GLsizei = c_int;
type GLchar = c_char;
type GLfloat = c_float;
type GLvoid = c_void;

#[link(name = "GL")]
extern "C" {
    fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);
    fn glClear(mask: GLbitfield);
    fn glCreateShader(typ: GLenum) -> GLuint;
    fn glShaderSource(shader: GLuint, count: GLsizei, string: *const *const GLchar, length: *const GLint);
    fn glCompileShader (shader: GLuint);
    fn glGetShaderiv (shader: GLuint, pname: GLenum, params: *mut GLint);
    fn glGetShaderInfoLog (shader: GLuint, bufSize: GLsizei, length: *mut GLsizei, infoLog: *mut u8);
    fn glCreateProgram() -> GLuint;
    fn glAttachShader(program: GLuint, shader: GLuint);
    fn glLinkProgram(program: GLuint);
    fn glGetProgramiv (program: GLuint, pname: GLenum, params: *mut GLint);
    fn glGetProgramInfoLog (program: GLuint, bufSize: GLsizei, length: *mut GLsizei, infoLog: *mut u8);
    fn glUseProgram (program: GLuint);
    fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    fn glGenVertexArrays (n: GLsizei, arrays: *mut GLuint);
    fn glBindVertexArray (array: GLuint);
    fn glGetUniformLocation (program: GLuint, name: *const GLchar) -> GLint;
    fn glUniform1f (location: GLint, v0: GLfloat);
    fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    fn glBindTexture(target: GLenum, texture: GLuint);
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    fn glTexImage2D(target: GLenum,
                    level: GLint,
                    internalFormat: GLint,
                    width: GLsizei, height: GLsizei,
                    border: GLint, format: GLenum, typ: GLenum,
                    pixels: *const GLvoid);
    fn glTexSubImage2D(target: GLenum,
                        level: GLint,
                        xoffset: GLint,
                        yoffset: GLint,
                        width: GLsizei,
                        height: GLsizei,
                        format: GLenum,
                        typ: GLenum,
                        data: *const GLvoid);
}

fn shader_type_name(shader_type: GLenum) -> &'static str {
    match shader_type {
        GL_VERTEX_SHADER => "Vertex",
        GL_FRAGMENT_SHADER => "Fragment",
        _ => "(Unknown)",
    }
}

unsafe fn compile_shader_from_source(shader_type: GLenum, source: &str) -> GLuint {
    let shader = glCreateShader(shader_type);
    let source_cstr = CString::new(source).unwrap();
    glShaderSource(shader, 1, &source_cstr.as_ptr(), null_mut());
    glCompileShader(shader);
    let mut compiled = 0;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &mut compiled);
    if compiled == 0 {
        let mut info_log: [u8; 1024] = [0; 1024];
        let mut length: GLsizei = 0;
        glGetShaderInfoLog(shader, info_log.len() as i32, &mut length, info_log.as_mut_ptr());
        panic!("Could not compile {} Shader: {}", shader_type_name(shader_type), str::from_utf8(&info_log[0..length as usize]).unwrap());
    }
    shader
}

unsafe fn link_shaders_into_program(shaders: &[GLuint]) -> GLuint {
    let program = glCreateProgram();
    for shader in shaders {
        glAttachShader(program, *shader);
    }
    glLinkProgram(program);
    let mut linked = 0;
    glGetProgramiv (program, GL_LINK_STATUS, &mut linked);
    if linked == 0 {
        let mut info_log: [u8; 1024] = [0; 1024];
        let mut length: GLsizei = 0;
        glGetProgramInfoLog(program, info_log.len() as i32, &mut length, info_log.as_mut_ptr());
        panic!("Could not link shader program: {}", str::from_utf8(&info_log[0..length as usize]).unwrap());
    }
    program
}

#[repr(C)]
#[allow(dead_code)]
enum pa_stream_direction {
    PA_STREAM_NODIRECTION,
    PA_STREAM_PLAYBACK,
    PA_STREAM_RECORD,
    PA_STREAM_UPLOAD
}

#[repr(C)]
#[allow(dead_code)]
enum pa_sample_format {
    PA_SAMPLE_U8,
    PA_SAMPLE_ALAW,
    PA_SAMPLE_ULAW,
    PA_SAMPLE_S16LE,
    PA_SAMPLE_S16BE,
    PA_SAMPLE_FLOAT32LE,
    PA_SAMPLE_FLOAT32BE,
    PA_SAMPLE_S32LE,
    PA_SAMPLE_S32BE,
    PA_SAMPLE_S24LE,
    PA_SAMPLE_S24BE,
    PA_SAMPLE_S24_32LE,
    PA_SAMPLE_S24_32BE,
    PA_SAMPLE_MAX,
    PA_SAMPLE_INVALID = -1,
}

#[repr(C)]
struct pa_sample_spec {
    format: pa_sample_format,
    rate: u32,
    channels: u8,
}

type pa_simple = *mut c_void;
type pa_channel_map = *mut c_void;
type pa_buffer_attr = *mut c_void;

#[link(name = "pulse-simple")]
#[link(name = "pulse")]
extern "C" {
    fn pa_simple_new(server: *const c_char, name: *const c_char,
                     dir: pa_stream_direction,
                     dev: *const c_char,
                     stream_name: *const c_char,
                     ss: *const pa_sample_spec,
                     map: *const pa_channel_map,
                     attr: *const pa_buffer_attr,
                     error: *mut c_int) -> *mut pa_simple;
    fn pa_simple_write(s: *mut pa_simple,
                       data: *const c_void,
                       bytes: u64,
                       error: *mut c_int) -> c_int;
    fn pa_strerror(error: c_int) -> *const c_char;
}

const DELTA_TIME: f32 = 1.0 / FPS as f32;

pub fn main() {
    use super::sim::*;

    let mut state = State::new(WIDTH as f32, HEIGHT as f32);
    let mut canvas = vec![0; WIDTH * HEIGHT];
    let mut sound = vec![0.0; (DELTA_TIME * SOUND_SAMPLE_RATE as f32).floor() as usize];

    unsafe {
        use self::pa_stream_direction::*;
        use self::pa_sample_format::*;

        let mut args = std::env::args();
        let program = CString::new(args.next().expect("Program name")).unwrap();
        let stream_name = CString::new("playback").unwrap();

        let ss = pa_sample_spec {
            format: PA_SAMPLE_FLOAT32LE,
            rate: SOUND_SAMPLE_RATE as u32,
            channels: 1,
        };

        let mut error: c_int = 0;

        let s = pa_simple_new(null_mut(),
                              program.as_ptr(),
                              PA_STREAM_PLAYBACK,
                              null_mut(),
                              stream_name.as_ptr(),
                              &ss,
                              null(),
                              null(),
                              &mut error);
        if s.is_null() {
            panic!("pa_simple_new() failed: {}", CStr::from_ptr(pa_strerror(error)).to_str().unwrap());
        }

        println!("Created the playback stream: {:?}", s);

        glfwSetErrorCallback(glfw_error_callback);

        glfwInit();
        println!("Initialized GLFW");

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);

        let title = CString::new("2mpeg4yuv").unwrap();
        let window = glfwCreateWindow(WIDTH as c_int, HEIGHT as c_int,
                                      title.as_ptr(),
                                      null_mut(),
                                      null_mut());
        println!("Create window {:?}", window);

        glfwSetKeyCallback(window, glfw_keyboard_callback);

        glfwMakeContextCurrent(window);
        glfwSwapInterval(1);

        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        glBindVertexArray(vao);

        let mut frame_texture: GLuint = 0;
        glGenTextures(1, &mut frame_texture);
        glBindTexture(GL_TEXTURE_2D, frame_texture);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
        glTexImage2D(GL_TEXTURE_2D,
                     0,
                     GL_RGBA as i32,
                     WIDTH as GLint,
                     HEIGHT as GLint,
                     0,
                     GL_RGBA,
                     GL_UNSIGNED_BYTE,
                     canvas.as_ptr() as *const GLvoid);

        let vert_shader = compile_shader_from_source(
            GL_VERTEX_SHADER,
            r#"#version 330 core
            out vec2 uv;
            void main()
            {
                uv = vec2(gl_VertexID & 1, gl_VertexID >> 1) ;
                gl_Position = vec4(2.0 * uv - 1.0, 0.0, 1.0);
            }"#);
        println!("Created Vertex Shader {}", vert_shader);

        let frag_shader = compile_shader_from_source(
            GL_FRAGMENT_SHADER,
            r#"#version 330 core
            uniform float time;
            uniform sampler2D frame;
            in vec2 uv;
            out vec4 color;

            float sin01(float a)
            {
                return (sin(a) + 1.0) / 2.0;
            }

            float cos01(float a)
            {
                return (cos(a) + 1.0) / 2.0;
            }

            void main()
            {
                // 0xAARRGGBB
                // 0xAABBGGRR
                vec4 pixel = texture(frame, vec2(uv.x, 1.0 - uv.y));
                color = vec4(pixel.zyx, 1.0);
            }"#);
        println!("Created Fragment Shader {}", frag_shader);

        let program = link_shaders_into_program(&[vert_shader, frag_shader]);

        let time_uniform_name = CString::new("time").unwrap();
        let time_uniform_location =
            glGetUniformLocation(program, time_uniform_name.as_ptr());

        glUseProgram(program);

        while glfwWindowShouldClose(window) == 0 {
            glfwPollEvents();

            canvas.fill(BACKGROUND);
            state.render(&mut canvas, WIDTH);
            glTexSubImage2D(GL_TEXTURE_2D,
                             0,
                             0,
                             0,
                             WIDTH as GLsizei,
                             HEIGHT as GLsizei,
                             GL_RGBA,
                             GL_UNSIGNED_BYTE,
                             canvas.as_ptr() as *const GLvoid);

            sound.fill(0.0);
            state.sound(&mut sound, SOUND_SAMPLE_RATE);
            pa_simple_write(s, sound.as_ptr() as *const c_void, 4 * sound.len() as u64, &mut error);

            state.update(1.0 / 60.0);

            glUniform1f(time_uniform_location, glfwGetTime() as f32);
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
            glfwSwapBuffers(window);
        }

        glfwTerminate();
    }
}
