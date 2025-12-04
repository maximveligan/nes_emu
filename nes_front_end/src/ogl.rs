use gl;

use crate::NES_SCREEN_HEIGHT;
use crate::NES_SCREEN_WIDTH;

const VS: &str = include_str!("shaders/vs.glsl");
const FS: &str = include_str!("shaders/fs.glsl");

pub fn init_shaders_and_texture(window: &mut glfw::Window) -> u32 {
    let mut texture = 0;
    let data: [u8; (NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 3) as usize] =
        [0xFF; (NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 3) as usize];

    unsafe {
        gl::load_with(|s| window.get_proc_address(s).unwrap() as *const _);
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Viewport(0, 0, NES_SCREEN_WIDTH as i32, NES_SCREEN_HEIGHT as i32);
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);

        gl::ShaderSource(
            vertex_shader,
            1,
            &(VS.as_bytes().as_ptr().cast()),
            &(VS.len().try_into().unwrap()),
        );
        gl::CompileShader(vertex_shader);

        let mut success = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(vertex_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        gl::ShaderSource(
            fragment_shader,
            1,
            &(FS.as_bytes().as_ptr().cast()),
            &(FS.len().try_into().unwrap()),
        );
        gl::CompileShader(fragment_shader);

        let mut success = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        gl::UseProgram(shader_program);

        gl::GenTextures(1, &mut texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            NES_SCREEN_WIDTH as i32,
            NES_SCREEN_HEIGHT as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            data.as_ptr().cast(),
        );

        gl::Uniform1i(
            gl::GetUniformLocation(shader_program, "tex".as_ptr().cast()),
            0,
        );
    }

    texture
}

pub fn update_texture(texture_id: u32, framebuffer: &[u8]) {
    unsafe {
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            NES_SCREEN_WIDTH as i32,
            NES_SCREEN_HEIGHT as i32,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            framebuffer.as_ptr().cast(),
        );
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
    }
}
