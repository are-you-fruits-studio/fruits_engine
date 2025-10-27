use std::ops::{Deref, DerefMut};

use fruits_ecs::Resource;

// todo: remove
// use crate::render_app_state::RenderAppState;

// // todo: support ffi
// #[derive(Resource)]
// pub struct RenderStateResource(RenderAppState);

// impl RenderStateResource {
//     pub fn new(state: RenderAppState) -> Self {
//         Self(state)
//     }
// }

// impl Deref for RenderStateResource {
//     type Target = RenderAppState;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl DerefMut for RenderStateResource {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }