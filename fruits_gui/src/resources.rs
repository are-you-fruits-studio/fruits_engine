use std::time::Instant;

use fruits_modules::{asset::AssetHandle, render::StandardMaterial};
use fruits_prelude::Resource;

#[derive(Resource)]
pub struct AssetsResource {
    pub material_text: AssetHandle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct RequestsResource {
    pub last_req_time: Option<Instant>,
}