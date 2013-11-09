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
#[feature(managed_boxes)]; //TODO do without managed boxes

extern mod glfw;
extern mod gl;

use std::libc;

use std::cast;
use std::ptr;
use std::str;
use std::vec;

use std::rt::io::Reader;

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

struct SpriteTexture {
    texture: GLuint,
    texcoords: (uint, uint),
    texsize: (uint, uint)
}

// takes single ascii character as a byte as input
fn texture_from_byte(b: u8) -> SpriteTexture {
    // 32 == ' ', the first character in atlas
    let cb: uint = (b as uint) - 32;
    let x = (cb % 16) * 7;
    let y = (cb / 16) * 14;
    SpriteTexture {
        texture: 0,
        texcoords: (x, y),
        texsize: (7, 14)
    }
}

fn texture_from_char(c: char) -> SpriteTexture {
    texture_from_byte(c.to_ascii().to_byte())
}

fn texture_from_uint(i: uint) -> SpriteTexture {
    texture_from_byte(((i+0x30) as u8))
}

struct Sprite {
    x_size: f64,
    y_size: f64,
    //instead of color+texture we should have something like material
    //which could be eg enum
    color: [f64, ..4],
    texture: Option<SpriteTexture>
}

struct Score {
    score: uint
}

struct Components {
    position: Option<@mut Position>,
    horiz_velocity: Option<@mut HorizVelocity>,
    vert_velocity: Option<@mut VertVelocity>,
    sprite: Option<@mut Sprite>,
    score: Option<@mut Score>,
}


//GLOBAL SYSTEM DEFINITIONS
trait GlobalSystem {
    fn process(&self, window: &glfw::Window) -> ();
}

struct BotInputSystem {
    paddle: @Components,
    ball: @Components
}

impl GlobalSystem for BotInputSystem {
    fn process(&self, _: &glfw::Window) -> () {
        let d = self.ball.position.unwrap().y - self.paddle.position.unwrap().y;
        if std::num::abs(d) > 0.2 {
            if d > 0.0 {
                self.paddle.vert_velocity.unwrap().y = 1.5/60.0;
            } else {
                self.paddle.vert_velocity.unwrap().y = -1.5/60.0;
            }
        } else {
            self.paddle.vert_velocity.unwrap().y = 0.0;
        }
    }
}

struct KeyboardInputSystem {
    paddle: @Components
}

impl GlobalSystem for KeyboardInputSystem {
    fn process(&self, window: &glfw::Window) -> () {
        let mut dir = 0.0;
        if window.get_key(glfw::KeyA) == glfw::Press {
            dir += 1.0;
        }
        if window.get_key(glfw::KeyZ) == glfw::Press {
            dir -= 1.0;
        }
        self.paddle.vert_velocity.unwrap().y = 1.5*dir/60.0;
    }
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
        match (entity.position, entity.vert_velocity, entity.sprite) {
            (Some(pos), Some(vel), Some(spr)) => {
                if (pos.y + (spr.y_size/2.0)) >= 3.0 {
                    vel.y *= -1.0;
                    pos.y = 3.0 - (spr.y_size/2.0);
                }
                if (pos.y - (spr.y_size/2.0)) <= 0.0 {
                    vel.y *= -1.0;
                    pos.y = spr.y_size/2.0;
                }
            },
            (_, _, _) => () 
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
        match (entity.position, entity.horiz_velocity, entity.vert_velocity, entity.sprite) {
            (Some(pos), Some(hvel), Some(vvel), Some(spr)) => {
                let mut paddles: Option<(@Components, @Components)> = None;
                if (pos.x+(spr.x_size)/2.0) >= (self.right_paddle.position.unwrap().x-(self.right_paddle.sprite.unwrap().x_size)/2.0) {
                    paddles = Some((self.right_paddle, self.left_paddle));
                } 
                else if (pos.x-(spr.x_size)/2.0) <= (self.left_paddle.position.unwrap().x+(self.left_paddle.sprite.unwrap().x_size)/2.0) {
                    paddles = Some((self.left_paddle, self.right_paddle));
                }
                match paddles {
                    Some((paddle_a, paddle_b)) => {
                        let paddle_distance = pos.y - paddle_a.position.unwrap().y;
                        let paddle_height = paddle_a.sprite.unwrap().y_size/2.0;
                        if std::num::abs(paddle_distance) < paddle_height {
                            hvel.x *= -1.0;
                            vvel.y = 0.5*hvel.x*std::num::sinh(3.14*paddle_distance/paddle_height);
                        }
                        else {
                            // SCORE FOR OTHER PLAYER
                            paddle_b.score.unwrap().score += 1;
                            pos.x = 2.0;
                            pos.y = 1.5;
                            hvel.x *= -1.0;
                            vvel.y = 0.0;
                        }
                    },
                    None => {}
                }
            },
            (_, _, _, _) => ()
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


// WORLD DEFINITION
// We need to figure out how to integrate World with the main game loop
// in Artemis world has a `setDelta` method for timestep
struct World {
    entities: ~[@Components],
    systems: ~[@System],
    global_systems: ~[@GlobalSystem]
}

impl World {
    fn new() -> World {
        return World {entities: ~[], systems: ~[], global_systems: ~[]};
    }

    fn process(&self, window: &glfw::Window) {
        for system in self.global_systems.iter() {
            system.process(window);
        }
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
            x_size: 0.10,
            y_size: 0.20,
            color: [0.8, 0.7, 0.3, 0.0],
            texture: Some(texture_from_char('@'))
        }),
        score: None
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
            color: [xpos/4.0, 1.0-(xpos/4.0), 0.3, 1.0],
            texture: None
        }),
        score: Some(@mut Score { score: 0 } )
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
            color: [0.0, 0.0, 0.0, 0.3],
            texture: None
        }),
        score: None
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
            color: [0.45, 0.4, 1.0, 1.0],
            texture: None
        }),
        score: None
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

