use std::fmt::Debug;

use fruits_math::{Mat4, Quat, Vec3};

use crate::LineBoundType;

/// A geometric primitive used for collision tests.
///
/// Every variant carries its concrete shape data. Use [`overlaps`](crate::overlaps) to test two shapes for
/// intersection, [`to_aabb`](Self::to_aabb) to obtain an enclosing box, and
/// [`apply_matrix_lossy`](Self::apply_matrix_lossy) to transform a shape into another space.
///
/// # Examples
///
/// ```
/// use fruits_collision::{CollisionShape, CollisionSphere, overlaps};
/// use fruits_math::Vec3;
///
/// let sphere = CollisionSphere { center: Vec3::splat(0.0), radius: 1.0 };
/// let point: CollisionShape = Vec3::new(0.5, 0.0, 0.0).into();
///
/// assert!(overlaps(point, sphere.into_shape()));
/// ```
#[repr(C, u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CollisionShape {
    /// A single point in space.
    Point(Vec3<f32>),
    /// A line, ray, or segment, depending on its [`LineBoundType`].
    Line(CollisionLine),
    /// An axis-aligned bounding box.
    Aabb(CollisionAabb),
    /// An oriented (rotatable) box.
    Box(CollisionBox),
    /// A sphere.
    Sphere(CollisionSphere),
    /// A triangle given by its three corner points.
    Triangle([Vec3<f32>; 3]),
}

impl From<Vec3<f32>> for CollisionShape {
    /// Wraps a point as [`CollisionShape::Point`].
    fn from(value: Vec3<f32>) -> Self {
        Self::Point(value)
    }
}
impl From<CollisionLine> for CollisionShape {
    /// Wraps a line as [`CollisionShape::Line`].
    fn from(value: CollisionLine) -> Self {
        Self::Line(value)
    }
}
impl From<CollisionAabb> for CollisionShape {
    /// Wraps an AABB as [`CollisionShape::Aabb`].
    fn from(value: CollisionAabb) -> Self {
        Self::Aabb(value)
    }
}
impl From<CollisionBox> for CollisionShape {
    /// Wraps an oriented box as [`CollisionShape::Box`].
    fn from(value: CollisionBox) -> Self {
        Self::Box(value)
    }
}
impl From<CollisionSphere> for CollisionShape {
    /// Wraps a sphere as [`CollisionShape::Sphere`].
    fn from(value: CollisionSphere) -> Self {
        Self::Sphere(value)
    }
}
impl From<[Vec3<f32>; 3]> for CollisionShape {
    /// Wraps three corner points as [`CollisionShape::Triangle`].
    fn from(value: [Vec3<f32>; 3]) -> Self {
        Self::Triangle(value)
    }
}

impl CollisionLine {
    /// Wraps this line as a [`CollisionShape::Line`] in a `const` context.
    pub const fn into_shape(self) -> CollisionShape {
        CollisionShape::Line(self)
    }
}
impl CollisionAabb {
    /// Wraps this AABB as a [`CollisionShape::Aabb`] in a `const` context.
    pub const fn into_shape(self) -> CollisionShape {
        CollisionShape::Aabb(self)
    }
}
impl CollisionBox {
    /// Wraps this box as a [`CollisionShape::Box`] in a `const` context.
    pub const fn into_shape(self) -> CollisionShape {
        CollisionShape::Box(self)
    }
}
impl CollisionSphere {
    /// Wraps this sphere as a [`CollisionShape::Sphere`] in a `const` context.
    pub const fn into_shape(self) -> CollisionShape {
        CollisionShape::Sphere(self)
    }
}

