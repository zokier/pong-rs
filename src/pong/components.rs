use gl::types::*;

// COMPONENT DEFINITIONS
pub struct Position {
    x: f64,
    y: f64
}

pub struct HorizVelocity {
    x: f64
}

pub struct VertVelocity {
    y: f64
}

pub struct SpriteTexture {
    texture: GLuint,
    texcoords: (uint, uint),
    texsize: (uint, uint)
}

// takes single ascii character as a byte as input
pub fn texture_from_byte(b: u8) -> SpriteTexture {
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

pub fn texture_from_char(c: char) -> SpriteTexture {
    texture_from_byte(c.to_ascii().to_byte())
}

pub fn texture_from_uint(i: uint) -> SpriteTexture {
    texture_from_byte(((i+0x30) as u8))
}

pub struct Sprite {
    x_size: f64,
    y_size: f64,
    //instead of color+texture we should have something like material
    //which could be eg enum
    color: [f64, ..4],
    texture: Option<SpriteTexture>
}

pub struct Components {
    position: Option<@mut Position>,
    horiz_velocity: Option<@mut HorizVelocity>,
    vert_velocity: Option<@mut VertVelocity>,
    sprite: Option<@mut Sprite>,
}

