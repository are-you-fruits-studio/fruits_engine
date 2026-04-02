use fruits_engine::versioned::Versioned;
use fruits_engine::*;

use crate::{
    features::{
        input_field::InputFieldComponent,
        project_window_selection::{FileSelectedEvent, SelectedFileResource},
    },
    *,
};

pub fn register_feature(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(InspectedAssetResource::default())
        .ok()
        .unwrap();

    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .group(SYSTEM_GROUP)
        .insert_child_system(parse_selected_file_system);
    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .group(SYSTEM_GROUP)
        .insert_child_system(update_inspector_window_system);

    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .order_system(select_file_system)
        .before_system(parse_selected_file_system)
        .before_system(update_inspector_window_system);
}

#[derive(Resource, Default)]
pub struct InspectedAssetResource {
    data: Versioned<Option<InspectedAsset>>,
}

pub struct GenericSystemResource<T: 'static + Default>(T);

impl<T: Default> Default for GenericSystemResource<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: 'static + Default> SystemResource for GenericSystemResource<T> { }

#[derive(Component, Default, Copy, Clone)]
pub struct InspectorWindowComponent {
    pub asset_type_text: Entity,
    pub content_container : Entity,
}

pub struct JsonFieldComponent {
    name: Entity,
    value: Entity,
}

#[derive(Component, Default)]
pub struct JsonValueComponent {
    data: Entity,
    value_type: JsonValueComponentType,
}

#[derive(Default)]
pub enum JsonValueComponentType {
    #[default]
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

pub enum InspectedAsset {
    Material(StandardMaterial),
}

pub fn parse_selected_file_system(
    file_selected_evt: Evt<FileSelectedEvent>,
    selected_file: Res<SelectedFileResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
) {
    return_if!(file_selected_evt.is_empty());

    let parsed_result = 'parsing: {
        let Ok(file_text) = String::from_utf8(selected_file.file_data.clone()) else {
            break 'parsing None;
        };

        let Some(JsonValue::Object(json)) = JsonValue::parse(&mut file_text.chars()) else {
            break 'parsing None;
        };

        let Some(JsonValue::String(asset_type)) = json.get_value("asset_type") else {
            break 'parsing None;
        };

        let asset_type = asset_type.clone();

        Some((json, asset_type))
    };

    let Some((json, asset_type)) = parsed_result else {
        *inspected_asset.data = None;
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

            *inspected_asset.data = Some(InspectedAsset::Material(material));
        }
        _ => {
            *inspected_asset.data = None;
        }
    }
}

pub fn save_inspector_window_system(mut world: ExclusiveWorldAccess) {
    let (mut res, mut ent, mut evt) = world.as_tuple_mut();

    let Some(window_c) = ent.query::<&InspectorWindowComponent>().iter().next().copied() else {
        return;
    };
    
    let Some(parent_c) = ent.get_component::<ParentComponent>(window_c.content_container) else {
        return;
    };

    let Some(&content_ent) = parent_c.children.first() else {
        return;
    };

    let parsed_json = parse_json(ent.as_ref(), content_ent);
}

