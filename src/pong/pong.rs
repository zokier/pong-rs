#[feature(globs)];
#[feature(macro_rules)];
#[feature(managed_boxes)]; //TODO do without managed boxes

extern mod glfw;
extern mod gl;
extern mod native;

use main::main;

mod callbacks;
mod components;
mod entities;
mod globalsystems;
mod graphics;
mod main;
mod systems;
mod world;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}
