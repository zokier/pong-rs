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

use gl::types::*;

trait Component {
    // There should be some central allocation mechanism for component_ids
    // maybe even an enum
    // actually can we just get rid of the whole component id stuff?
    // could we merge the two functions?
    fn component_id(_: Option<Self>) -> uint;
    fn struct_component_id(&self) -> uint;
}

//Do these need to be right here?
struct Position {
    x: f64,
    y: f64
}

impl Component for Position {
    fn component_id(_: Option<Position>) -> uint { 0 }
    fn struct_component_id(&self) -> uint { 0 }
}

struct HorizVelocity {
    x: f64
}

impl Component for HorizVelocity {
    fn component_id(_: Option<HorizVelocity>) -> uint { 1 }
    fn struct_component_id(&self) -> uint { 1 }
}

struct VertVelocity {
    y: f64
}

impl Component for VertVelocity {
    fn component_id(_: Option<VertVelocity>) -> uint { 2 }
    fn struct_component_id(&self) -> uint { 2 }
}

struct Sprite {
    x_size: f64,
    y_size: f64
}

impl Component for Sprite{
    fn component_id(_: Option<Sprite>) -> uint { 3 }
    fn struct_component_id(&self) -> uint { 3 }
}

static COMPONENT_COUNT: uint = 4;
struct Components {
    position: Option<@mut Position>,
    horiz_velocity: Option<@mut HorizVelocity>,
    vert_velocity: Option<@mut VertVelocity>,
    sprite: Option<@mut Sprite>
}

impl Components {
    fn contains(&self, id: uint) -> bool
    {
        // we should use sizeof(Components)/sizeof(Option<@uint>) here
        // presuming sizeof(Option<@uint>) == sizeof(Option<@Foo>)
        // waiting for rust to get constexpr sizeof
        unsafe {
            let v: &[Option<@uint>, ..COMPONENT_COUNT] = std::cast::transmute::<&Components, &[Option<@uint>, ..COMPONENT_COUNT]>(self);
            return (id < COMPONENT_COUNT) && match v[id] {
                Some(_) => true,
                None => false
            }
        }
    }
}

// Do we need more advanced rules?
// if not, maybe a boolean would suffice?
enum Operator {
    HAS,
    HAS_NOT
}

struct Rule {
    component_id: uint, //TODO a unique type for this?
    operator: Operator
}

trait System {
    fn process(&self, entity: @Components) -> ();
    fn rules(&self) -> @[Rule];
}

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
                // there is significant amount of nested iteration going on here
                // could this be optimized somehow
                let mut rule_matches = false;
                for rule in system.rules().iter() {
                    match (entity.contains(rule.component_id), rule.operator) {
                        (true, HAS_NOT) => { rule_matches = false; break; },
                        (false, HAS) => { rule_matches = false; break; },
                        (false, HAS_NOT) => (),
                        (true, HAS) => rule_matches = true
                    }
                }
                if rule_matches {
                    system.process(*entity);
                }
            }
        }
    }
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
            None => println!("rulecheck fail")
        }
    }

    fn rules(&self) -> @[Rule] {
        // TODO how to express HorizVelocity OR VertVelocity
        let r: @[Rule] = @[Rule{component_id: Component::component_id(None::<Position>), operator: HAS}];
        return r;
    }
}

struct EdgeCollisionSystem;

impl System for EdgeCollisionSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.vert_velocity) {
            (Some(pos), Some(vel)) => {
                if pos.y >= 1.0 || pos.y <= 0.0 {
                    vel.y *= -1.0;
                }
            },
            (_, _) => () //rulechecks should prevent this from ever happening!
        }
    }

    fn rules(&self) -> @[Rule] {
        let r: @[Rule] = @[Rule{component_id: Component::component_id(None::<Position>), operator: HAS}, Rule{component_id: Component::component_id(None::<VertVelocity>), operator: HAS}];
        return r;
    }
}

struct PaddleCollisionSystem {
    left_paddle: @Components,
    right_paddle: @Components,
    paddle_width: f64
}

impl System for PaddleCollisionSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.horiz_velocity) {
            (Some(pos), Some(vel)) => {
                if pos.x >= 1.0 {
                    if std::num::abs(pos.y - self.right_paddle.position.unwrap().y) < (self.paddle_width/2.0) {
                        vel.x *= -1.0;
                    }
                }
                if pos.x <= 0.0 {
                    if std::num::abs(pos.y - self.left_paddle.position.unwrap().y) < (self.paddle_width/2.0) {
                        vel.x *= -1.0;
                    }
                }
            },
            (_, _) => () //rulechecks should prevent this from ever happening!
        }
    }

    fn rules(&self) -> @[Rule] {
        let r: @[Rule] = @[Rule{component_id: Component::component_id(None::<Position>), operator: HAS}, Rule{component_id: Component::component_id(None::<HorizVelocity>), operator: HAS}];
        return r;
    }
}

