// light.rs
use raylib::prelude::*;

pub struct Light {
    pub position: Vector3,
    pub color: Vector3,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vector3, color: Vector3, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }
}
