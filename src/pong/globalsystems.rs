extern mod glfw;
extern mod std;
use components::{Components,texture_from_uint};

//GLOBAL SYSTEM DEFINITIONS
pub trait GlobalSystem {
    fn process(&mut self, window: &glfw::Window) -> ();
}

pub struct ScoreUpdateSystem {
    paddle: @Components,
    counter: @Components,
    score: uint,
    port: Port<uint>
}

impl GlobalSystem for ScoreUpdateSystem {
    fn process(&mut self, _: &glfw::Window) -> () {
        loop {
            match self.port.try_recv() {
                Some(i) => {
                    self.score += i;
                }
                None => break
            }
        }
        self.counter.sprite.unwrap().texture = Some(texture_from_uint(self.score));
    }
}

pub struct BotInputSystem {
    paddle: @Components,
    ball: @Components
}

impl GlobalSystem for BotInputSystem {
    fn process(&mut self, _: &glfw::Window) -> () {
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

pub struct KeyboardInputSystem {
    paddle: @Components
}

impl GlobalSystem for KeyboardInputSystem {
    fn process(&mut self, window: &glfw::Window) -> () {
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
