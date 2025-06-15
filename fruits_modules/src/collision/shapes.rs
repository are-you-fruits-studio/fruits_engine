use fruits_math::{Quat, Vec3};

use crate::collision::LineBoundType;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CollisionShape {
    Point(Vec3<f32>),
    Line(CollisionLine),
    Aabb(CollisionAabb),
    Box(CollisionBox),
    Sphere(CollisionSphere),
    Triangle([Vec3<f32>; 3]),
}

impl From<Vec3<f32>> for CollisionShape { fn from(value: Vec3<f32>) -> Self { Self::Point(value) } }
impl From<CollisionLine> for CollisionShape { fn from(value: CollisionLine) -> Self { Self::Line(value) } }
impl From<CollisionAabb> for CollisionShape { fn from(value: CollisionAabb) -> Self { Self::Aabb(value) } }
impl From<CollisionBox> for CollisionShape { fn from(value: CollisionBox) -> Self { Self::Box(value) } }
impl From<CollisionSphere> for CollisionShape { fn from(value: CollisionSphere) -> Self { Self::Sphere(value) } }
impl From<[Vec3<f32>; 3]> for CollisionShape { fn from(value: [Vec3<f32>; 3]) -> Self { Self::Triangle(value) } }

impl CollisionShape {
    pub fn to_aab(&self) -> CollisionAabb {
        match self {
            CollisionShape::Point(collision_point) => CollisionAabb {
                center: *collision_point,
                extents: Vec3::with_all(0.0),
            },
            CollisionShape::Line(collision_line) => {
                if collision_line.bounds != LineBoundType::SEGMENT {
                    panic!("Trying to create infinite AABB.");
                }

                CollisionAabb::from_points([collision_line.start, collision_line.end].into_iter())
            },
            CollisionShape::Aabb(collision_aabb) => *collision_aabb,
            CollisionShape::Box(collision_box) => {
                let ext = collision_box.extents;

                let points = [
                    ext * Vec3::new(-1.0, -1.0, -1.0),
                    ext * Vec3::new(-1.0, -1.0, 1.0),
                    ext * Vec3::new(-1.0, 1.0, -1.0),
                    ext * Vec3::new(-1.0, 1.0, 1.0),
                    ext * Vec3::new(1.0, -1.0, -1.0),
                    ext * Vec3::new(1.0, -1.0, 1.0),
                    ext * Vec3::new(1.0, 1.0, -1.0),
                    ext * Vec3::new(1.0, 1.0, 1.0),
                ];

                CollisionAabb::from_points(points.iter().map(|p| collision_box.center + collision_box.rotation.to_matrix() * *p))
            },
            CollisionShape::Sphere(collision_sphere) => CollisionAabb {
                center: collision_sphere.center,
                extents: Vec3::with_all(collision_sphere.radius),
            },
            CollisionShape::Triangle(collision_triangle) => CollisionAabb::from_points(collision_triangle.into_iter().copied()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionBox {
    pub center: Vec3<f32>,
    pub extents: Vec3<f32>,
    pub rotation: Quat<f32>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionAabb {
    pub center: Vec3<f32>,
    pub extents: Vec3<f32>,
}
impl CollisionAabb {
    pub fn from_min_max(min: Vec3<f32>, max: Vec3<f32>) -> Self {
        Self {
            center: (min + max) * 0.5,
            extents: (max - min) * 0.5,
        }
    }

    pub fn min(&self) -> Vec3<f32> {
        self.center - self.extents
    }

    pub fn max(&self) -> Vec3<f32> {
        self.center + self.extents
    }

    pub fn from_points(mut points: impl Iterator<Item = Vec3<f32>>) -> Self {
        let Some(first) = points.next() else {
            return Self {
                center: Vec3::with_all(0.0),
                extents: Vec3::with_all(0.0),
            };
        };

        let mut min = first;
        let mut max = first;

        for point in points {
            min = min.zip(point, f32::min);
            max = max.zip(point, f32::max);
        }

        Self::from_min_max(min, max)
    }

    pub fn merge(self, other: Self) -> Self {
        let min = self.min().zip(other.min(), f32::min);
        let max = self.max().zip(other.max(), f32::max);

        Self::from_min_max(min, max)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionLine {
    pub start: Vec3<f32>,
    pub end: Vec3<f32>,
    pub bounds: LineBoundType,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionSphere {
    pub center: Vec3<f32>,
    pub radius: f32,
}