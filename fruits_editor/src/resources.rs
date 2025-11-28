use std::ffi::OsString;

use fruits_engine::prelude::*;

#[derive(Resource, Clone, Default)]
pub struct UiInteractionResource {
    pub selection: Entity,
    pub drag: Entity,
}

#[derive(Resource, Clone)]
pub struct StandardAssetsResource {
    pub material_panel: AssetHandle<StandardMaterial>,
    pub material_text: AssetHandle<StandardMaterial>,
}

#[derive(Resource, Clone, Default)]
pub struct SelectedFileResource {
    pub path: OsString,
    pub file_data: Vec<u8>,
}

#[derive(Resource, Clone, Default)]
pub struct InspectedFileResource {
    pub path: OsString,
}
