// SYSTEM DEFINITIONS
extern mod std;
use components::Components;

pub trait System {
    fn process(&self, entity: @Components) -> ();
}

pub struct MovementSystem;

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

pub struct EdgeCollisionSystem;

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

pub struct ScoreCollisionSystem {
    left_chan: Chan<uint>,
    right_chan: Chan<uint>
}

impl System for ScoreCollisionSystem {
    fn process(&self, entity: @Components) -> () {
        match (entity.position, entity.vert_velocity, entity.horiz_velocity) {
            (Some(pos), Some(vvel), Some(hvel)) => {
                if pos.x > 4.0 {
                    self.left_chan.send(1);
                } else if pos.x < 0.0 {
                    self.right_chan.send(1);
                } else {
                    return
                }
                pos.x = 2.0;
                pos.y = 1.5;
                hvel.x *= -1.0;
                vvel.y = 0.0;
            },
            (_, _, _) => ()
        }
    }
}

//AABB collision detection
pub fn doEntitiesCollide(a: @Components, b: @Components) -> bool {
    if std::managed::ptr_eq(a,b) {
        false
    } else {
        match (a.position, a.sprite, b.position, b.sprite) {
            (Some(a_pos), Some(a_spr), Some(b_pos), Some(b_spr)) => {
                (std::num::abs(a_pos.x - b_pos.x) * 2.0 <= (a_spr.x_size + b_spr.x_size))
                    && (std::num::abs(a_pos.y - b_pos.y) * 2.0 <= (a_spr.y_size + b_spr.y_size))
            },
            (_, _, _, _) => false
        }
    }
}

pub struct PaddleCollisionSystem {
    paddle: @Components,
}

impl System for PaddleCollisionSystem {
    fn process(&self, entity: @Components) -> () {
        if doEntitiesCollide(self.paddle, entity) {
            match (entity.horiz_velocity, entity.vert_velocity, entity.position) {
                (Some(hvel), Some(vvel), Some(pos)) => {
                    let paddle_distance = pos.y - self.paddle.position.unwrap().y;
                    let paddle_height = self.paddle.sprite.unwrap().y_size/2.0;
                    hvel.x *= -1.0;
                    vvel.y = 0.5*hvel.x*std::num::sinh(3.14*paddle_distance/paddle_height);
                },
                (_, _, _) => ()
            }
        }
    }
}

