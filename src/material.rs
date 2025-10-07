// material.rs
use raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Vector3, // Color
    pub albedo: [f32; 2], // que tan colorido es: [color del objeto, color que viene de la luz]
    pub specular: f32, // brillo
    pub reflectivity: f32, // reflectividad, 1.0 espejo, 0.0 no refleja nada
    pub transparency: f32, // transparencia, 1.0 perfectamente transparente, 0.0 no transparente
    pub refractive_index: f32, // indice de refraccion
    pub texture: Option<String>, // path to texture
    pub normal_map_id: Option<String>, // path to normal map
}

impl Material {
    pub fn new(diffuse: Vector3, albedo: [f32; 2], specular: f32, reflectivity: f32, transparency: f32, refractive_index: f32, texture: Option<String>, normal_map_id: Option<String>) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
            reflectivity,
            transparency,
            refractive_index,
            texture,
            normal_map_id,
        }
    }
    
    pub fn black() -> Self {
        Material {
            diffuse: Vector3::zero(),
            albedo: [0.0, 0.0],
            specular: 0.0,
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: 0.0,
            texture: None,
            normal_map_id: None,
        }
    }
}

pub fn vector3_to_color(v: Vector3) -> Color {
    Color::new(
        (v.x * 255.0).min(255.0) as u8,
        (v.y * 255.0).min(255.0) as u8,
        (v.z * 255.0).min(255.0) as u8,
        255,
    )
}

pub fn color_to_vector3(color: Color) -> Vector3 {
    Vector3::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    )
}