struct RenderSystem {
    program: GLuint,
    position_uniform: GLint,
    scale_uniform: GLint
}

impl System for RenderSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.sprite) {
            (Some(pos), Some(sprite)) => {
                // Set uniforms
                gl::ProgramUniform2f(self.program, self.position_uniform, pos.x as f32, pos.y as f32);
                gl::ProgramUniform2f(self.program, self.scale_uniform, sprite.x_size as f32, sprite.y_size as f32);
                // Draw a rect from the 4 vertices
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            },
            (_, _) => ()
        }
    }

    fn rules(&self) -> @[Rule] {
        let r: @[Rule] = @[Rule{component_id: Component::component_id(None::<Position>), operator: HAS}, Rule{component_id: Component::component_id(None::<Sprite>), operator: HAS}];
        return r;
    }
}

enum PaddleSide {
    RIGHT,
    LEFT
}

fn new_ball() -> @Components {
    let components = @Components { position: Some(@mut Position { x: 0.5, y: 0.5 }), horiz_velocity: Some(@mut HorizVelocity { x: 0.2/60.0 }), vert_velocity: Some(@mut VertVelocity { y: 0.0 }), sprite: Some(@mut Sprite {x_size: 0.025, y_size: 0.025})};
    return components;
}

fn new_paddle(side: PaddleSide) -> @Components {
    let xpos = match side {
        RIGHT => 0.8,
        LEFT => 0.2
    };
    let components = @Components { position: Some(@mut Position { x: xpos, y: 0.5 }), horiz_velocity: None, vert_velocity: Some(@mut VertVelocity { y: 0.0 }), sprite: Some(@mut Sprite {x_size: 0.05, y_size: 0.2})};
    return components;
}

// Vertex data
static VERTEX_DATA: [GLfloat, ..8] = [
    -1.0,  1.0,
    -1.0, -1.0,
     1.0,  1.0,
     1.0, -1.0
];

// Shader sources
static VS_SRC: &'static str =
   "#version 150\n\
    in vec2 vertex;\n\
    uniform vec2 scale;\n\
    uniform vec2 position;\n\
    void main() {\n\
       vec2 out_vert = vertex * scale;\n\
       out_vert += position;\n\
       gl_Position = vec4(out_vert, 0.0, 1.0);\n\
    }";

static FS_SRC: &'static str =
   "#version 150\n\
    out vec4 out_color;\n\
    void main() {\n\
       out_color = vec4(1.0, 1.0, 1.0, 1.0);\n\
    }";

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

fn main() {
    do glfw::set_error_callback |_, description| {
        println!("GLFW Error: {}", description);
    }

    do glfw::start {
        // initialize game world
        let left_paddle: @Components = new_paddle(LEFT);
        let right_paddle: @Components = new_paddle(RIGHT);
        let ball: @Components = new_ball();
        let ms: @System = @MovementSystem as @System;
        let es: @System = @EdgeCollisionSystem as @System;
        let ps: @System = @PaddleCollisionSystem{ right_paddle: right_paddle, left_paddle: left_paddle, paddle_width: 0.2} as @System;

        let mut world: World = World::new();
        world.entities.push(left_paddle);
        world.entities.push(right_paddle);
        world.entities.push(ball);
        world.systems.push(ms);
        world.systems.push(es);
        world.systems.push(ps);

        // Choose a GL profile that is compatible with OS X 10.7+
        glfw::window_hint::context_version(3, 2);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(1024, 600, "Pong", glfw::Windowed).unwrap();
        window.set_key_callback(key_callback);
        window.make_context_current();

        // Load the OpenGL function pointers
        gl::load_with(glfw::get_proc_address);

        // Create GLSL shaders
        let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
        let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs, "out_color");

        let mut vao = 0;
        let mut vbo = 0;
        
        let position_uniform: GLint;
        let scale_uniform: GLint;

        unsafe {
            position_uniform = "position".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
            scale_uniform = "scale".with_c_str(|ptr| gl::GetUniformLocation(program, ptr));
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

        let rs: @System = @RenderSystem {program: program, position_uniform: position_uniform, scale_uniform: scale_uniform} as @System;
        world.systems.push(rs);

        while !window.should_close() {
            // Poll events
            glfw::poll_events();

            // Clear the screen to black
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // process game world
            world.process();

            // Swap buffers
            window.swap_buffers();
        }

        // Cleanup
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        unsafe {
            gl::DeleteBuffers(1, &vbo);
            gl::DeleteVertexArrays(1, &vao);
        }
    }
}

fn key_callback(window: &glfw::Window, key: glfw::Key, _: libc::c_int, action: glfw::Action, _: glfw::Modifiers) {
    if action == glfw::Press && key == glfw::KeyEscape {
        window.set_should_close(true);
    }
}
