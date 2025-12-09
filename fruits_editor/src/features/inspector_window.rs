use fruits_engine::prelude::utils::destroy_entity_children;

use crate::{features::{input_field::InputFieldComponent, project_window_selection::{FileSelectedEvent, SelectedFileResource}}, *};

pub fn register_feature(mut world: WorldBuilderMut) {
    world.data_mut().resources_mut().insert(InspectedAssetResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(parse_selected_file_system);
    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(update_inspector_window_system);
    
    world.behavior_mut().get_mut(Schedule::Update)
        .order_system(select_file_system)
        .before_system(parse_selected_file_system)
        .before_system(update_inspector_window_system);
}

#[derive(Resource, Default)]
pub struct InspectedAssetResource {
    data: Option<InspectedAsset>,
}

#[derive(Component, Default)]
pub struct InspectorWindowComponent;

#[derive(Event, Default)]
pub struct InspectedAssetChangedEvent;

pub enum InspectedAsset {
    Material(StandardMaterial),
}

pub fn parse_selected_file_system(
    file_selected_evt: Evt<FileSelectedEvent>,
    mut inspected_asset_changed_evt: EvtMut<InspectedAssetChangedEvent>,
    selected_file: Res<SelectedFileResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
) {
    if file_selected_evt.is_empty() {
        return;
    }

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

            if let Some(JsonValue::Number(roughness)) = json.get_value("roughness") {
                material.roughness = roughness.to_f() as f32;
            };

            inspected_asset.data = Some(InspectedAsset::Material(material));
            inspected_asset_changed_evt.push(InspectedAssetChangedEvent::default());
        },
        _ => {
            inspected_asset.data = None;
            inspected_asset_changed_evt.push(InspectedAssetChangedEvent::default());
        },
    }
}

pub fn update_inspector_window_system(
    mut world: ExclusiveWorldAccess,
) {
    let (mut res, mut ent, mut evt) = world.as_tuple_mut();

    if evt.get::<InspectedAssetChangedEvent>().is_empty() {
        return;
    }

    let inspected_asset = res.get::<InspectedAssetResource>().unwrap();
    let assets = res.get::<StandardAssetsResource>().unwrap().clone();
    let render_assets = res.get::<StandardRenderAssetsResource>().unwrap();
    let container_q = ent.query_filtered::<Entity, WithFilter<InspectorWindowComponent>>();
    
    let font = render_assets.font_px_8_8.clone();

    for window_ent in container_q.iter().collect::<Vec<_>>() {
        destroy_entity_children(ent.as_mut(), window_ent);

        match &inspected_asset.data {
            None => return,
            Some(InspectedAsset::Material(material)) => {
                spawn_text_ent(
                    ent.as_mut(),
                    window_ent,
                    String::from("asset type: material").into(),
                    assets.material_panel.clone(),
                    assets.material_text.clone(),
                    font.clone(),
                );
                spawn_text_field(
                    ent.as_mut(),
                    window_ent,
                    String::from("metallic").into(),
                    material.metallic.to_string().into(),
                    assets.material_panel.clone(),
                    assets.material_text.clone(),
                    font.clone(),
                );
                spawn_text_field(
                    ent.as_mut(),
                    window_ent,
                    String::from("roughness").into(),
                    material.roughness.to_string().into(),
                    assets.material_panel.clone(),
                    assets.material_text.clone(),
                    font.clone(),
                );
            },
        }
    }
}

