// Tutorial 04
// Destroy the asteroid. The target is in a random
// location given by the "target()" function.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: "angle_diff(heading(), target().angle())" returns the direction
        // your ship needs to turn to face the target.
        torque(1.0);
        fire(0);
    }
}