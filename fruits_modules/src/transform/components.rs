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
    pub z: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UiDirection {
    Vertical,
    Horizontal,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct LocalRectComponent {
    pub anchor_min: Vec2<f32>,
    pub anchor_max: Vec2<f32>,
    pub offset_min: Vec2<UiVal>,
    pub offset_max: Vec2<UiVal>,
    pub pivot: Vec2<f32>,
    pub offset: Vec2<UiVal>,
    // todo: handle none (= fit children)
    pub scale: Vec2<Option<UiVal>>,
    pub z: f32,
    // todo
    pub children_align: Option<UiDirection>,
    // todo: ignore children_align field of the parent
    pub ignore_parent_align: bool,
}
impl Default for LocalRectComponent {
    fn default() -> Self {
        Self {
            anchor_min: Vec2::with_all(0.0),
            anchor_max: Vec2::with_all(1.0),
            offset_min: Vec2::with_all(UiVal::Px(0.0)),
            offset_max: Vec2::with_all(UiVal::Px(0.0)),
            pivot: Vec2::with_all(0.5),
            offset: Vec2::with_all(UiVal::Px(0.0)),
            scale: Vec2::with_all(Some(UiVal::Px(0.0))),
            z: -1.0,
            children_align: None,
            ignore_parent_align: false,
        }
    }
}
impl LocalRectComponent {
    pub fn calculate_global_rect(
        local_rect: &LocalRectComponent,
        parent_rect: &GlobalRectComponent,
        window_size: Vec2<f32>,
    ) -> GlobalRectComponent {
        let parent_min = parent_rect.center - parent_rect.scale * 0.5;
        let parent_max = parent_rect.center + parent_rect.scale * 0.5;

        let ui_val_to_px = |v: UiVal| -> f32 {
            v.into_px(parent_rect.scale, window_size)
        };

        let anchored_min = parent_min.lerp_separately(parent_max, local_rect.anchor_min);
        let anchored_max = parent_min.lerp_separately(parent_max, local_rect.anchor_max);

        let mut min = anchored_min + local_rect.offset_min.map(ui_val_to_px);
        let mut max = anchored_max + local_rect.offset_max.map(ui_val_to_px);

        // todo: calculate scale from children
        let scaling = local_rect.scale.map(|x| x.map(ui_val_to_px).unwrap_or(0.0));
        let pivot = local_rect.pivot.map(|x| x.clamp(0.0, 1.0));

        min += local_rect.offset.map(ui_val_to_px) + -scaling * pivot;
        max += local_rect.offset.map(ui_val_to_px) + scaling * (Vec2::with_all(1.0) - pivot);

        GlobalRectComponent {
            center: (max + min) * 0.5,
            scale: max - min,
            z: parent_rect.z + local_rect.z,
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