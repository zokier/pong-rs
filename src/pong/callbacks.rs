extern mod glfw;
extern mod native;

use std::libc;

pub struct ErrorContext;
impl glfw::ErrorCallback for ErrorContext {
    fn call(&self, _: glfw::Error, description: ~str) {
        println!("GLFW Error: {:s}", description);
    }
}

pub struct KeyContext;
impl glfw::KeyCallback for KeyContext {
    fn call(&self, window: &glfw::Window, key: glfw::Key, _scancode: libc::c_int, action: glfw::Action, _mods: glfw::Modifiers) {
        match (key, action) {
            (glfw::KeyEscape, glfw::Press) => {
                window.set_should_close(true);
            }

            _ => ()
        }
    }
}

pub struct FramebufferSizeContext {
    chan: Chan<(u32,u32)>
}

impl glfw::FramebufferSizeCallback for FramebufferSizeContext {
    fn call(&self, _: &glfw::Window, width: i32, height: i32) {
        self.chan.send((width as u32,height as u32));
    }
}
