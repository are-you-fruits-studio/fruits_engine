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