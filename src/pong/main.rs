// Copyright 2013-2014 Torste Aikio and others.
// See AUTHORS file for details.

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

extern mod glfw;
extern mod gl;
extern mod std;
use gl::types::*;

use callbacks::*;
use entities::*;
use components::*;
use globalsystems::*;
use systems::*;
use graphics::RenderSystem;
use world::World;

pub fn main() {
    glfw::set_error_callback(~ErrorContext);

    do glfw::start {
        // initialize game world
        let left_paddle: @Components = new_paddle(LEFT);
        let right_paddle: @Components = new_paddle(RIGHT);
        let ball: @Components = new_ball();
        let left_score_counter: @Components = new_score_counter(LEFT);
        let right_score_counter: @Components = new_score_counter(RIGHT);
        let background: @Components = new_background();
        let background_2: @Components = new_background_2();
        let ms = @MovementSystem;
        let es = @EdgeCollisionSystem;
        let (left_score_port, left_score_chan): (Port<uint>, Chan<uint>) = std::comm::Chan::new();
        let (right_score_port, right_score_chan): (Port<uint>, Chan<uint>) = std::comm::Chan::new();
        let ss = @ScoreCollisionSystem { left_chan: left_score_chan, right_chan: right_score_chan };
        let lps = @PaddleCollisionSystem{ paddle: left_paddle };
        let rps = @PaddleCollisionSystem{ paddle: right_paddle };

        let mut world: World = World::new();
        world.entities.push(background);
        world.entities.push(background_2);
        world.entities.push(left_score_counter);
        world.entities.push(right_score_counter);
        world.entities.push(left_paddle);
        world.entities.push(right_paddle);
        world.entities.push(ball);
        world.systems.push(ms as @System);
        world.systems.push(es as @System);
        world.systems.push(ss as @System);
        world.systems.push(lps as @System);
        world.systems.push(rps as @System);

        // Choose a GL profile that is compatible with OS X 10.7+
        glfw::window_hint::context_version(3, 2);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let mut window_width = 800;
        let mut window_height = 480;
        let window = glfw::Window::create(window_width, window_height, "Pong", glfw::Windowed).expect("Failed to create GLFW window.");;
        window.set_key_callback(~KeyContext);
        window.make_context_current();

        // Load the OpenGL function pointers
        gl::load_with(glfw::get_proc_address);

        let rs = @RenderSystem::new();

        let (fb_size_port, fb_size_chan): (Port<(u32,u32)>, Chan<(u32,u32)>) = std::comm::Chan::new();
        window.set_framebuffer_size_callback(~FramebufferSizeContext { chan: fb_size_chan });

        world.systems.push(rs as @System);

        let kbs = @mut KeyboardInputSystem { paddle: left_paddle };
        world.global_systems.push(kbs as @mut GlobalSystem);

        let bis = @mut BotInputSystem { paddle: right_paddle, ball: ball };
        world.global_systems.push(bis as @mut GlobalSystem);

        // score update systems need to be mutable as they maintain the score within
        let lsus = @mut ScoreUpdateSystem { paddle: left_paddle, counter: left_score_counter, score: 0, port: left_score_port };
        world.global_systems.push(lsus as @mut GlobalSystem);
        let rsus = @mut ScoreUpdateSystem { paddle: right_paddle, counter: right_score_counter, score: 0, port: right_score_port };
        world.global_systems.push(rsus as @mut GlobalSystem);

        while !window.should_close() {
            // Poll events
            glfw::poll_events();

            loop {
                match fb_size_port.try_recv() {
                    Some((w,h)) => {
                        window_width = w;
                        window_height = h;
                    }
                    None => break
                }
            }

            gl::Viewport(0,0, window_width as GLint, window_height as GLint);
            gl::ProgramUniform2f(rs.program, rs.window_uniform, window_width as f32, window_height as f32);
            // Clear the screen
            gl::ClearColor(0.8, 0.8, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // process game world
            world.process(&window);

            // Swap buffers
            window.swap_buffers();
        }
    }
}