fn spawn_text_field(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    text_name: FfiString,
    text_value: FfiString,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) {
    let ent_root = ent.create_entity();
    let ent_name = ent.create_entity();
    let ent_value = ent.create_entity();

    ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_root, LocalRectComponent {
        scale: Vec2::new(UiVal::pd(1.0).into(), None.into()),
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_root, ChildComponent { parent: ent_parent, }).ok().unwrap();
    ent.add_component(ent_root, ParentComponent { children: vec![ent_name, ent_value].into(), }).ok().unwrap();
    ent.add_component(ent_root, RectChildAlignComponent {
        direction: UiDirection::Horizontal,
        spacing: UiSpacing::SpaceBetween,
        min_gap: UiVal::px(0.0),
        anchor: Vec2::new(0.5, 0.5),
    }).ok().unwrap();
    ent.get_component_mut::<ParentComponent>(ent_parent).unwrap().children.push(ent_root);
    
    ent.add_component(ent_name, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_name, LocalRectComponent {
        scale: Vec2::new(UiVal::pd(0.5).into(), None.into()),
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_name, ChildComponent { parent: ent_root, }).ok().unwrap();
    ent.add_component(ent_name, ParentComponent { children: vec![].into() }).ok().unwrap();
    ent.add_component(ent_name, RectChildAlignComponent::default()).ok().unwrap();
    
    ent.add_component(ent_value, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_value, LocalRectComponent {
        scale: Vec2::new(UiVal::pd(0.5).into(), None.into()),
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_value, ChildComponent { parent: ent_root, }).ok().unwrap();
    ent.add_component(ent_value, ParentComponent { children: vec![].into() }).ok().unwrap();
    ent.add_component(ent_value, RectChildAlignComponent::default()).ok().unwrap();

    spawn_text_ent(ent.as_mut(), ent_name, text_name, material_panel.clone(), material_text.clone(), font.clone());
    spawn_input_area_ent(ent, ent_value, text_value, material_panel, material_text, font);
}

fn spawn_text_ent(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    text: FfiString,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) {
    let ent_root = ent.create_entity();
    let ent_text = ent.create_entity();

    ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_root, LocalRectComponent {
        scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(20.0).into()),
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_root, ChildComponent { parent: ent_parent, }).ok().unwrap();
    ent.add_component(ent_root, StandardMaterialComponent { material: material_panel }).ok().unwrap();
    ent.add_component(ent_root, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_root, ImageComponent {
        color: Vec4::from_array(rgba_f32_array!("#38383800")),
        ..Default::default()
    }).ok().unwrap();
    ent.get_component_mut::<ParentComponent>(ent_parent).unwrap().children.push(ent_root);

    ent.add_component(ent_text, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, LocalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, ChildComponent { parent: ent_root, }).ok().unwrap();
    ent.add_component(ent_text, StandardMaterialComponent { material: material_text }).ok().unwrap();
    ent.add_component(ent_text, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_text, TextComponent {
        color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
        font: font,
        font_size: UiVal::px(18.0),
        is_y_inverted: true,
        text,
        horizontal_spacing: UiVal::px(0.0),
        vertical_align: VerticalAlign::Middle,
        horizontal_align: HorizontalAlign::Left,
    }).ok().unwrap();
}

fn spawn_input_area_ent(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    text: FfiString,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) {
    let ent_root = ent.create_entity();
    let ent_background = ent.create_entity();
    let ent_selection_border = ent.create_entity();
    let ent_text = ent.create_entity();

    ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_root, LocalRectComponent {
        scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(20.0).into()),
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_root, ChildComponent { parent: ent_parent, }).ok().unwrap();
    ent.add_component(ent_root, InputFieldComponent {
        text: ent_text,
        selection_border: ent_selection_border,
    }).ok().unwrap();
    ent.add_component(ent_root, StandardMaterialComponent { material: material_panel.clone() }).ok().unwrap();
    ent.add_component(ent_root, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_root, ButtonComponent).ok().unwrap();
    ent.add_component(ent_root, ImageComponent {
        color: Vec4::from_array(rgba_f32_array!("#a7a7a7ff")),
        ..Default::default()
    }).ok().unwrap();
    ent.get_component_mut::<ParentComponent>(ent_parent).unwrap().children.push(ent_root);

    ent.add_component(ent_selection_border, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_selection_border, LocalRectComponent {
        z: 0.0,
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_selection_border, GlobalDisableableComponent::default()).ok().unwrap();
    ent.add_component(ent_selection_border, LocalDisableableComponent::default()).ok().unwrap();
    ent.add_component(ent_selection_border, ChildComponent { parent: ent_root, }).ok().unwrap();
    ent.add_component(ent_selection_border, StandardMaterialComponent { material: material_panel.clone() }).ok().unwrap();
    ent.add_component(ent_selection_border, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_selection_border, ImageComponent {
        color: Vec4::from_array(rgba_f32_array!("#5c66ebff")),
        ..Default::default()
    }).ok().unwrap();

    ent.add_component(ent_background, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_background, LocalRectComponent {
        parent_padding_min: Vec2::new(UiVal::px(1.0), UiVal::px(1.0)),
        parent_padding_max: Vec2::new(UiVal::px(1.0), UiVal::px(1.0)),
        ..Default::default()
    }).ok().unwrap();
    ent.add_component(ent_background, ChildComponent { parent: ent_root, }).ok().unwrap();
    ent.add_component(ent_background, StandardMaterialComponent { material: material_panel }).ok().unwrap();
    ent.add_component(ent_background, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_background, ImageComponent {
        color: Vec4::from_array(rgba_f32_array!("#272727ff")),
        ..Default::default()
    }).ok().unwrap();

    ent.add_component(ent_text, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, LocalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, ChildComponent { parent: ent_background, }).ok().unwrap();
    ent.add_component(ent_text, StandardMaterialComponent { material: material_text }).ok().unwrap();
    ent.add_component(ent_text, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_text, TextComponent {
        color: Vec4::from_array(parse_color_rgba_f32("#ffffffff").unwrap()),
        font: font,
        font_size: UiVal::px(18.0),
        is_y_inverted: true,
        text,
        horizontal_spacing: UiVal::px(0.0),
        vertical_align: VerticalAlign::Middle,
        horizontal_align: HorizontalAlign::Left,
    }).ok().unwrap();
}