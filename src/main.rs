#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::thread;

mod bvh;
mod camera;
mod cube;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod snell;
mod textures;

use bvh::BVHNode;
use camera::Camera;
use cube::Cube;
use framebuffer::Framebuffer;
use light::Light;
use material::{Material, vector3_to_color};
use ray_intersect::{Intersect, RayIntersect};
use snell::{reflect, refract};
use textures::TextureManager;

pub enum SceneObject {
    Cube(Cube),
}

impl RayIntersect for SceneObject {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        match self {
            SceneObject::Cube(cube) => cube.ray_intersect(ray_origin, ray_direction),
        }
    }
}

fn load_scene_from_file(
    filepath: &str,
    materials: &std::collections::HashMap<String, Material>,
) -> Result<Vec<Cube>, String> {
    let contents = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Failed to read scene file '{}': {}", filepath, e))?;

    let mut cubes = Vec::new();

    for (line_num, line) in contents.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.len() != 5 {
            return Err(format!(
                "Line {}: Expected 5 parameters (x y z size material), got {}",
                line_num + 1,
                parts.len()
            ));
        }

        let x = parts[0]
            .parse::<f32>()
            .map_err(|_| format!("Line {}: Invalid x coordinate '{}'", line_num + 1, parts[0]))?;
        let y = parts[1]
            .parse::<f32>()
            .map_err(|_| format!("Line {}: Invalid y coordinate '{}'", line_num + 1, parts[1]))?;
        let z = parts[2]
            .parse::<f32>()
            .map_err(|_| format!("Line {}: Invalid z coordinate '{}'", line_num + 1, parts[2]))?;
        let size = parts[3]
            .parse::<f32>()
            .map_err(|_| format!("Line {}: Invalid size '{}'", line_num + 1, parts[3]))?;
        let material_name = parts[4];

        let material = materials.get(material_name).ok_or_else(|| {
            format!(
                "Line {}: Unknown material '{}'. Available: {}",
                line_num + 1,
                material_name,
                materials
                    .keys()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

        cubes.push(Cube::new(Vector3::new(x, y, z), size, material.clone()));
    }

    Ok(cubes)
}

fn procedural_sky(
    dir: Vector3,
    texture_manager: &TextureManager,
    skybox_texture: Option<&str>,
) -> Vector3 {
    if let Some(skybox_path) = skybox_texture {
        if let Some(texture) = texture_manager.get_texture(skybox_path) {
            let d = dir.normalized();

            let theta = (-d.x).atan2(-d.z);
            let phi = d.y.asin();

            let u = 0.5 + theta / (2.0 * PI);
            let v = 0.5 - phi / PI;

            let u_clamped = u.clamp(0.0, 0.9999);
            let v_clamped = v.clamp(0.0, 0.9999);

            let width = texture.width() as f32;
            let height = texture.height() as f32;
            let tx = (u_clamped * width) as u32;
            let ty = (v_clamped * height) as u32;

            return texture_manager.get_pixel_color(skybox_path, tx, ty);
        }
    }

    let d = dir.normalized();
    let t = (d.y + 1.0) * 0.5;

    let dark_red = Vector3::new(0.2, 0.05, 0.05);
    let crimson = Vector3::new(0.4, 0.08, 0.1);
    let dark_orange = Vector3::new(0.3, 0.1, 0.05);

    if t < 0.3 {
        dark_red
    } else if t < 0.6 {
        let k = (t - 0.3) / 0.3;
        dark_red * (1.0 - k) + crimson * k
    } else {
        let k = (t - 0.6) / 0.4;
        crimson * (1.0 - k) + dark_orange * k
    }
}

fn cast_shadow(intersect: &Intersect, light: &Light, bvh: &BVHNode, objects: &[Cube]) -> f32 {
    let light_dir = (light.position - intersect.point).normalized();
    let shadow_origin = intersect.point + intersect.normal * 1e-4;
    let inv_dir = Vector3::new(1.0 / light_dir.x, 1.0 / light_dir.y, 1.0 / light_dir.z);

    let shadow_hit = bvh.intersect(objects, &shadow_origin, &light_dir, &inv_dir);

    if shadow_hit.is_intersecting {
        let light_distance = (light.position - intersect.point).length();
        if shadow_hit.distance < light_distance {
            return 0.7;
        }
    }
    0.0
}

const ORIGIN_BIAS: f32 = 1e-4;

fn offset_origin(intersect: &Intersect, ray_direction: &Vector3) -> Vector3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if ray_direction.dot(intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    bvh: &BVHNode,
    objects: &[Cube],
    lights: &[Light],
    depth: u32,
    texture_manager: &TextureManager,
    skybox_texture: Option<&str>,
) -> Vector3 {
    if depth > 2 {
        return procedural_sky(*ray_direction, texture_manager, skybox_texture);
    }

    let inv_dir = Vector3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );

    let intersect = bvh.intersect(objects, ray_origin, ray_direction, &inv_dir);

    if !intersect.is_intersecting {
        return procedural_sky(*ray_direction, texture_manager, skybox_texture);
    }

    let view_direction = (*ray_origin - intersect.point).normalized();
    let normal = intersect.normal;

    let mut total_diffuse = Vector3::zero();
    let mut total_specular = Vector3::zero();

    for light in lights {
        let light_direction = (light.position - intersect.point).normalized();

        let diffuse_intensity = normal.dot(light_direction).max(0.0);

        if diffuse_intensity < 0.01 {
            continue;
        }

        let shadow_intensity = cast_shadow(&intersect, light, bvh, objects);
        let light_intensity = light.intensity * (1.0 - shadow_intensity);
        let final_diffuse_intensity = diffuse_intensity * light_intensity;

        total_diffuse = total_diffuse + light.color * final_diffuse_intensity;

        let reflection_direction = reflect(&-light_direction, &normal).normalized();
        let specular_intensity = view_direction
            .dot(reflection_direction)
            .max(0.0)
            .powf(intersect.material.specular)
            * light_intensity;
        total_specular = total_specular + light.color * specular_intensity;
    }

    let diffuse_color = if let Some(texture_path) = &intersect.material.texture {
        if let Some(texture) = texture_manager.get_texture(texture_path) {
            let width = texture.width() as u32;
            let height = texture.height() as u32;
            let tx = (intersect.u * width as f32) as u32;
            let ty = (intersect.v * height as f32) as u32;
            texture_manager.get_pixel_color(texture_path, tx, ty)
        } else {
            intersect.material.diffuse
        }
    } else {
        intersect.material.diffuse
    };

    let diffuse = diffuse_color * total_diffuse;
    let specular = total_specular;

    let mut reflection_color = Vector3::zero();
    let reflectivity = intersect.material.reflectivity;

    if reflectivity > 0.05 {
        let reflect_direction = reflect(ray_direction, &normal);
        let reflect_origin = intersect.point + normal * ORIGIN_BIAS;
        reflection_color = cast_ray(
            &reflect_origin,
            &reflect_direction,
            bvh,
            objects,
            lights,
            depth + 1,
            texture_manager,
            skybox_texture,
        );
    }

    let transparency = intersect.material.transparency;
    let mut refraction_color = Vector3::zero();

    if transparency > 0.05 {
        let refract_direction =
            refract(ray_direction, &normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_direction);
        refraction_color = cast_ray(
            &refract_origin,
            &refract_direction,
            bvh,
            objects,
            lights,
            depth + 1,
            texture_manager,
            skybox_texture,
        );
    }

    let emissive = if intersect.material.emission_strength > 0.01 {
        diffuse_color * intersect.material.emission * intersect.material.emission_strength
    } else {
        Vector3::zero()
    };

    diffuse * intersect.material.albedo[0]
        + specular * intersect.material.albedo[1]
        + reflection_color * reflectivity
        + refraction_color * transparency
        + emissive
}

pub struct RenderConfig {
    pub aspect_ratio: f32,
    pub perspective_scale: f32,
    pub inv_width: f32,
    pub inv_height: f32,
}

impl RenderConfig {
    pub fn new(width: i32, height: i32, fov: f32) -> Self {
        let w = width as f32;
        let h = height as f32;
        RenderConfig {
            aspect_ratio: w / h,
            perspective_scale: (fov * 0.5).tan(),
            inv_width: 1.0 / w,
            inv_height: 1.0 / h,
        }
    }
}

struct RowRange {
    start: i32,
    end: i32,
    pixels: Vec<Color>,
}

pub fn render_row_range(
    start_y: i32,
    end_y: i32,
    width: i32,
    bvh: &BVHNode,
    objects: &[Cube],
    camera: &Camera,
    lights: &[Light],
    texture_manager: &TextureManager,
    config: &RenderConfig,
    skybox_texture: Option<String>,
) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(((end_y - start_y) * width) as usize);

    for y in start_y..end_y {
        for x in 0..width {
            let screen_x = (2.0 * x as f32 * config.inv_width - 1.0)
                * config.aspect_ratio
                * config.perspective_scale;
            let screen_y = (1.0 - 2.0 * y as f32 * config.inv_height) * config.perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);

            let skybox_ref = skybox_texture.as_deref();
            let pixel_color_vec = cast_ray(
                &camera.eye,
                &rotated_direction,
                bvh,
                objects,
                lights,
                0,
                texture_manager,
                skybox_ref,
            );
            let pixel_color = vector3_to_color(pixel_color_vec);

            pixels.push(pixel_color);
        }
    }

    pixels
}