pub fn update_inspector_window_system(
    mut world: ExclusiveWorldAccess,
    mut inspected_asset_changed_l: Local<GenericSystemResource<Option<u64>>>,
) {
    let (mut res, mut ent, mut evt) = world.as_tuple_mut();

    let inspected_asset_resource_ver = res.get::<InspectedAssetResource>().map(|a| Versioned::version(&a.data));

    if std::mem::replace(&mut inspected_asset_changed_l.0, inspected_asset_resource_ver) == inspected_asset_resource_ver {
        return;
    }

    let inspected_asset = res.get::<InspectedAssetResource>().unwrap();
    let assets = res.get::<StandardAssetsResource>().unwrap().clone();
    let render_assets = res.get::<StandardRenderAssetsResource>().unwrap();
    let container_q = ent.query::<&InspectorWindowComponent>().iter().copied().collect::<Vec<_>>();

    let font = render_assets.font_px_8_8.clone();

    for window_c in container_q {
        destroy_entity_children(ent.as_mut(), window_c.content_container);

        let asset_type_text = ent.get_component_mut::<TextComponent>(window_c.asset_type_text).unwrap();
        asset_type_text.text.clear();

        match &*inspected_asset.data {
            None => {

            },
            Some(InspectedAsset::Material(material)) => {
                asset_type_text.text.push_str("asset type: material");

                let json = JsonValue::Object(
                    JsonObject::new()
                        .with_field("roughness", material.roughness)
                        .ok()
                        .unwrap()
                        .with_field("metallic", material.metallic)
                        .ok()
                        .unwrap()
                        .with_field("is_lit", material.is_lit)
                        .ok()
                        .unwrap()
                        .with_field("alpha_threshold", material.alpha_threshold.to_json())
                        .ok()
                        .unwrap()
                        .with_field("color", color_to_string(material.color).to_json())
                        .ok()
                        .unwrap()
                        .with_field("emission_color", color_to_string(material.emission_color).to_json())
                        .ok()
                        .unwrap()
                        .with_field("values", vec![
                            55,
                            42,
                            1,
                            -2,
                        ].to_json())
                        .ok()
                        .unwrap(),
                );

                //

                let ent_json = spawn_json(
                    ent.as_mut(),
                    window_c.content_container,
                    &json,
                    assets.material_panel.clone(),
                    assets.material_text.clone(),
                    font.clone(),
                );

                let parsed_json = parse_json(ent.as_ref(), ent_json);

                dbg!(parsed_json);
            }
        }
    }
}

fn parse_json(
    ent: EntitiesHolderRef,
    ent_target: Entity,
) -> JsonValue {
    let Some(serialized_value_component) = ent.get_component::<SerializedValueComponent>(ent_target) else {
        return JsonValue::Null;
    };

    match serialized_value_component {
        SerializedValueComponent::Primitive { text, ty } => {
            let default_result = match ty {
                SerializedValuePrimitiveType::Null => JsonValue::Null,
                SerializedValuePrimitiveType::Bool => JsonValue::Bool(false),
                SerializedValuePrimitiveType::Number => JsonValue::Number(JsonNumber::I(0)),
                SerializedValuePrimitiveType::String => JsonValue::String(String::new()),
            };

            let Some(input_c) = ent.get_component::<InputFieldComponent>(*text) else {
                return default_result;
            };

            let Some(text_c) = ent.get_component::<TextComponent>(input_c.text) else {
                return default_result;
            };
            
            match ty {
                SerializedValuePrimitiveType::Null => JsonValue::Null,
                SerializedValuePrimitiveType::Bool => JsonValue::Bool(text_c.text.as_str() == "true" || text_c.text.as_str() == "True"),
                SerializedValuePrimitiveType::Number => JsonValue::Number(JsonNumber::F(text_c.text.as_str().parse().unwrap_or(0.0))),
                SerializedValuePrimitiveType::String => JsonValue::String(text_c.text.as_str().to_string()),
            }
        },
        SerializedValueComponent::Container { container, ty } => {
            let default_result = match ty {
                SerializedValueContainerType::Array => JsonValue::Array(Vec::new()),
                SerializedValueContainerType::Object => JsonValue::Object(JsonObject::new()),
            };

            let Some(container_parent_c) = ent.get_component::<ParentComponent>(*container) else {
                return default_result;
            };

            let mut values = Vec::new();

            for child in &container_parent_c.children {
                let Some(field_c) = ent.get_component::<SerializedFieldComponent>(*child) else {
                    continue;
                };

                let Some(value_parent_c) = ent.get_component::<ParentComponent>(field_c.value_container) else {
                    continue;
                };

                let Some(value_ent) = value_parent_c.children.first() else {
                    continue;
                };

                let value = parse_json(ent, *value_ent);

                let key = match ent.get_component::<InputFieldComponent>(field_c.key_text) {
                    Some(input_c) => match ent.get_component::<TextComponent>(input_c.text) {
                        Some(key_text_c) => key_text_c.text.to_string(),
                        None => String::new(),
                    },
                    None => match ent.get_component::<TextComponent>(field_c.key_text) {
                        Some(key_text_c) => key_text_c.text.to_string(),
                        None => String::new(),
                    },
                };

                values.push((key, value));
            }
            
            match ty {
                SerializedValueContainerType::Array => JsonValue::Array(values.into_iter().map(|(k, v)| v).collect()),
                SerializedValueContainerType::Object => JsonValue::Object({
                    let mut obj = JsonObject::new();

                    for (key, value) in values {
                        obj.push_field(key, value).ok();
                    }

                    obj
                }),
            }
        },
    }
}

