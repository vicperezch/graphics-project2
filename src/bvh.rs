// bvh.rs - Bounding Volume Hierarchy for spatial acceleration
use crate::cube::Cube;
use crate::ray_intersect::{Intersect, RayIntersect};
use raylib::prelude::*;

pub struct AABB {
    pub min: Vector3,
    pub max: Vector3,
}

impl AABB {
    pub fn from_cube(cube: &Cube) -> Self {
        AABB {
            min: cube.min_bounds,
            max: cube.max_bounds,
        }
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: Vector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    pub fn intersect(&self, ray_origin: &Vector3, inv_dir: &Vector3) -> bool {
        let mut tmin = (self.min.x - ray_origin.x) * inv_dir.x;
        let mut tmax = (self.max.x - ray_origin.x) * inv_dir.x;
        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min.y - ray_origin.y) * inv_dir.y;
        let mut tymax = (self.max.y - ray_origin.y) * inv_dir.y;
        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if tmin > tymax || tymin > tmax {
            return false;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min.z - ray_origin.z) * inv_dir.z;
        let mut tzmax = (self.max.z - ray_origin.z) * inv_dir.z;
        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if tmin > tzmax || tzmin > tmax {
            return false;
        }

        true
    }

    pub fn center(&self) -> Vector3 {
        Vector3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }
}

pub enum BVHNode {
    Leaf {
        bounds: AABB,
        object_idx: usize,
    },
    Internal {
        bounds: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
}

impl BVHNode {
    pub fn build(cubes: &[Cube], indices: &mut [usize]) -> Self {
        if indices.len() == 1 {
            let idx = indices[0];
            return BVHNode::Leaf {
                bounds: AABB::from_cube(&cubes[idx]),
                object_idx: idx,
            };
        }

        let mut bounds = AABB::from_cube(&cubes[indices[0]]);
        for &idx in indices.iter().skip(1) {
            bounds = bounds.merge(&AABB::from_cube(&cubes[idx]));
        }

        let extent = Vector3::new(
            bounds.max.x - bounds.min.x,
            bounds.max.y - bounds.min.y,
            bounds.max.z - bounds.min.z,
        );

        let axis = if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        };

        indices.sort_by(|&a, &b| {
            let ca = AABB::from_cube(&cubes[a]).center();
            let cb = AABB::from_cube(&cubes[b]).center();
            let va = match axis {
                0 => ca.x,
                1 => ca.y,
                _ => ca.z,
            };
            let vb = match axis {
                0 => cb.x,
                1 => cb.y,
                _ => cb.z,
            };
            va.partial_cmp(&vb).unwrap()
        });

        let mid = indices.len() / 2;
        let (left_indices, right_indices) = indices.split_at_mut(mid);

        BVHNode::Internal {
            bounds,
            left: Box::new(BVHNode::build(cubes, left_indices)),
            right: Box::new(BVHNode::build(cubes, right_indices)),
        }
    }

    pub fn intersect<'a>(
        &self,
        cubes: &'a [Cube],
        ray_origin: &Vector3,
        ray_direction: &Vector3,
        inv_dir: &Vector3,
    ) -> Intersect {
        match self {
            BVHNode::Leaf { bounds, object_idx } => {
                if bounds.intersect(ray_origin, inv_dir) {
                    cubes[*object_idx].ray_intersect(ray_origin, ray_direction)
                } else {
                    Intersect::empty()
                }
            }
            BVHNode::Internal {
                bounds,
                left,
                right,
            } => {
                if !bounds.intersect(ray_origin, inv_dir) {
                    return Intersect::empty();
                }

                let left_hit = left.intersect(cubes, ray_origin, ray_direction, inv_dir);
                let right_hit = right.intersect(cubes, ray_origin, ray_direction, inv_dir);

                if left_hit.is_intersecting && right_hit.is_intersecting {
                    if left_hit.distance < right_hit.distance {
                        left_hit
                    } else {
                        right_hit
                    }
                } else if left_hit.is_intersecting {
                    left_hit
                } else {
                    right_hit
                }
            }
        }
    }
}
