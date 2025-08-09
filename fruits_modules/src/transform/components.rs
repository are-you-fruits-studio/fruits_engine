use fruits_ecs::{Component, Entity};
use fruits_math::{Mat, Mat3, Quat, Vec2, Vec3};

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct GlobalTransform {
    pub position: Vec3<f32>,
    pub scale_rotation: Mat3<f32>,
}
impl GlobalTransform {
    pub const IDENTITY: GlobalTransform = GlobalTransform {
        position: Vec3::with_all(0.0),
        scale_rotation: Mat::IDENTITY,
    };
}
impl Default for GlobalTransform {
    fn default() -> Self { GlobalTransform::IDENTITY }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct LocalTransform {
    pub position: Vec3<f32>,
    pub rotation: Quat<f32>,
    pub scale: Vec3<f32>,
}
impl LocalTransform {
    pub const IDENTITY: Self = Self {
        position: Vec3::with_all(0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::with_all(1.0),
    };
}
impl Default for LocalTransform {
    fn default() -> Self { LocalTransform::IDENTITY }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct ParentComponent {
    pub children: Vec<Entity>,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ChildComponent {
    pub parent: Entity,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Default)]
pub struct GlobalRectComponent {
    pub center: Vec2<f32>,
    pub scale: Vec2<f32>,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct LocalRectComponent {
    pub anchor_min: Vec2<f32>,
    pub anchor_max: Vec2<f32>,
    pub offset_min: Vec2<UiVal>,
    pub offset_max: Vec2<UiVal>,
}
impl Default for LocalRectComponent {
    fn default() -> Self {
        Self {
            anchor_min: Vec2::with_all(0.0),
            anchor_max: Vec2::with_all(1.0),
            offset_min: Vec2::with_all(UiVal::Px(0.0)),
            offset_max: Vec2::with_all(UiVal::Px(0.0)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UiVal {
    Px(f32),
    Pw(f32),
    Ph(f32),
    Pmin(f32),
    Pmax(f32),
    Vw(f32),
    Vh(f32),
    Vmin(f32),
    Vmax(f32),
}
impl UiVal {
    pub fn into_px(self, parent_size: Vec2<f32>, view_size: Vec2<f32>) -> f32 {
        match self {
            UiVal::Px(v) => v,
            UiVal::Pw(v) => parent_size.x * v,
            UiVal::Ph(v) => parent_size.y * v,
            UiVal::Pmin(v) => f32::min(parent_size.x, parent_size.y) * v,
            UiVal::Pmax(v) => f32::max(parent_size.x, parent_size.y) * v,
            UiVal::Vw(v) => view_size.x * v,
            UiVal::Vh(v) => view_size.y * v,
            UiVal::Vmin(v) => f32::min(view_size.x, view_size.y) * v,
            UiVal::Vmax(v) => f32::max(view_size.x, view_size.y) * v,
        }
    }
}