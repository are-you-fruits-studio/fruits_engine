use fruits_engine::*;

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
