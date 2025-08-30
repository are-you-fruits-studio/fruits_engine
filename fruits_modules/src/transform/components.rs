use fruits_ecs::{Component, Entity};
use fruits_math::{Mat, Mat3, Quat, Vec2, Vec3};

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct GlobalTransform {
    pub position: Vec3<f32>,
    pub scale_rotation: Mat3<f32>,
}
impl GlobalTransform {
    pub const IDENTITY: GlobalTransform = GlobalTransform {
        position: Vec3::splat(0.0),
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
        position: Vec3::splat(0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::splat(1.0),
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
    pub z: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UiDirection {
    Vertical,
    Horizontal,
}

impl UiDirection {
    pub const fn to_axis_idx(self) -> usize {
        match self {
            UiDirection::Horizontal => 0,
            UiDirection::Vertical => 1,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct LocalRectComponent {
    pub parent_padding_min: Vec2<UiVal>,
    pub parent_padding_max: Vec2<UiVal>,
    pub offset: Vec2<UiVal>,
    pub anchor: Vec2<f32>,
    pub pivot: Vec2<f32>,
    pub scale: Vec2<Option<UiVal>>,
    pub z: f32,
}
impl Default for LocalRectComponent {
    fn default() -> Self {
        Self {
            parent_padding_min: Vec2::splat(UiVal::Px(0.0)),
            parent_padding_max: Vec2::splat(UiVal::Px(0.0)),
            offset: Vec2::splat(UiVal::Px(0.0)),
            anchor: Vec2::splat(0.5),
            pivot: Vec2::splat(0.5),
            scale: Vec2::new(Some(UiVal::Pw(1.0)), Some(UiVal::Ph(1.0))),
            z: -1.0,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct RectChildAlignComponent {
    pub anchor: Vec2<f32>,
    pub direction: UiDirection,
    // todo: to UiVal
    pub min_gap: f32,
    pub spacing: UiSpacing,
}

impl Default for RectChildAlignComponent {
    fn default() -> Self {
        Self {
            anchor: Vec2::splat(0.5),
            direction: UiDirection::Vertical,
            min_gap: 0.0,
            spacing: UiSpacing::SpaceBetween,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UiSpacing {
    Chunk,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UiVal {
    Px(f32),
    Pw(f32),
    Ph(f32),
    Pd(f32),
    Pmin(f32),
    Pmax(f32),
    Vw(f32),
    Vh(f32),
    Vd(f32),
    Vmin(f32),
    Vmax(f32),
}
impl UiVal {
    pub fn into_px(self, parent_size: Vec2<f32>, view_size: Vec2<f32>) -> Vec2<f32> {
        match self {
            UiVal::Px(v) => Vec2::splat(v),
            UiVal::Pw(v) => Vec2::splat(parent_size.x * v),
            UiVal::Ph(v) => Vec2::splat(parent_size.y * v),
            UiVal::Pd(v) => parent_size * v,
            UiVal::Pmin(v) => Vec2::splat(f32::min(parent_size.x, parent_size.y) * v),
            UiVal::Pmax(v) => Vec2::splat(f32::max(parent_size.x, parent_size.y) * v),
            UiVal::Vw(v) => Vec2::splat(view_size.x * v),
            UiVal::Vh(v) => Vec2::splat(view_size.y * v),
            UiVal::Vd(v) => view_size * v,
            UiVal::Vmin(v) => Vec2::splat(f32::min(view_size.x, view_size.y) * v),
            UiVal::Vmax(v) => Vec2::splat(f32::max(view_size.x, view_size.y) * v),
        }
    }

    pub fn into_px_without_parent(self, view_size: Vec2<f32>) -> Option<Vec2<f32>> {
        Some(match self {
            UiVal::Px(v) => Vec2::splat(v),
            UiVal::Vw(v) => Vec2::splat(view_size.x * v),
            UiVal::Vh(v) => Vec2::splat(view_size.y * v),
            UiVal::Vd(v) => view_size * v,
            UiVal::Vmin(v) => Vec2::splat(f32::min(view_size.x, view_size.y) * v),
            UiVal::Vmax(v) => Vec2::splat(f32::max(view_size.x, view_size.y) * v),
            _ => return None,
        })
    }
}