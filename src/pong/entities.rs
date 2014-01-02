use components::*;

//ENTITY CONSTRUCTORS
pub enum PaddleSide {
    RIGHT,
    LEFT
}

pub fn new_ball() -> @Components {
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
    }
}

pub fn new_paddle(side: PaddleSide) -> @Components {
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
    }
}

pub fn new_background_2() -> @Components {
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
    }
}

pub fn new_background() -> @Components {
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
    }
}

pub fn new_score_counter(side: PaddleSide) -> @Components {
    let xpos = match side {
        RIGHT => 2.5,
        LEFT => 1.5
    };
    @Components {
        position: Some(@mut Position { x: xpos, y: 2.5 }),
        horiz_velocity: None,
        vert_velocity: None,
        sprite: Some(@mut Sprite {
            x_size: 0.3,
            y_size: 0.6,
            color: [1.0, 1.0, 1.0, 0.0],
            texture: Some(texture_from_char('0'))
        }),
    }
}
