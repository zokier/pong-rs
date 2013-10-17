// Copyright 2013 Torste Aikio and gl-rs developers. 

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[feature(globs)];
#[feature(macro_rules)];

extern mod glfw;
extern mod gl;

use std::libc;

use std::cast;
use std::ptr;
use std::str;
use std::sys;
use std::vec;

use std::io;

use gl::types::*;

// COMPONENT DEFINITIONS
struct Position {
    x: f64,
    y: f64
}

struct HorizVelocity {
    x: f64
}

struct VertVelocity {
    y: f64
}

struct Sprite {
    x_size: f64,
    y_size: f64,
    color: [f64, ..4]
}

struct Components {
    position: Option<@mut Position>,
    horiz_velocity: Option<@mut HorizVelocity>,
    vert_velocity: Option<@mut VertVelocity>,
    sprite: Option<@mut Sprite>
}


// SYSTEM DEFINITIONS
trait System {
    fn process(&self, entity: @Components) -> ();
}

struct MovementSystem;

impl System for MovementSystem {
    fn process(&self, entity: @Components) -> () {
        match entity.position { 
            Some(pos) => {
                match entity.vert_velocity {
                    Some(v) => pos.y += v.y,
                    None => ()
                } 
                match entity.horiz_velocity {
                    Some(v) => pos.x += v.x,
                    None => ()
                }
            },
            None => ()
        }
    }
}

struct EdgeCollisionSystem;

impl System for EdgeCollisionSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.vert_velocity) {
            (Some(pos), Some(vel)) => {
                if pos.y >= 4.0 || pos.y <= 0.0 {
                    vel.y *= -1.0;
                }
            },
            (_, _) => () 
        }
    }
}

struct PaddleCollisionSystem {
    left_paddle: @Components,
    right_paddle: @Components,
}

impl System for PaddleCollisionSystem {
    fn process(&self, entity: @Components) -> () {
        //TODO use a hitbox or something instead of sprite
        match (entity.position, entity.horiz_velocity, entity.sprite) {
            (Some(pos), Some(vel), Some(spr)) => {
                if (pos.x+(spr.x_size)/2.0) >= (self.right_paddle.position.unwrap().x-(self.right_paddle.sprite.unwrap().x_size)/2.0) {
                    if std::num::abs(pos.y - self.right_paddle.position.unwrap().y) < (self.right_paddle.sprite.unwrap().y_size/2.0) {
                        vel.x *= -1.0;
                    }
                }
                if (pos.x-(spr.x_size)/2.0) <= (self.left_paddle.position.unwrap().x+(self.left_paddle.sprite.unwrap().x_size)/2.0) {
                    if std::num::abs(pos.y - self.left_paddle.position.unwrap().y) < (self.left_paddle.sprite.unwrap().y_size/2.0) {
                        vel.x *= -1.0;
                    }
                }
            },
            (_, _, _) => ()
        }
    }
}

struct RenderSystem {
    program: GLuint,
    fs: GLuint,
    vs: GLuint,
    vbo: GLuint,
    vao: GLuint,
    position_uniform: GLint,
    scale_uniform: GLint,
    color_uniform: GLint,
    window_uniform: GLint
}

impl System for RenderSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.sprite) {
            (Some(pos), Some(sprite)) => {
                // Set uniforms
                gl::ProgramUniform2f(self.program, self.position_uniform, pos.x as f32, pos.y as f32);
                gl::ProgramUniform2f(self.program, self.scale_uniform, sprite.x_size as f32, sprite.y_size as f32);
                gl::ProgramUniform4f(self.program, self.color_uniform, sprite.color[0] as f32, sprite.color[1] as f32, sprite.color[2] as f32, sprite.color[3] as f32);
                // Draw a rect from the 4 vertices
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            },
            (_, _) => ()
        }
    }
}


// WORLD DEFINITION
// We need to figure out how to integrate World with the main game loop
// in Artemis world has a `setDelta` method for timestep
struct World {
    entities: ~[@Components],
    systems: ~[@System]
}

impl World {
    fn new() -> World {
        return World {entities: ~[], systems: ~[]};
    }

    fn process(&self) {
        for system in self.systems.iter() {
            for entity in self.entities.iter() {
                system.process(*entity);
            }
        }
    }
}



//ENTITY CONSTRUCTORS
enum PaddleSide {
    RIGHT,
    LEFT
}

fn new_ball() -> @Components {
    @Components {
        position: Some(@mut Position { x: 2.0, y: 1.5 }),
        horiz_velocity: Some(@mut HorizVelocity { x: 1.0/60.0 }),
        vert_velocity: Some(@mut VertVelocity { y: 0.0 }),
        sprite: Some(@mut Sprite {
            x_size: 0.05,
            y_size: 0.05,
            color: [0.8, 0.7, 0.3, 1.0]
        })
    }
}

fn new_paddle(side: PaddleSide) -> @Components {
    let xpos = match side {
        RIGHT => 3.9,
        LEFT => 0.1
    };
    @Components {
        position: Some(@mut Position { x: xpos, y: 1.5 }),
        horiz_velocity: None,
        vert_velocity: Some(@mut VertVelocity { y: 0.0 }),
        sprite: Some(@mut Sprite {
            x_size: 0.1,
            y_size: 0.4,
            color: [xpos/4.0, 1.0-(xpos/4.0), 0.3, 1.0]
        })
    }
}

