// OPENGL ETC STUFF
extern mod gl;
extern mod std;
use gl::types::*;
use systems::System;
use components::Components;

use std::{vec,ptr,str,cast};

pub struct RenderSystem {
    program: GLuint,
    fs: GLuint,
    vs: GLuint,
    vbo: GLuint,
    vao: GLuint,
    position_uniform: GLint,
    scale_uniform: GLint,
    color_uniform: GLint,
    window_uniform: GLint,
    texcoords_uniform: GLint,
    texenabled_uniform: GLint,
    char_atlas_tex: GLuint
}

impl System for RenderSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.sprite) {
            (Some(pos), Some(sprite)) => {
                // Set uniforms
                gl::ProgramUniform2f(self.program, self.position_uniform, pos.x as f32, pos.y as f32);
                gl::ProgramUniform2f(self.program, self.scale_uniform, sprite.x_size as f32, sprite.y_size as f32);
                //gl::ProgramUniform4fv would probably work for color
                gl::ProgramUniform4f(self.program, self.color_uniform, sprite.color[0] as f32, sprite.color[1] as f32, sprite.color[2] as f32, sprite.color[3] as f32);
                match sprite.texture {
                    Some(tex) => {
                        gl::BindTexture(gl::TEXTURE_2D, tex.texture);
                        let (tex_x, tex_y) = tex.texcoords;
                        let (tex_w, tex_h) = tex.texsize;
                        gl::ProgramUniform4f(self.program, self.texcoords_uniform, tex_x as f32, tex_y as f32, tex_w as f32, tex_h as f32);
                        gl::ProgramUniform1f(self.program, self.texenabled_uniform, 1.0 as f32);
                    },
                    None => {
                        gl::ProgramUniform1f(self.program, self.texenabled_uniform, 0.0 as f32);
                    }
                }
                // Draw a rect from the 4 vertices
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            },
            (_, _) => ()
        }
    }
}

// Vertex data
static VERTEX_DATA: [GLfloat, ..8] = [
    -0.5,  0.5,
    -0.5, -0.5,
     0.5,  0.5,
     0.5, -0.5
];

fn compile_shader(src: &[u8], ty: GLenum) -> GLuint {
    let shader = gl::CreateShader(ty);
    unsafe {
        // Attempt to compile the shader
        //transmute is used here because `as` causes ICE
        //wait a sec, is `src` null-terminated properly?
        gl::ShaderSource(shader, 1, std::cast::transmute(ptr::to_unsafe_ptr(&src.as_ptr())), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec::from_elem(len as uint - 1, 0u8);     // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::mut_null(), buf.as_mut_ptr() as *mut GLchar);
            fail!(str::raw::from_utf8(buf).to_owned());
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint, out_color: &str) -> GLuint {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    // gl::BindFragDataLocation needs to be called before linking to have effect
    unsafe {
        out_color.with_c_str(|ptr| gl::BindFragDataLocation(program, 0, ptr));
    }
    gl::LinkProgram(program);
    unsafe {
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec::from_elem(len as uint - 1, 0u8);     // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::mut_null(), buf.as_mut_ptr() as *mut GLchar);
            fail!(str::raw::from_utf8(buf).to_owned());
        }
    }
    program
}

impl RenderSystem {
    pub fn new() -> RenderSystem {
        // Create GLSL shaders
        let vs_src = std::io::fs::File::open_mode(&std::path::Path::new("main.vs.glsl"), std::io::Open, std::io::Read).unwrap().read_to_end();
        let vs = compile_shader(vs_src, gl::VERTEX_SHADER);
        let fs_src = std::io::fs::File::open_mode(&std::path::Path::new("main.fs.glsl"), std::io::Open, std::io::Read).unwrap().read_to_end();
        let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs, "out_color");

        let mut vao = 0;
        let mut vbo = 0;

        let position_uniform: GLint;
        let scale_uniform: GLint;
        let color_uniform: GLint;
        let window_uniform: GLint;
        let texcoords_uniform: GLint;
        let texenabled_uniform: GLint;

        unsafe {
            position_uniform = "position".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            scale_uniform = "scale".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            color_uniform = "color".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            window_uniform = "window".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            texcoords_uniform = "texcoords".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            texenabled_uniform = "texenabled".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            // Create Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (VERTEX_DATA.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                           cast::transmute(&VERTEX_DATA[0]),
                           gl::STATIC_DRAW);

            // Use shader program
            gl::UseProgram(program);

            // Specify the layout of the vertex data
            let vert_attr = "vertex".with_c_str(|ptr| gl::GetAttribLocation(program, ptr));
            gl::EnableVertexAttribArray(vert_attr as GLuint);
            gl::VertexAttribPointer(vert_attr as GLuint, 2, gl::FLOAT,
                                    gl::FALSE as GLboolean, 0, ptr::null());
        }
        //enable alpha blending
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        //load character atlas texture
        let char_atlas_src = std::io::fs::File::open_mode(&std::path::Path::new("dina_128x128.gray"), std::io::Open, std::io::Read).unwrap().read_to_end();
        let mut char_atlas_tex: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut char_atlas_tex);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_RECTANGLE, char_atlas_tex);
            gl::TexImage2D(gl::TEXTURE_RECTANGLE, 0, gl::RED as GLint, 128, 128, 0, gl::RED, gl::UNSIGNED_BYTE, cast::transmute(&char_atlas_src[0]));
            //TODO why doesn't this work?!
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        }

        RenderSystem {
            program: program,
            fs: fs,
            vs: vs,
            vbo: vbo,
            vao: vao,
            position_uniform: position_uniform,
            scale_uniform: scale_uniform,
            color_uniform: color_uniform,
            window_uniform: window_uniform,
            texcoords_uniform: texcoords_uniform,
            texenabled_uniform: texenabled_uniform,
            char_atlas_tex: char_atlas_tex
        }
    }
}

impl Drop for RenderSystem {
    fn drop(&mut self) {
        // Cleanup
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.fs);
        gl::DeleteShader(self.vs);
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(1, &self.char_atlas_tex);
        }
    }
}
