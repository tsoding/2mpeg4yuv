use std::ffi::{c_void, CString, CStr};
use std::ptr::{null, null_mut};
use std::os::raw::{c_char, c_int, c_float, c_uint, c_double};
use std::str;

type GLFWwindow = c_void;
type GLFWmonitor = c_void;
type GLFWerrorfun = extern fn(c_int, *const c_char);
type GLFWkeyfun = extern fn (window: *mut GLFWwindow, key: c_int, scancode: c_int, action: c_int, mods: c_int);

extern fn glfw_error_callback(code: c_int, description: *const c_char) {
    unsafe {
        panic!("GLFW ERROR {}: {}", code, CStr::from_ptr(description).to_str().unwrap());
    }
}

extern fn glfw_keyboard_callback(window: *mut GLFWwindow, key: c_int, _scancode: c_int, _action: c_int, _mods: c_int) {
    unsafe {
        if key == 81 {
            glfwSetWindowShouldClose(window, 1);
        }
    }
}

const GLFW_CONTEXT_VERSION_MAJOR: c_int = 0x00022002;
const GLFW_CONTEXT_VERSION_MINOR: c_int = 0x00022003;

#[link(name = "glfw")]
extern {
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

type GLclampf = c_float;
type GLbitfield = c_uint;
type GLuint = c_uint;
type GLint = c_int;
type GLenum = c_uint;
type GLsizei = c_int;
type GLchar = c_char;
type GLfloat = c_float;

#[link(name = "GL")]
extern {
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
}

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

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
#[allow(non_camel_case_types)]
enum pa_stream_direction {
    PA_STREAM_NODIRECTION,
    PA_STREAM_PLAYBACK,
    PA_STREAM_RECORD,
    PA_STREAM_UPLOAD
}

#[repr(C)]
#[allow(non_camel_case_types)]
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
extern {
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

pub fn main() {
    use std::f32::consts::PI;

    const SAMPLE_RATE: usize = 48000;
    const SOUND_FREQUENCY: f32 = 440.0;

    unsafe {
        use self::pa_stream_direction::*;
        use self::pa_sample_format::*;

        let mut args = std::env::args();
        let program = CString::new(args.next().expect("Program name")).unwrap();
        let stream_name = CString::new("playback").unwrap();

        let ss = pa_sample_spec {
            format: PA_SAMPLE_FLOAT32LE,
            rate: SAMPLE_RATE as u32,
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
        glfwSwapInterval(0);

        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        glBindVertexArray(vao);

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
                color = vec4(sin01(uv.x + time),
                             cos01(uv.y + time),
                             sin01(uv.x + uv.y + time),
                             1.0);
            }"#);
        println!("Created Fragment Shader {}", frag_shader);

        let program = link_shaders_into_program(&[vert_shader, frag_shader]);

        let time_uniform_name = CString::new("time").unwrap();
        let time_uniform_location =
            glGetUniformLocation(program, time_uniform_name.as_ptr());

        glUseProgram(program);

        let mut samples: [f32; 1024] = [0.0; 1024];
        let mut time: f32 = 0.0;
        let delta_time: f32 = 1.0 / SAMPLE_RATE as f32;

        while glfwWindowShouldClose(window) == 0 {
            glfwPollEvents();

            for (i, sample) in samples.iter_mut().enumerate() {
                *sample = (2.0*PI*SOUND_FREQUENCY*time).sin() * 0.10;
                time += delta_time;
            }

            pa_simple_write(s, samples.as_ptr() as *const c_void, 4 * samples.len() as u64, &mut error);

            glUniform1f(time_uniform_location, glfwGetTime() as f32);

            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
            glfwSwapBuffers(window);
        }

        glfwTerminate();
    }
}
