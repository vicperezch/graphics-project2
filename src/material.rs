// material.rs
use raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Vector3,
    pub albedo: [f32; 2],
    pub specular: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
    pub texture: Option<String>,
    pub normal_map_id: Option<String>,
    pub emission: Vector3,
    pub emission_strength: f32,
}

impl Material {
    pub fn new(
        diffuse: Vector3,
        albedo: [f32; 2],
        specular: f32,
        reflectivity: f32,
        transparency: f32,
        refractive_index: f32,
        texture: Option<String>,
        normal_map_id: Option<String>,
        emission: Vector3,
        emission_strength: f32,
    ) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
            reflectivity,
            transparency,
            refractive_index,
            texture,
            normal_map_id,
            emission,
            emission_strength,
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
            emission: Vector3::zero(),
            emission_strength: 0.0,
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