impl CollisionShape {
    /// Returns the smallest axis-aligned box enclosing this shape.
    ///
    /// # Panics
    ///
    /// Panics for a [`CollisionShape::Line`] that is not a finite
    /// [`SEGMENT`](LineBoundType::SEGMENT), since an unbounded line has no finite AABB.
    pub fn to_aabb(&self) -> CollisionAabb {
        match self {
            CollisionShape::Point(collision_point) => CollisionAabb {
                center: *collision_point,
                extents: Vec3::splat(0.0),
            },
            CollisionShape::Line(collision_line) => {
                if collision_line.bounds != LineBoundType::SEGMENT {
                    panic!("Trying to create infinite AABB.");
                }

                CollisionAabb::from_points([collision_line.start, collision_line.end].into_iter())
            }
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

                CollisionAabb::from_points(
                    points
                        .iter()
                        .map(|p| collision_box.center + collision_box.rotation.to_matrix() * *p),
                )
            }
            CollisionShape::Sphere(collision_sphere) => CollisionAabb {
                center: collision_sphere.center,
                extents: Vec3::splat(collision_sphere.radius),
            },
            CollisionShape::Triangle(collision_triangle) => CollisionAabb::from_points(collision_triangle.into_iter().copied()),
        }
    }

    /// Returns this shape transformed by `mat`.
    ///
    /// The transform is *lossy*: non-uniform scale and shear are approximated. A sphere's
    /// radius is scaled by the average of the matrix's lossy scale, and an [`Aabb`](CollisionShape::Aabb)
    /// keeps its extents (only its center is moved) rather than being re-fitted.
    pub fn apply_matrix_lossy(&self, mat: Mat4<f32>) -> Self {
        // todo: check
        match self {
            CollisionShape::Point(collision_point) => CollisionShape::Point(mat.mul_with_projection(*collision_point)),
            CollisionShape::Line(collision_line) => CollisionShape::Line(CollisionLine {
                start: mat.mul_with_projection(collision_line.start),
                end: mat.mul_with_projection(collision_line.end),
                bounds: collision_line.bounds,
            }),
            CollisionShape::Aabb(collision_aabb) => CollisionShape::Aabb(CollisionAabb {
                center: mat.mul_with_projection(collision_aabb.center),
                extents: collision_aabb.extents,
            }),
            CollisionShape::Box(collision_box) => {
                let (lossy_scale, rotation) = mat.ignored(3, 3).to_lossy_scale_rotation();

                CollisionShape::Box(CollisionBox {
                    center: mat.mul_with_projection(collision_box.center),
                    extents: lossy_scale * collision_box.extents,
                    rotation: rotation * collision_box.rotation,
                })
            }
            CollisionShape::Sphere(collision_sphere) => {
                let lossy_scale = mat.ignored(3, 3).to_lossy_scale();

                let avg_scale = lossy_scale.into_array().iter().sum::<f32>() / 3.0;

                CollisionShape::Sphere(CollisionSphere {
                    center: mat.mul_with_projection(collision_sphere.center),
                    radius: avg_scale * collision_sphere.radius,
                })
            }
            CollisionShape::Triangle(collision_triangle) => {
                CollisionShape::Triangle(collision_triangle.map(|p| mat.mul_with_projection(p)))
            }
        }
    }
}

/// An oriented box defined by a center, half-extents, and a rotation.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionBox {
    /// Center of the box in the current space.
    pub center: Vec3<f32>,
    /// Half-sizes along the box's own local axes before rotation.
    pub extents: Vec3<f32>,
    /// Orientation applied to the local axes.
    pub rotation: Quat<f32>,
}

/// An axis-aligned bounding box stored as a center and half-extents.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionAabb {
    /// Center of the box.
    pub center: Vec3<f32>,
    /// Half-sizes along the X, Y, and Z axes.
    pub extents: Vec3<f32>,
}
impl CollisionAabb {
    /// Builds an AABB spanning the inclusive range `[min, max]`.
    pub fn from_min_max(min: Vec3<f32>, max: Vec3<f32>) -> Self {
        Self {
            center: (min + max) * 0.5,
            extents: (max - min) * 0.5,
        }
    }

    /// Returns the minimum corner (`center - extents`).
    pub fn min(&self) -> Vec3<f32> {
        self.center - self.extents
    }

    /// Returns the maximum corner (`center + extents`).
    pub fn max(&self) -> Vec3<f32> {
        self.center + self.extents
    }

    /// Returns the smallest AABB enclosing all `points`.
    ///
    /// Returns a zero-sized box centered at the origin when the iterator is empty.
    pub fn from_points(mut points: impl Iterator<Item = Vec3<f32>>) -> Self {
        let Some(first) = points.next() else {
            return Self {
                center: Vec3::splat(0.0),
                extents: Vec3::splat(0.0),
            };
        };

        let mut min = first;
        let mut max = first;

        for point in points {
            min = min.zip_copied(point, f32::min);
            max = max.zip_copied(point, f32::max);
        }

        Self::from_min_max(min, max)
    }

    /// Returns the smallest AABB enclosing both `self` and `other`.
    pub fn merge(self, other: Self) -> Self {
        let min = self.min().zip_copied(other.min(), f32::min);
        let max = self.max().zip_copied(other.max(), f32::max);

        Self::from_min_max(min, max)
    }
}

/// A line through two points whose finiteness is controlled by [`LineBoundType`].
///
/// Depending on `bounds`, the same `start`/`end` pair represents an infinite line, a ray,
/// or a finite segment.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionLine {
    /// First defining point.
    pub start: Vec3<f32>,
    /// Second defining point.
    pub end: Vec3<f32>,
    /// Which ends of the line are bounded.
    pub bounds: LineBoundType,
}

/// A sphere defined by a center and radius.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionSphere {
    /// Center of the sphere.
    pub center: Vec3<f32>,
    /// Radius of the sphere.
    pub radius: f32,
}