pub fn render(
    framebuffer: &mut Framebuffer,
    bvh: &BVHNode,
    objects: &[Cube],
    camera: &Camera,
    lights: &[Light],
    texture_manager: &TextureManager,
    config: &RenderConfig,
    skybox_texture: Option<String>,
) {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let height = framebuffer.height;
    let width = framebuffer.width;
    let rows_per_thread = (height as f32 / num_threads as f32).ceil() as i32;

    let results = thread::scope(|s| {
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let start_y = thread_id as i32 * rows_per_thread;
            let end_y = ((thread_id as i32 + 1) * rows_per_thread).min(height);

            if start_y >= height {
                break;
            }

            let skybox_clone = skybox_texture.clone();

            let handle = s.spawn(move || {
                let pixels = render_row_range(
                    start_y,
                    end_y,
                    width,
                    bvh,
                    objects,
                    camera,
                    lights,
                    texture_manager,
                    config,
                    skybox_clone,
                );

                RowRange {
                    start: start_y,
                    end: end_y,
                    pixels,
                }
            });

            handles.push(handle);
        }

        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    for row_range in results {
        let mut pixel_idx = 0;

        for y in row_range.start..row_range.end {
            for x in 0..width {
                let color = row_range.pixels[pixel_idx];
                framebuffer.set_current_color(color);
                framebuffer.set_pixel(x, y);
                pixel_idx += 1;
            }
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer - Nether Crimson Forest")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/obsidian.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/shroomlight.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/crimson_nylium.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/crimson_stem.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/nether_wart_block.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/portal.png");

    let skybox_texture = if std::path::Path::new("assets/nether_skybox.png").exists() {
        texture_manager.load_texture(&mut window, &raylib_thread, "assets/nether_skybox.png");
        Some("assets/nether_skybox.png".to_string())
    } else {
        None
    };

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32);
    framebuffer.set_background_color(Color::new(51, 13, 13, 255));

    let obsidian = Material {
        diffuse: Vector3::new(0.15, 0.1, 0.2),
        albedo: [0.9, 0.1],
        specular: 90.0,
        reflectivity: 0.1,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/obsidian.png".to_string()),
        normal_map_id: None,
        emission: Vector3::zero(),
        emission_strength: 0.0,
    };

    let shroomlight = Material {
        diffuse: Vector3::new(0.95, 0.6, 0.3),
        albedo: [0.9, 0.1],
        specular: 15.0,
        reflectivity: 0.0,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/shroomlight.png".to_string()),
        normal_map_id: None,
        emission: Vector3::new(1.0, 0.45, 0.15),
        emission_strength: 1.2,
    };

    let crimson_nylium = Material {
        diffuse: Vector3::new(0.5, 0.1, 0.15),
        albedo: [0.95, 0.05],
        specular: 5.0,
        reflectivity: 0.0,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/crimson_nylium.png".to_string()),
        normal_map_id: None,
        emission: Vector3::zero(),
        emission_strength: 0.0,
    };

    let crimson_stem = Material {
        diffuse: Vector3::new(0.4, 0.15, 0.35),
        albedo: [0.85, 0.15],
        specular: 15.0,
        reflectivity: 0.0,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/crimson_stem.png".to_string()),
        normal_map_id: None,
        emission: Vector3::zero(),
        emission_strength: 0.0,
    };

    let nether_wart_block = Material {
        diffuse: Vector3::new(0.5, 0.05, 0.08),
        albedo: [0.95, 0.05],
        specular: 8.0,
        reflectivity: 0.0,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/nether_wart_block.png".to_string()),
        normal_map_id: None,
        emission: Vector3::zero(),
        emission_strength: 0.0,
    };

    let portal = Material {
        diffuse: Vector3::new(0.8, 0.8, 0.8),
        albedo: [0.9, 0.1],
        specular: 10.0,
        reflectivity: 0.0,
        transparency: 0.5,
        refractive_index: 1.3,
        texture: Some("assets/portal.png".to_string()),
        normal_map_id: None,
        emission: Vector3::zero(),
        emission_strength: 0.0,
    };

    let mut materials = std::collections::HashMap::new();
    materials.insert("obsidian".to_string(), obsidian.clone());
    materials.insert("shroomlight".to_string(), shroomlight.clone());
    materials.insert("crimson_nylium".to_string(), crimson_nylium.clone());
    materials.insert("crimson_stem".to_string(), crimson_stem.clone());
    materials.insert("nether_wart_block".to_string(), nether_wart_block.clone());
    materials.insert("portal".to_string(), portal.clone());

    let objects = if std::path::Path::new("scene.txt").exists() {
        match load_scene_from_file("scene.txt", &materials) {
            Ok(cubes) => cubes,
            Err(e) => {
                eprintln!("Error loading scene: {}", e);
                eprintln!("Using default scene instead.");
                vec![
                    Cube::new(Vector3::new(-2.5, 0.0, 0.0), 1.5, obsidian),
                    Cube::new(Vector3::new(0.0, 0.0, -1.0), 1.5, shroomlight),
                    Cube::new(Vector3::new(2.5, 0.0, 0.0), 1.5, crimson_nylium),
                    Cube::new(Vector3::new(-1.5, 0.0, 2.0), 1.5, crimson_stem),
                    Cube::new(Vector3::new(1.5, 0.0, 2.0), 1.5, nether_wart_block),
                    Cube::new(Vector3::new(0.0, 0.0, 3.0), 1.5, portal),
                ]
            }
        }
    } else {
        vec![
            Cube::new(Vector3::new(-2.5, 0.0, 0.0), 1.5, obsidian),
            Cube::new(Vector3::new(0.0, 0.0, -1.0), 1.5, shroomlight),
            Cube::new(Vector3::new(2.5, 0.0, 0.0), 1.5, crimson_nylium),
            Cube::new(Vector3::new(-1.5, 0.0, 2.0), 1.5, crimson_stem),
            Cube::new(Vector3::new(1.5, 0.0, 2.0), 1.5, nether_wart_block),
            Cube::new(Vector3::new(0.0, 0.0, 3.0), 1.5, portal),
        ]
    };

    let mut indices: Vec<usize> = (0..objects.len()).collect();
    let bvh = BVHNode::build(&objects, &mut indices);

    let mut camera = Camera::new(
        Vector3::new(0.0, 2.0, 8.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;

    let light1 = Light::new(
        Vector3::new(5.0, 8.0, 5.0),
        Vector3::new(1.0, 0.7, 0.5),
        1.3,
    );

    let mut lights = vec![light1];

    for obj in objects.iter() {
        if obj.material.emission_strength > 0.0 {
            let center = (obj.min_bounds + obj.max_bounds) * 0.5;
            let emissive_light = Light::new(
                center,
                obj.material.emission,
                obj.material.emission_strength * 2.0,
            );
            lights.push(emissive_light);
        }
    }

    let render_config = RenderConfig::new(window_width as i32, window_height as i32, PI / 3.0);

    let mut frame_count = 0;
    let mut fps_timer = std::time::Instant::now();

    while !window.window_should_close() {
        framebuffer.clear();

        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, rotation_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_W) {
            camera.zoom(zoom_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            camera.zoom(-zoom_speed);
        }

        render(
            &mut framebuffer,
            &bvh,
            &objects,
            &camera,
            &lights,
            &texture_manager,
            &render_config,
            skybox_texture.clone(),
        );

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        frame_count += 1;
        let elapsed = fps_timer.elapsed().as_secs_f32();
        if elapsed >= 2.0 {
            let fps = frame_count as f32 / elapsed;
            println!("FPS: {:.1}", fps);
            frame_count = 0;
            fps_timer = std::time::Instant::now();
        }
    }
}

