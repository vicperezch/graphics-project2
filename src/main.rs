#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
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

use camera::Camera;
use cube::Cube;
use framebuffer::Framebuffer;
use ray_intersect::{Intersect, RayIntersect};

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}
use light::Light;
use material::{Material, vector3_to_color};

unsafe impl Send for Light {}
unsafe impl Sync for Light {}
use bvh::BVHNode;
use snell::{reflect, refract};
use textures::TextureManager;

fn procedural_sky(dir: Vector3) -> Vector3 {
    let d = dir.normalized();
    let t = (d.y + 1.0) * 0.5;

    let green = Vector3::new(0.1, 0.6, 0.2);
    let white = Vector3::new(1.0, 1.0, 1.0);
    let blue = Vector3::new(0.3, 0.5, 1.0);

    if t < 0.54 {
        let k = t / 0.55;
        green * (1.0 - k) + white * k
    } else if t < 0.55 {
        white
    } else if t < 0.8 {
        let k = (t - 0.55) / 0.25;
        white * (1.0 - k) + blue * k
    } else {
        blue
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
    light: &Light,
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    if depth > 3 {
        return procedural_sky(*ray_direction);
    }

    let inv_dir = Vector3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );

    let intersect = bvh.intersect(objects, ray_origin, ray_direction, &inv_dir);

    if !intersect.is_intersecting {
        return procedural_sky(*ray_direction);
    }

    let light_direction = (light.position - intersect.point).normalized();
    let view_direction = (*ray_origin - intersect.point).normalized();

    let normal = intersect.normal;

    let reflection_direction = reflect(&-light_direction, &normal).normalized();

    let diffuse_intensity = normal.dot(light_direction).max(0.0);
    let shadow_intensity = if diffuse_intensity > 0.01 {
        cast_shadow(&intersect, light, bvh, objects)
    } else {
        0.0
    };

    let light_intensity = light.intensity * (1.0 - shadow_intensity);
    let final_diffuse_intensity = diffuse_intensity * light_intensity;

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

    let diffuse = diffuse_color * final_diffuse_intensity;

    let specular_intensity = view_direction
        .dot(reflection_direction)
        .max(0.0)
        .powf(intersect.material.specular)
        * light_intensity;
    let specular = light.color * specular_intensity;

    let mut reflection_color = Vector3::zero();
    let reflectivity = intersect.material.reflectivity;

    if reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &normal);
        let reflect_origin = intersect.point + normal * ORIGIN_BIAS;
        reflection_color = cast_ray(
            &reflect_origin,
            &reflect_direction,
            bvh,
            objects,
            light,
            depth + 1,
            texture_manager,
        );
    }

    let transparency = intersect.material.transparency;
    let mut refraction_color = Vector3::zero();

    if transparency > 0.0 {
        let refract_direction =
            refract(ray_direction, &normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_direction);
        refraction_color = cast_ray(
            &refract_origin,
            &refract_direction,
            bvh,
            objects,
            light,
            depth + 1,
            texture_manager,
        );
    }

    diffuse * intersect.material.albedo[0]
        + specular * intersect.material.albedo[1]
        + reflection_color * reflectivity
        + refraction_color * transparency
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
    light: &Light,
    texture_manager: &TextureManager,
    config: &RenderConfig,
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

            let pixel_color_vec = cast_ray(
                &camera.eye,
                &rotated_direction,
                bvh,
                objects,
                light,
                0,
                texture_manager,
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
    light: &Light,
    texture_manager: &TextureManager,
    config: &RenderConfig,
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

            let handle = s.spawn(move || {
                let pixels = render_row_range(
                    start_y,
                    end_y,
                    width,
                    bvh,
                    objects,
                    camera,
                    light,
                    texture_manager,
                    config,
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
        .title("Raytracer - Multithreaded Minecraft Diorama")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/obsidian.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/shroomlight.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/crimson_nylium.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/crimson_stem.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/blackstone.png");

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32);

    let obsidian = Material {
        diffuse: Vector3::new(0.1, 0.05, 0.15),
        albedo: [0.8, 0.3],
        specular: 90.0,
        reflectivity: 0.2,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/obsidian.png".to_string()),
        normal_map_id: None,
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
    };

    let blackstone = Material {
        diffuse: Vector3::new(0.15, 0.15, 0.18),
        albedo: [0.9, 0.1],
        specular: 20.0,
        reflectivity: 0.05,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/blackstone.png".to_string()),
        normal_map_id: None,
    };

    let objects = vec![
        Cube::new(Vector3::new(-3.0, 0.0, 0.0), 1.5, obsidian),
        Cube::new(Vector3::new(-1.0, 0.0, 0.0), 1.5, shroomlight),
        Cube::new(Vector3::new(1.0, 0.0, 0.0), 1.5, crimson_nylium),
        Cube::new(Vector3::new(3.0, 0.0, 0.0), 1.5, crimson_stem),
        Cube::new(Vector3::new(0.0, 0.0, -2.0), 1.5, blackstone),
    ];

    let mut indices: Vec<usize> = (0..objects.len()).collect();
    let bvh = BVHNode::build(&objects, &mut indices);

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;

    let light = Light::new(
        Vector3::new(5.0, 5.0, 5.0),
        Vector3::new(1.0, 1.0, 1.0),
        1.5,
    );

    let render_config = RenderConfig::new(window_width as i32, window_height as i32, PI / 3.0);

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
            &light,
            &texture_manager,
            &render_config,
        );

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
