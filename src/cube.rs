use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use raylib::prelude::Vector3;

pub struct Cube {
    pub min_bounds: Vector3,
    pub max_bounds: Vector3,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vector3, size: f32, material: Material) -> Self {
        let half_size = Vector3::new(size / 2.0, size / 2.0, size / 2.0);
        Self {
            min_bounds: center - half_size,
            max_bounds: center + half_size,
            material,
        }
    }

    pub fn new_rect(
        center: Vector3,
        width: f32,
        height: f32,
        depth: f32,
        material: Material,
    ) -> Self {
        let half = Vector3::new(width / 2.0, height / 2.0, depth / 2.0);
        Self {
            min_bounds: center - half,
            max_bounds: center + half,
            material,
        }
    }

    fn get_uv(&self, point: &Vector3, normal: &Vector3) -> (f32, f32) {
        let size = self.max_bounds - self.min_bounds;
        let u: f32;
        let v: f32;

        if normal.x.abs() > 0.5 {
            u = (point.z - self.min_bounds.z) / size.z;
            v = 1.0 - (point.y - self.min_bounds.y) / size.y;
        } else if normal.y.abs() > 0.5 {
            u = (point.x - self.min_bounds.x) / size.x;
            v = (point.z - self.min_bounds.z) / size.z;
        } else {
            u = (point.x - self.min_bounds.x) / size.x;
            v = 1.0 - (point.y - self.min_bounds.y) / size.y;
        }
        (u, v)
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let inv_dir = Vector3::new(
            1.0 / ray_direction.x,
            1.0 / ray_direction.y,
            1.0 / ray_direction.z,
        );

        let mut tmin = (self.min_bounds.x - ray_origin.x) * inv_dir.x;
        let mut tmax = (self.max_bounds.x - ray_origin.x) * inv_dir.x;

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min_bounds.y - ray_origin.y) * inv_dir.y;
        let mut tymax = (self.max_bounds.y - ray_origin.y) * inv_dir.y;

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return Intersect::empty();
        }

        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min_bounds.z - ray_origin.z) * inv_dir.z;
        let mut tzmax = (self.max_bounds.z - ray_origin.z) * inv_dir.z;

        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return Intersect::empty();
        }

        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        let distance = if tmin > 0.001 { tmin } else { tmax };

        if distance < 0.001 {
            return Intersect::empty();
        }

        let point = *ray_origin + *ray_direction * distance;

        let epsilon = 1e-4;
        let mut normal = Vector3::zero();

        if (point.x - self.min_bounds.x).abs() < epsilon {
            normal.x = -1.0;
        } else if (point.x - self.max_bounds.x).abs() < epsilon {
            normal.x = 1.0;
        } else if (point.y - self.min_bounds.y).abs() < epsilon {
            normal.y = -1.0;
        } else if (point.y - self.max_bounds.y).abs() < epsilon {
            normal.y = 1.0;
        } else if (point.z - self.min_bounds.z).abs() < epsilon {
            normal.z = -1.0;
        } else if (point.z - self.max_bounds.z).abs() < epsilon {
            normal.z = 1.0;
        }

        let (u, v) = self.get_uv(&point, &normal);

        Intersect::new(self.material.clone(), distance, normal, point, u, v)
    }
}
