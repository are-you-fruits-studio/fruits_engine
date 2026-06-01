use crate::{features::{inspector_window::{InspectedAssetEditedEvent, save_inspector_window_system}, project_window_parsing::InspectedAssetResource, project_window_selection::SelectedFileResource}, *};

pub fn register_feature(mut world: WorldBuilderMut) {
    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(save_inspected_asset_system);

    update.order_system(save_inspector_window_system)
        .before_system(save_inspected_asset_system)
        .before_system(select_file_system);
}

pub fn save_inspected_asset_system(
    evt_inspected_asset_edited: Evt<InspectedAssetEditedEvent>,
    inspected_asset: Res<InspectedAssetResource>,
    serializer: Res<SerializersResource>,
    mut selected_file: ResMut<SelectedFileResource>,
    render_api: Res<RenderApiResource>,
    mut prefabs: ResMut<AssetStorageResource<Prefab>>,
    mut textures: ResMut<AssetStorageResource<StandardTexture>>,
    mut materials: ResMut<AssetStorageResource<StandardMaterial>>,
    mut meshes: ResMut<AssetStorageResource<StandardMesh>>,
) {
    return_if!(evt_inspected_asset_edited.is_empty());

    let Some(inspected_asset) = &inspected_asset.data else {
        return;
    };

    let serialized = deserialize_with_assets(
        &*render_api,
        &*serializer,
        &mut *prefabs,
        &mut *textures,
        &mut *materials,
        &mut *meshes,
        |local_serializer, _| {
            inspected_asset.to_serialized(&serializer.to_ctx(Some(&local_serializer)))
        }
    );

    let json_str = serde_json::to_string_pretty(&serialized.to_json()).unwrap();

    let json_bytes = json_str.into_bytes();

    if let Err(err) = std::fs::write(&selected_file.path, &json_bytes) {
        eprintln!("failed to write to {:?}. {}", &selected_file.path, err);
        return;
    }

    selected_file.file_data = json_bytes;
}
