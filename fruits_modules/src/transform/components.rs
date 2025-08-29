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
    pub parent_padding_min: Vec2<UiVal>,
    pub parent_padding_max: Vec2<UiVal>,
    pub offset: Vec2<UiVal>,
    pub anchor: Vec2<f32>,
    pub pivot: Vec2<f32>,
    // todo: handle none (= fit children)
    pub scale: Vec2<Option<UiVal>>,
    pub z: f32,
    // todo
    pub children_align: Option<UiDirection>,
}
impl Default for LocalRectComponent {
    fn default() -> Self {
        Self {
            parent_padding_min: Vec2::with_all(UiVal::Px(0.0)),
            parent_padding_max: Vec2::with_all(UiVal::Px(0.0)),
            offset: Vec2::with_all(UiVal::Px(0.0)),
            anchor: Vec2::with_all(0.5),
            pivot: Vec2::with_all(0.5),
            scale: Vec2::new(Some(UiVal::Pw(1.0)), Some(UiVal::Ph(1.0))),
            z: -1.0,
            children_align: None,
        }
    }
}
impl LocalRectComponent {
    pub fn calculate_global_rect(
        local_rect: &LocalRectComponent,
        parent_rect: &GlobalRectComponent,
        window_size: Vec2<f32>,
        child_based_scale: Vec2<f32>,
    ) -> GlobalRectComponent {
        let parent_min = parent_rect.center - parent_rect.scale * 0.5;
        let parent_max = parent_rect.center + parent_rect.scale * 0.5;

        let ui_val_to_px = |v: UiVal| -> f32 {
            v.into_px(parent_rect.scale, window_size)
        };

        let parent_min = parent_min + local_rect.parent_padding_min.map(ui_val_to_px);
        let parent_max = parent_max - local_rect.parent_padding_max.map(ui_val_to_px);

        let padded_parent_scale = parent_max - parent_min;

        let ui_val_to_px = |v: UiVal| -> f32 {
            v.into_px(padded_parent_scale, window_size)
        };

        let final_scale = local_rect.scale.zip(child_based_scale, |x, c| x.map(ui_val_to_px).unwrap_or(*c));

        let anchored_pos = parent_min.lerp_separately(parent_max, local_rect.anchor);
        let offset_pos = anchored_pos + local_rect.offset.map(ui_val_to_px);
        let pivoted_center = offset_pos + final_scale * (Vec2::with_all(0.5) - local_rect.pivot);

        GlobalRectComponent {
            center: pivoted_center,
            scale: final_scale,
            z: parent_rect.z + local_rect.z,
        }
    }

    pub fn calculate_scale_hierarchy_independent(
        local_rect: &LocalRectComponent,
        window_size: Vec2<f32>,
    ) -> Vec2<f32> {
        let ui_val_to_px = |v: UiVal| -> f32 {
            v.into_px_without_parent(window_size).unwrap_or(0.0)
        };

        let scale = local_rect.scale.map(|x| x.map(ui_val_to_px).unwrap_or(0.0));

        scale
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

    pub fn into_px_without_parent(self, view_size: Vec2<f32>) -> Option<f32> {
        Some(match self {
            UiVal::Px(v) => v,
            UiVal::Vw(v) => view_size.x * v,
            UiVal::Vh(v) => view_size.y * v,
            UiVal::Vmin(v) => f32::min(view_size.x, view_size.y) * v,
            UiVal::Vmax(v) => f32::max(view_size.x, view_size.y) * v,
            _ => return None,
        })
    }
}