// WORLD DEFINITION
extern mod glfw;
use components::Components;
use systems::System;
use globalsystems::GlobalSystem;

// We need to figure out how to integrate World with the main game loop
// in Artemis world has a `setDelta` method for timestep
pub struct World {
    entities: ~[@Components],
    systems: ~[@System],
    global_systems: ~[@mut GlobalSystem]
}

impl World {
    pub fn new() -> World {
        return World {entities: ~[], systems: ~[], global_systems: ~[]};
    }

    pub fn process(&self, window: &glfw::Window) {
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