fn spawn_json(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    json: &JsonValue,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) -> Entity {
    match json {
        JsonValue::Null => todo!(),
        JsonValue::Bool(json_bool) => {
            let ent_text = spawn_input_area_ent(ent.as_mut(), ent_parent, json_bool.to_string().into(), material_panel.clone(), material_text.clone(), font.clone());

            ent.add_component(ent_text, SerializedValueComponent::Primitive { text: ent_text, ty: SerializedValuePrimitiveType::Bool }).ok().unwrap();

            ent_text
        },
        JsonValue::Number(json_number) => {
            let ent_text = spawn_input_area_ent(ent.as_mut(), ent_parent, json_number.to_string().into(), material_panel.clone(), material_text.clone(), font.clone());
            
            ent.add_component(ent_text, SerializedValueComponent::Primitive { text: ent_text, ty: SerializedValuePrimitiveType::Number }).ok().unwrap();

            ent_text
        },
        JsonValue::String(json_string) => {
            let ent_text = spawn_input_area_ent(ent.as_mut(), ent_parent, json_string.to_string().into(), material_panel.clone(), material_text.clone(), font.clone());
            
            ent.add_component(ent_text, SerializedValueComponent::Primitive { text: ent_text, ty: SerializedValuePrimitiveType::String }).ok().unwrap();

            ent_text
        },
        JsonValue::Array(json_values) => {
            let ent_root = ent.create_entity();
            
            ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
            ent.add_component(ent_root, LocalRectComponent::default()).ok().unwrap();
            ent.add_component(ent_root, ChildComponent { parent: ent_parent }).ok().unwrap();
            ent.add_component(ent_root, ParentComponent { children: vec![].into() }).ok().unwrap();
            ent.add_component(ent_root, RectChildAlignComponent {
                direction: UiDirection::Vertical,
                spacing: UiSpacing::Chunk,
                anchor: Vec2::new(0.0, 0.0),
                min_gap: UiVal::px(0.0),
            }).ok().unwrap();
            ent.add_component(ent_root, SerializedValueComponent::Container { container: ent_root, ty: SerializedValueContainerType::Array }).ok().unwrap();

            ent.get_component_mut::<ParentComponent>(ent_parent)
                .unwrap()
                .children
                .push(ent_root);

            for (i, json_value) in json_values.iter().enumerate() {
                let ent_field = ent.create_entity();

                ent.add_component(ent_field, GlobalRectComponent::default()).ok().unwrap();
                ent.add_component(ent_field, LocalRectComponent {
                    scale: Vec2::new(UiVal::pw(1.0).into(), None.into()),
                    ..Default::default()
                }).ok().unwrap();
                ent.add_component(ent_field, ChildComponent { parent: ent_root }).ok().unwrap();
                ent.add_component(ent_field, ParentComponent { children: vec![].into() }).ok().unwrap();
                ent.add_component(ent_field, RectChildAlignComponent {
                    direction: UiDirection::Vertical,
                    spacing: UiSpacing::Chunk,
                    anchor: Vec2::new(1.0, 0.0),
                    min_gap: UiVal::px(0.0),
                }).ok().unwrap();
                ent.get_component_mut::<ParentComponent>(ent_root)
                    .unwrap()
                    .children
                    .push(ent_field);

                let (ent_key, ent_key_text) = spawn_text_ent(ent.as_mut(), ent_field, i.to_string().into(), material_panel.clone(), material_text.clone(), font.clone());

                let ent_value_container = ent.create_entity();

                ent.add_component(ent_value_container, GlobalRectComponent::default()).ok().unwrap();
                ent.add_component(ent_value_container, LocalRectComponent {
                    scale: Vec2::new(UiVal::pw(1.0).into(), None.into()),
                    parent_padding_min: Vec2::new(UiVal::px(20.0), UiVal::px(0.0)),
                    ..Default::default()
                }).ok().unwrap();
                ent.add_component(ent_value_container, ChildComponent { parent: ent_field }).ok().unwrap();
                ent.add_component(ent_value_container, ParentComponent { children: vec![].into() }).ok().unwrap();
                ent.add_component(ent_value_container, RectChildAlignComponent {
                    direction: UiDirection::Vertical,
                    spacing: UiSpacing::Chunk,
                    anchor: Vec2::new(0.5, 0.5),
                    min_gap: UiVal::px(0.0),
                }).ok().unwrap();
                ent.get_component_mut::<ParentComponent>(ent_field)
                    .unwrap()
                    .children
                    .push(ent_value_container);

                spawn_json(ent.as_mut(), ent_value_container, json_value, material_panel.clone(), material_text.clone(), font.clone());
                
                ent.add_component(ent_field, SerializedFieldComponent { key_text: ent_key_text, value_container: ent_value_container }).ok().unwrap();
            }

            ent_root
        },
        JsonValue::Object(json_object) => {
            let ent_root = ent.create_entity();
            
            ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
            ent.add_component(ent_root, LocalRectComponent::default()).ok().unwrap();
            ent.add_component(ent_root, ChildComponent { parent: ent_parent }).ok().unwrap();
            ent.add_component(ent_root, ParentComponent { children: vec![].into() }).ok().unwrap();
            ent.add_component(ent_root, RectChildAlignComponent {
                direction: UiDirection::Vertical,
                spacing: UiSpacing::Chunk,
                anchor: Vec2::new(0.0, 0.0),
                min_gap: UiVal::px(0.0),
            }).ok().unwrap();
            ent.add_component(ent_root, SerializedValueComponent::Container { container: ent_root, ty: SerializedValueContainerType::Object }).ok().unwrap();

            ent.get_component_mut::<ParentComponent>(ent_parent)
                .unwrap()
                .children
                .push(ent_root);

            for field_name in json_object.field_names() {
                let field_value = json_object.get_value(field_name).unwrap();

                //
                
                let ent_field = ent.create_entity();

                ent.add_component(ent_field, GlobalRectComponent::default()).ok().unwrap();
                ent.add_component(ent_field, LocalRectComponent {
                    scale: Vec2::new(UiVal::pw(1.0).into(), None.into()),
                    ..Default::default()
                }).ok().unwrap();
                ent.add_component(ent_field, ChildComponent { parent: ent_root }).ok().unwrap();
                ent.add_component(ent_field, ParentComponent { children: vec![].into() }).ok().unwrap();
                ent.add_component(ent_field, RectChildAlignComponent {
                    direction: UiDirection::Vertical,
                    spacing: UiSpacing::Chunk,
                    anchor: Vec2::new(1.0, 0.0),
                    min_gap: UiVal::px(0.0),
                }).ok().unwrap();
                ent.get_component_mut::<ParentComponent>(ent_root)
                    .unwrap()
                    .children
                    .push(ent_field);

                let (ent_key, ent_key_text) = spawn_text_ent(ent.as_mut(), ent_field, field_name.clone().into(), material_panel.clone(), material_text.clone(), font.clone());

                let ent_value_container = ent.create_entity();

                ent.add_component(ent_value_container, GlobalRectComponent::default()).ok().unwrap();
                ent.add_component(ent_value_container, LocalRectComponent {
                    scale: Vec2::new(UiVal::pw(1.0).into(), None.into()),
                    parent_padding_min: Vec2::new(UiVal::px(20.0), UiVal::px(0.0)),
                    ..Default::default()
                }).ok().unwrap();
                ent.add_component(ent_value_container, ChildComponent { parent: ent_field }).ok().unwrap();
                ent.add_component(ent_value_container, ParentComponent { children: vec![].into() }).ok().unwrap();
                ent.add_component(ent_value_container, RectChildAlignComponent {
                    direction: UiDirection::Vertical,
                    spacing: UiSpacing::Chunk,
                    anchor: Vec2::new(0.5, 0.5),
                    min_gap: UiVal::px(0.0),
                }).ok().unwrap();
                ent.get_component_mut::<ParentComponent>(ent_field)
                    .unwrap()
                    .children
                    .push(ent_value_container);

                //

                spawn_json(ent.as_mut(), ent_value_container, field_value, material_panel.clone(), material_text.clone(), font.clone());
                
                ent.add_component(ent_field, SerializedFieldComponent { key_text: ent_key_text, value_container: ent_value_container }).ok().unwrap();
            }

            ent_root
        },
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
) -> Entity {
    let ent_root = ent.create_entity();

    let ent_name = ent.create_entity();
    let ent_value = ent.create_entity();

    ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(
        ent_root,
        LocalRectComponent {
            scale: Vec2::new(UiVal::pd(1.0).into(), None.into()),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_root, ChildComponent { parent: ent_parent })
        .ok()
        .unwrap();
    ent.add_component(
        ent_root,
        ParentComponent {
            children: vec![ent_name, ent_value].into(),
        },
    )
    .ok()
    .unwrap();
    ent.add_component(
        ent_root,
        RectChildAlignComponent {
            direction: UiDirection::Horizontal,
            spacing: UiSpacing::SpaceBetween,
            min_gap: UiVal::px(0.0),
            anchor: Vec2::new(0.5, 0.5),
        },
    )
    .ok()
    .unwrap();
    ent.get_component_mut::<ParentComponent>(ent_parent)
        .unwrap()
        .children
        .push(ent_root);

    ent.add_component(ent_name, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(
        ent_name,
        LocalRectComponent {
            scale: Vec2::new(UiVal::pd(0.5).into(), None.into()),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_name, ChildComponent { parent: ent_root }).ok().unwrap();
    ent.add_component(ent_name, ParentComponent { children: vec![].into() })
        .ok()
        .unwrap();
    ent.add_component(ent_name, RectChildAlignComponent::default()).ok().unwrap();

    ent.add_component(ent_value, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(
        ent_value,
        LocalRectComponent {
            scale: Vec2::new(UiVal::pd(0.5).into(), None.into()),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_value, ChildComponent { parent: ent_root })
        .ok()
        .unwrap();
    ent.add_component(ent_value, ParentComponent { children: vec![].into() })
        .ok()
        .unwrap();
    ent.add_component(ent_value, RectChildAlignComponent::default()).ok().unwrap();

    spawn_text_ent(
        ent.as_mut(),
        ent_name,
        text_name,
        material_panel.clone(),
        material_text.clone(),
        font.clone(),
    );
    spawn_input_area_ent(ent, ent_value, text_value, material_panel, material_text, font);

    ent_root
}

fn spawn_text_ent(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    text: FfiString,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) -> (Entity, Entity) {
    let ent_root = ent.create_entity();

    let ent_text = ent.create_entity();

    ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(
        ent_root,
        LocalRectComponent {
            scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(20.0).into()),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_root, ChildComponent { parent: ent_parent })
        .ok()
        .unwrap();
    ent.add_component(
        ent_root,
        StandardMaterialComponent {
            material: material_panel,
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_root, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(
        ent_root,
        ImageComponent {
            color: Vec4::from_array(rgba_f32_array!("#38383800")),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    // todo
    // ent.add_component(ent_root, JsonValueComponent {
    //     value_type: JsonValueComponentType::Null,
    //     data:
    // }).ok().unwrap();
    ent.get_component_mut::<ParentComponent>(ent_parent)
        .unwrap()
        .children
        .push(ent_root);

    ent.add_component(ent_text, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, LocalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, ChildComponent { parent: ent_root }).ok().unwrap();
    ent.add_component(ent_text, StandardMaterialComponent { material: material_text })
        .ok()
        .unwrap();
    ent.add_component(ent_text, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(
        ent_text,
        TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            font,
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text,
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        },
    )
    .ok()
    .unwrap();

    (ent_root, ent_text)
}

fn spawn_input_area_ent(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    text: FfiString,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) -> Entity {
    let ent_root = ent.create_entity();

    let ent_background = ent.create_entity();
    let ent_selection_border = ent.create_entity();
    let ent_text = ent.create_entity();

    ent.add_component(ent_root, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(
        ent_root,
        LocalRectComponent {
            scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(20.0).into()),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_root, ChildComponent { parent: ent_parent })
        .ok()
        .unwrap();
    ent.add_component(
        ent_root,
        InputFieldComponent {
            text: ent_text,
            selection_border: ent_selection_border,
        },
    )
    .ok()
    .unwrap();
    ent.add_component(
        ent_root,
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_root, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(ent_root, ButtonComponent).ok().unwrap();
    ent.add_component(
        ent_root,
        ImageComponent {
            color: Vec4::from_array(rgba_f32_array!("#a7a7a7ff")),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.get_component_mut::<ParentComponent>(ent_parent)
        .unwrap()
        .children
        .push(ent_root);

    ent.add_component(ent_selection_border, GlobalRectComponent::default())
        .ok()
        .unwrap();
    ent.add_component(
        ent_selection_border,
        LocalRectComponent {
            z: -0.5,
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_selection_border, GlobalDisableableComponent::default())
        .ok()
        .unwrap();
    ent.add_component(ent_selection_border, LocalDisableableComponent::default())
        .ok()
        .unwrap();
    ent.add_component(ent_selection_border, ChildComponent { parent: ent_root })
        .ok()
        .unwrap();
    ent.add_component(
        ent_selection_border,
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_selection_border, BatchedMeshComponent::default())
        .ok()
        .unwrap();
    ent.add_component(
        ent_selection_border,
        ImageComponent {
            color: Vec4::from_array(rgba_f32_array!("#5c66ebff")),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();

    ent.add_component(ent_background, GlobalRectComponent::default())
        .ok()
        .unwrap();
    ent.add_component(
        ent_background,
        LocalRectComponent {
            parent_padding_min: Vec2::new(UiVal::px(1.0), UiVal::px(1.0)),
            parent_padding_max: Vec2::new(UiVal::px(1.0), UiVal::px(1.0)),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_background, ChildComponent { parent: ent_root })
        .ok()
        .unwrap();
    ent.add_component(
        ent_background,
        StandardMaterialComponent {
            material: material_panel,
        },
    )
    .ok()
    .unwrap();
    ent.add_component(ent_background, BatchedMeshComponent::default())
        .ok()
        .unwrap();
    ent.add_component(
        ent_background,
        ImageComponent {
            color: Vec4::from_array(rgba_f32_array!("#272727ff")),
            ..Default::default()
        },
    )
    .ok()
    .unwrap();

    ent.add_component(ent_text, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, LocalRectComponent::default()).ok().unwrap();
    ent.add_component(ent_text, ChildComponent { parent: ent_background })
        .ok()
        .unwrap();
    ent.add_component(ent_text, StandardMaterialComponent { material: material_text })
        .ok()
        .unwrap();
    ent.add_component(ent_text, BatchedMeshComponent::default()).ok().unwrap();
    ent.add_component(
        ent_text,
        TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#ffffffff").unwrap()),
            font,
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text,
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        },
    )
    .ok()
    .unwrap();

    ent_root
}

fn color_to_string(color: Vec4<f32>) -> String {
    let mut text = String::new();

    text.push('#');
    for v in color.as_array() {
        let v = (v * 255.0).clamp(0.0, 255.0) as u8;
        text.push_str(&format!("{:02x}", v));
    }

    text
}
