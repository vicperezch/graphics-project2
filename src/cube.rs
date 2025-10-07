// src/cube.rs
use raylib::prelude::Vector3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct Cube {
    pub min_bounds: Vector3,
    pub max_bounds: Vector3,
    pub material: Material,
}

impl Cube {
    /// Crea un nuevo cubo a partir de un centro y un tamaño.
    pub fn new(center: Vector3, size: f32, material: Material) -> Self {
        let half_size = Vector3::new(size / 2.0, size / 2.0, size / 2.0);
        Self {
            min_bounds: center - half_size,
            max_bounds: center + half_size,
            material,
        }
    }

    /// Calcula las coordenadas UV para texturizar, basándose en el punto de intersección y la normal de la cara.
    fn get_uv(&self, point: &Vector3, normal: &Vector3) -> (f32, f32) {
        let size = self.max_bounds - self.min_bounds;
        let u: f32;
        let v: f32;

        if normal.x.abs() > 0.5 { // Caras laterales (normal en X)
            u = (point.z - self.min_bounds.z) / size.z;
            v = (point.y - self.min_bounds.y) / size.y;
        } else if normal.y.abs() > 0.5 { // Caras superior/inferior (normal en Y)
            u = (point.x - self.min_bounds.x) / size.x;
            v = (point.z - self.min_bounds.z) / size.z;
        } else { // Caras frontal/trasera (normal en Z)
            u = (point.x - self.min_bounds.x) / size.x;
            v = (point.y - self.min_bounds.y) / size.y;
        }
        (u, v)
    }
}

impl RayIntersect for Cube {
    /// Implementa el test de intersección rayo-cubo usando el método "Slab".
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let inv_dir = Vector3::new(1.0 / ray_direction.x, 1.0 / ray_direction.y, 1.0 / ray_direction.z);

        let mut tmin = (self.min_bounds.x - ray_origin.x) * inv_dir.x;
        let mut tmax = (self.max_bounds.x - ray_origin.x) * inv_dir.x;

        if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

        let mut tymin = (self.min_bounds.y - ray_origin.y) * inv_dir.y;
        let mut tymax = (self.max_bounds.y - ray_origin.y) * inv_dir.y;

        if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

        if (tmin > tymax) || (tymin > tmax) {
            return Intersect::empty();
        }

        if tymin > tmin { tmin = tymin; }
        if tymax < tmax { tmax = tymax; }

        let mut tzmin = (self.min_bounds.z - ray_origin.z) * inv_dir.z;
        let mut tzmax = (self.max_bounds.z - ray_origin.z) * inv_dir.z;

        if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

        if (tmin > tzmax) || (tzmin > tmax) {
            return Intersect::empty();
        }

        if tzmin > tmin { tmin = tzmin; }
        if tzmax < tmax { tmax = tzmax; }

        // Si tmin es negativo, el rayo empieza dentro del cubo, usamos tmax.
        let distance = if tmin > 0.001 { tmin } else { tmax };

        // Si la distancia es demasiado pequeña o negativa, no hay intersección visible.
        if distance < 0.001 {
            return Intersect::empty();
        }

        let point = *ray_origin + *ray_direction * distance;
        
        // Se determina la normal de la cara intersectada comparando la posición del punto
        // con los límites del cubo.
        let epsilon = 1e-4;
        let mut normal = Vector3::zero();

        if (point.x - self.min_bounds.x).abs() < epsilon { normal.x = -1.0; }
        else if (point.x - self.max_bounds.x).abs() < epsilon { normal.x = 1.0; }
        else if (point.y - self.min_bounds.y).abs() < epsilon { normal.y = -1.0; }
        else if (point.y - self.max_bounds.y).abs() < epsilon { normal.y = 1.0; }
        else if (point.z - self.min_bounds.z).abs() < epsilon { normal.z = -1.0; }
        else if (point.z - self.max_bounds.z).abs() < epsilon { normal.z = 1.0; }

        let (u, v) = self.get_uv(&point, &normal);

        Intersect::new(
            self.material.clone(),
            distance,
            normal,
            point,
            u,
            v,
        )
    }
}