fn new_background_2() -> @Components {
    @Components {
        position: Some(@mut Position { x: 2.0, y: 1.5 }),
        horiz_velocity: None,
        vert_velocity: None,
        sprite: Some(@mut Sprite {
            x_size: 3.0,
            y_size: 2.0,
            color: [0.0, 0.0, 0.0, 0.3]
        })
    }
}


fn new_background() -> @Components {
    @Components {
        position: Some(@mut Position { x: 2.0, y: 1.5 }),
        horiz_velocity: None,
        vert_velocity: None,
        sprite: Some(@mut Sprite {
            x_size: 4.0,
            y_size: 3.0,
            color: [0.45, 0.4, 1.0, 1.0]
        })
    }
}


// OPENGL ETC STUFF

// Vertex data
static VERTEX_DATA: [GLfloat, ..8] = [
    -0.5,  0.5,
    -0.5, -0.5,
     0.5,  0.5,
     0.5, -0.5
];

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader = gl::CreateShader(ty);
    unsafe {
        // Attempt to compile the shader
        src.with_c_str(|ptr| gl::ShaderSource(shader, 1, &ptr, ptr::null()));
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec::from_elem(len as uint - 1, 0u8);     // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::mut_null(), vec::raw::to_mut_ptr(buf) as *mut GLchar);
            fail!(str::raw::from_utf8(buf));
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
            gl::GetProgramInfoLog(program, len, ptr::mut_null(), vec::raw::to_mut_ptr(buf) as *mut GLchar);
            fail!(str::raw::from_utf8(buf));
        }
    }
    program
}

impl RenderSystem {
    fn new() -> RenderSystem {
        // Create GLSL shaders
        let vs_src = io::read_whole_file_str(&std::path::Path::new("main.vs.glsl")).unwrap();
        let vs = compile_shader(vs_src, gl::VERTEX_SHADER);
        let fs_src = io::read_whole_file_str(&std::path::Path::new("main.fs.glsl")).unwrap();
        let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs, "out_color");

        let mut vao = 0;
        let mut vbo = 0;
        
        let position_uniform: GLint;
        let scale_uniform: GLint;
        let color_uniform: GLint;
        let window_uniform: GLint;

        unsafe {
            position_uniform = "position".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            scale_uniform = "scale".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            color_uniform = "color".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            window_uniform = "window".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            // Create Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (VERTEX_DATA.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
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
        RenderSystem {
            program: program,
            fs: fs,
            vs: vs,
            vbo: vbo,
            vao: vao,
            position_uniform: position_uniform,
            scale_uniform: scale_uniform,
            color_uniform: color_uniform,
            window_uniform: window_uniform
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
        }
    }
}

fn main() {
    do glfw::set_error_callback |_, description| {
        println!("GLFW Error: {}", description);
    }

    do glfw::start {
        // initialize game world
        let left_paddle: @Components = new_paddle(LEFT);
        let right_paddle: @Components = new_paddle(RIGHT);
        let ball: @Components = new_ball();
        let background: @Components = new_background();
        let background_2: @Components = new_background_2();
        let ms = @MovementSystem;
        let es = @EdgeCollisionSystem;
        let ps = @PaddleCollisionSystem{ right_paddle: right_paddle, left_paddle: left_paddle };

        let mut world: World = World::new();
        world.entities.push(background);
        world.entities.push(background_2);
        world.entities.push(left_paddle);
        world.entities.push(right_paddle);
        world.entities.push(ball);
        world.systems.push(ms as @System);
        world.systems.push(es as @System);
        world.systems.push(ps as @System);

        // Choose a GL profile that is compatible with OS X 10.7+
        glfw::window_hint::context_version(3, 2);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window_width = 800;
        let window_height = 480;
        let window = glfw::Window::create(window_width, window_height, "Pong", glfw::Windowed).unwrap();
        window.set_key_callback(key_callback);
        window.make_context_current();

        // Load the OpenGL function pointers
        gl::load_with(glfw::get_proc_address);


        let rs = @RenderSystem::new();
        gl::ProgramUniform2f(rs.program, rs.window_uniform, window_width as f32, window_height as f32);
        /* // TODO figure out how to make this work
        window.set_framebuffer_size_callback(|_: &glfw::Window, width: int, height: int| {
            gl::ProgramUniform2f(rs.program, rs.window_uniform, width as f32, height as f32);
        });
        */

        world.systems.push(rs as @System);

        while !window.should_close() {
            // Poll events
            glfw::poll_events();

            // Clear the screen 
            gl::ClearColor(0.8, 0.8, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // process game world
            world.process();

            // Swap buffers
            window.swap_buffers();
        }
    }
}

fn key_callback(window: &glfw::Window, key: glfw::Key, _: libc::c_int, action: glfw::Action, _: glfw::Modifiers) {
    if action == glfw::Press && key == glfw::KeyEscape {
        window.set_should_close(true);
    }
}