fn compile_shader(src: &[u8], ty: GLenum) -> GLuint {
    let shader = gl::CreateShader(ty);
    unsafe {
        // Attempt to compile the shader
        //transmute is used here because `as` causes ICE
        //wait a sec, is `src` null-terminated properly?
        gl::ShaderSource(shader, 1, std::cast::transmute(std::ptr::to_unsafe_ptr(&std::vec::raw::to_ptr(src))), ptr::null());
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
        let vs_src = std::rt::io::fs::File::open_mode(&std::path::Path::new("main.vs.glsl"), std::rt::io::Open, std::rt::io::Read).unwrap().read_to_end();
        let vs = compile_shader(vs_src, gl::VERTEX_SHADER);
        let fs_src = std::rt::io::fs::File::open_mode(&std::path::Path::new("main.fs.glsl"), std::rt::io::Open, std::rt::io::Read).unwrap().read_to_end();
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
        let char_atlas_src = std::rt::io::fs::File::open_mode(&std::path::Path::new("dina_128x128.gray"), std::rt::io::Open, std::rt::io::Read).unwrap().read_to_end();
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

        let mut window_width = 800;
        let mut window_height = 480;
        let window = glfw::Window::create(window_width, window_height, "Pong", glfw::Windowed).expect("Failed to create GLFW window.");;
        window.set_key_callback(
            |window: &glfw::Window, key: glfw::Key, _: libc::c_int, action: glfw::Action, _: glfw::Modifiers| {
                if action == glfw::Press {
                    match key {
                        glfw::KeyEscape => {
                            window.set_should_close(true);
                        },
                        _ => {}
                    }
                }
            }
        );
        window.make_context_current();

        // Load the OpenGL function pointers
        gl::load_with(glfw::get_proc_address);


        let rs = @RenderSystem::new();
        let (fb_size_port, fb_size_chan): (Port<(uint,uint)>, Chan<(uint,uint)>) = std::comm::stream();
        window.set_framebuffer_size_callback(
            |_: &glfw::Window, width: int, height: int| {
                fb_size_chan.send((width as uint,height as uint));
            }
        );

        world.systems.push(rs as @System);

        let kbs = @KeyboardInputSystem { paddle: left_paddle };
        world.global_systems.push(kbs as @GlobalSystem);

        let bis = @BotInputSystem { paddle: right_paddle, ball: ball };
        world.global_systems.push(bis as @GlobalSystem);

        let mut prev_scores = (0,0);
        while !window.should_close() {
            // Poll events
            glfw::poll_events();

            while fb_size_port.peek() {
                let (w,h) = fb_size_port.recv();
                window_width = w;
                window_height = h;
            }

            gl::Viewport(0,0, window_width as GLint, window_height as GLint);
            gl::ProgramUniform2f(rs.program, rs.window_uniform, window_width as f32, window_height as f32);
            // Clear the screen 
            gl::ClearColor(0.8, 0.8, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // process game world
            world.process(&window);
            let new_scores = (left_paddle.score.unwrap().score, right_paddle.score.unwrap().score);
            if new_scores != prev_scores {
                println!("{:?}", new_scores);
                prev_scores = new_scores;
            }

            // Swap buffers
            window.swap_buffers();
        }
    }
}
