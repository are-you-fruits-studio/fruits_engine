use fruits_engine::prelude::utils::destroy_entity_children;

use crate::{features::project_window_selection::SelectedFileResource, *};

pub fn register_feature(mut world: WorldBuilderMut) {
    world.data_mut().resources_mut().insert(InspectedAssetResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(parse_selected_file_system);
    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(update_inspector_window_system);
    
    world.behavior_mut().get_mut(Schedule::Update)
        .order_system(parse_selected_file_system)
        .before_system(update_inspector_window_system);
}

#[derive(Resource, Default)]
pub struct InspectedAssetResource {
    data: Option<InspectedAsset>,
}

#[derive(Component, Default)]
pub struct InspectorWindowComponent;

pub enum InspectedAsset {
    Material(StandardMaterial),
}

pub fn parse_selected_file_system(
    selected_file: Res<SelectedFileResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
) {
    let Ok(file_text) = String::from_utf8(selected_file.file_data.clone()) else {
        return;
    };
    
    let Some(JsonValue::Object(json)) = JsonValue::parse(&mut file_text.chars()) else {
        return;
    };
    
    let Some(JsonValue::String(asset_type)) = json.get_value("asset_type") else {
        return;
    };
    
    match asset_type.as_str() {
        "material" => {
            let mut material = StandardMaterial::default();

            if let Some(JsonValue::Number(metallic)) = json.get_value("metallic") {
                material.metallic = metallic.to_f() as f32;
            };

            inspected_asset.data = Some(InspectedAsset::Material(material));
        },
        _ => return,
    }
}

pub fn update_inspector_window_system(
    mut world: ExclusiveWorldAccess,
) {
    let (mut res, mut ent, mut evt) = world.as_tuple_mut();

    let inspected_asset = res.get::<InspectedAssetResource>().unwrap();
    let container_q = ent.query_filtered::<(&mut ParentComponent, Entity), WithFilter<InspectorWindowComponent>>();

    // todo
    // for (window_container, window_ent) in container_q.iter() {
    //     destroy_entity_children(ent.as_mut(), window_ent);
    // }

    match &inspected_asset.data {
        None => return,
        Some(InspectedAsset::Material(material)) => {
        },
    }
}