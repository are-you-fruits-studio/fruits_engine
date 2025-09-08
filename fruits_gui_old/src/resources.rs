use std::time::Instant;

use fruits_engine::prelude::*;

#[derive(Resource)]
pub struct AssetsResource {
    pub material_text: AssetHandle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct RequestsResource {
    pub last_req_time: Option<Instant>,
}