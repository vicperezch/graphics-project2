// sphere.rs
use raylib::prelude::Vector3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::{Material};
use std::f32::consts::PI;

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    fn get_uv(&self, point: &Vector3) -> (f32, f32) {
        let normlaized = (*point - self.center) / self.radius;
        let u = 0.5 + normlaized.x.atan2(normlaized.z) / (2.0 * PI);
        let v = 0.5 + normlaized.y.asin() / PI;
        (u, v)
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let oc = *ray_origin - self.center;

        let a = ray_direction.dot(*ray_direction);
        let b = 2.0 * oc.dot(*ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant > 0.0 {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            let point = *ray_origin + *ray_direction * t;
            let normal = (point - self.center).normalized();
            let (u, v) = self.get_uv(&point);
            if t > 0.0 {
                return Intersect::new(
                    self.material.clone(),
                    t,
                    normal,
                    point,
                    u,
                    v
                );
            }
        }

        Intersect::empty()
    }
}