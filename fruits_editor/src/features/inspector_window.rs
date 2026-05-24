use fruits_engine::*;

use crate::{
    features::{
        input_field::InputFieldComponent,
        project_window_parsing::{InspectedAsset, InspectedAssetResource, parse_selected_file_system}, project_window_selection::FileSelectedEvent, serialization::SerializerResource,
    },
    *,
};

pub fn register_feature(mut world: WorldBuilderMut) {
    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(update_inspector_window_system)
        .insert_child_system(save_inspector_window_system);

    update.order_system(parse_selected_file_system)
        .before_system(save_inspector_window_system)
        .before_system(update_inspector_window_system);
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

#[derive(Event)]
pub struct InspectedAssetEditedEvent;

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

    let inspected_asset = res.get_mut::<InspectedAssetResource>().unwrap();
    let Some(stored_inspected_asset) = &inspected_asset.data else {
        return;
    };

    let mut serialized = parse_serialized(ent.as_ref(), content_ent);

    if let SerializedValue::Composite(serialized) = &mut serialized {
        if let SerializedCompositeValues::Map(serialized) = &mut serialized.values {
            serialized.values.insert_before(0, String::from("asset_type"), SerializedValue::Primitive(SerializedPrimitive::String(stored_inspected_asset.to_asset_type().to_string())));
        }
    }

    let serializer_ctx = res.get::<SerializerResource>().unwrap().0.to_ctx(None);
    let asset = InspectedAsset::from_serialized(&serializer_ctx, &serialized);

    let Some(asset) = asset else {
        return;
    };

    let inspected_asset = res.get_mut::<InspectedAssetResource>().unwrap();
    if inspected_asset.data.as_ref() == Some(&asset) {
        return;
    }

    inspected_asset.data = Some(asset);
    evt.get_mut().push(InspectedAssetEditedEvent);
}

pub fn update_inspector_window_system(
    mut world: ExclusiveWorldAccess,
) {
    let (mut res, mut ent, mut evt) = world.as_tuple_mut();

    return_if!(evt.get::<FileSelectedEvent>().is_empty());

    let inspected_asset = res.get::<InspectedAssetResource>().unwrap();
    let assets = res.get::<StandardAssetsResource>().unwrap().clone();
    let render_assets = res.get::<StandardRenderAssetsResource>().unwrap();
    let serializer = res.get::<SerializerResource>().unwrap();
    let container_q = ent.query::<&InspectorWindowComponent>().iter().copied().collect::<Vec<_>>();

    let font = render_assets.font_px_8_8.clone();

    for window_c in container_q {
        destroy_entity_children(ent.as_mut(), window_c.content_container);

        let asset_type_text = ent.get_component_mut::<TextComponent>(window_c.asset_type_text).unwrap();
        asset_type_text.text.clear();

        match &inspected_asset.data {
            None => {

            },
            Some(InspectedAsset::Material(material)) => {
                asset_type_text.text.push_str("asset type: material");

                let serialized = material.serialize(&serializer.0.to_ctx(None)).result;

                //

                let ent_serialized = spawn_serialized(
                    ent.as_mut(),
                    window_c.content_container,
                    &serialized,
                    assets.material_panel.clone(),
                    assets.material_text.clone(),
                    font.clone(),
                );

                // todo: remove
                // let parsed_json = parse_json(ent.as_ref(), ent_json);
                // dbg!(parsed_json);
            }
        }
    }
}

fn parse_serialized(
    ent: EntitiesHolderRef,
    ent_target: Entity,
) -> SerializedValue {
    let Some(serialized_value_component) = ent.get_component::<SerializedValueComponent>(ent_target) else {
        return SerializedValue::Null;
    };

    match serialized_value_component {
        SerializedValueComponent::Primitive { text, ty } => {
            let default_result = match ty {
                SerializedValuePrimitiveType::Null => SerializedValue::Null,
                SerializedValuePrimitiveType::Bool => SerializedValue::Primitive(SerializedPrimitive::Bool(false)),
                SerializedValuePrimitiveType::Int => SerializedValue::Primitive(SerializedPrimitive::Int(0)),
                SerializedValuePrimitiveType::Float => SerializedValue::Primitive(SerializedPrimitive::Float(0.0)),
                SerializedValuePrimitiveType::String => SerializedValue::Primitive(SerializedPrimitive::String(String::new())),
            };

            let Some(input_c) = ent.get_component::<InputFieldComponent>(*text) else {
                return default_result;
            };

            let Some(text_c) = ent.get_component::<TextComponent>(input_c.text) else {
                return default_result;
            };
            
            match ty {
                SerializedValuePrimitiveType::Null => SerializedValue::Null,
                SerializedValuePrimitiveType::Bool => SerializedValue::Primitive(SerializedPrimitive::Bool(text_c.text.as_str() == "true" || text_c.text.as_str() == "True")),
                SerializedValuePrimitiveType::Int => SerializedValue::Primitive(SerializedPrimitive::Int(text_c.text.as_str().parse().unwrap_or(0))),
                SerializedValuePrimitiveType::Float => SerializedValue::Primitive(SerializedPrimitive::Float(text_c.text.as_str().parse().unwrap_or(0.0))),
                SerializedValuePrimitiveType::String => SerializedValue::Primitive(SerializedPrimitive::String(text_c.text.as_str().to_string())),
            }
        },
        SerializedValueComponent::Container { container, ty, enum_metadata: enum_metadata_ent } => {
            let default_result = match ty {
                SerializedValueContainerType::List => SerializedValue::Composite(SerializedComposite { is_rigid: false, values: SerializedCompositeValues::List(Vec::new()) }),
                SerializedValueContainerType::Map => SerializedValue::Composite(SerializedComposite { is_rigid: false, values: SerializedCompositeValues::Map(SerializedMap::default()) }),
            };

            let Some(container_parent_c) = ent.get_component::<ParentComponent>(*container) else {
                return default_result;
            };

            let mut enum_metadata = None;

            if let Some(enum_metadata_c) = ent.get_component::<SerializedEnumMetadataComponent>(*enum_metadata_ent) {
                if let Some(input_c) = ent.get_component::<InputFieldComponent>(enum_metadata_c.value_text) {
                    if let Some(text_c) = ent.get_component::<TextComponent>(input_c.text) {
                        enum_metadata = Some(SerializedEnumMetadata {
                            variant: text_c.text.to_string(),
                            variants: enum_metadata_c.variants.clone(),
                        });
                    };
                };
            }

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

                let value = parse_serialized(ent, *value_ent);

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
                SerializedValueContainerType::List => SerializedValue::Composite(SerializedComposite {
                    is_rigid: false,
                    values: SerializedCompositeValues::List(values.into_iter().map(|(k, v)| v).collect()),
                }),
                SerializedValueContainerType::Map => SerializedValue::Composite(SerializedComposite {
                    is_rigid: false,
                    values: SerializedCompositeValues::Map({
                        let mut map = SerializedMap::default();

                        map.enum_metadata = enum_metadata;
    
                        for (key, value) in values {
                            map.values.insert(key, value);
                        }
                
                        map
                    })
                }),
            }
        },
    }
}

fn spawn_serialized(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    serialized: &SerializedValue,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) -> Entity {
    match serialized {
        SerializedValue::Null => todo!(),
        SerializedValue::Primitive(serialized) => {
            let (text, ty) = match serialized {
                SerializedPrimitive::Bool(serialized) => (serialized.to_string().into(), SerializedValuePrimitiveType::Bool),
                SerializedPrimitive::Int(serialized) => (serialized.to_string().into(), SerializedValuePrimitiveType::Int),
                SerializedPrimitive::Float(serialized) => (serialized.to_string().into(), SerializedValuePrimitiveType::Float),
                SerializedPrimitive::String(serialized) => (serialized.to_string().into(), SerializedValuePrimitiveType::String),
            };

            let ent_text = spawn_input_area_ent(ent.as_mut(), ent_parent, text, material_panel.clone(), material_text.clone(), font.clone());
            ent.add_component(ent_text, SerializedValueComponent::Primitive { text: ent_text, ty }).ok().unwrap();
            ent_text
        },
        SerializedValue::Composite(serialized) => match &serialized.values {
            SerializedCompositeValues::List(serialized_list) => {
                let ent_root = spawn_default_layout_ent(ent.as_mut(), ent_parent, false);
                
                ent.add_component(ent_root, SerializedValueComponent::Container {
                    container: ent_root,
                    ty: SerializedValueContainerType::List,
                    enum_metadata: Entity::EMPTY,
                }).ok().unwrap();

                for (i, serialized_value) in serialized_list.iter().enumerate() {
                    let ent_field = spawn_default_layout_ent(ent.as_mut(), ent_root, false);

                    let (ent_key, ent_key_text) = spawn_text_ent(ent.as_mut(), ent_field, i.to_string().into(), material_panel.clone(), material_text.clone(), font.clone());

                    let ent_value_container = spawn_default_layout_ent(ent.as_mut(), ent_field, true);

                    spawn_serialized(ent.as_mut(), ent_value_container, serialized_value, material_panel.clone(), material_text.clone(), font.clone());
                    
                    ent.add_component(ent_field, SerializedFieldComponent { key_text: ent_key_text, value_container: ent_value_container }).ok().unwrap();
                }

                ent_root
            },
            SerializedCompositeValues::Map(serialized_map) => {
                let ent_root = spawn_default_layout_ent(ent.as_mut(), ent_parent, false);
                
                let mut enum_metadata_ent = Entity::EMPTY;

                if let Some(enum_metadata) = &serialized_map.enum_metadata {
                    let ent_enum_field = spawn_default_layout_ent(ent.as_mut(), ent_root, false);

                    enum_metadata_ent = ent_enum_field;

                    let (ent_key_enum, ent_key_text_enum) = spawn_text_ent(ent.as_mut(), ent_enum_field, String::from("$enum_variant").into(), material_panel.clone(), material_text.clone(), font.clone());

                    let ent_value_container = spawn_default_layout_ent(ent.as_mut(), ent_enum_field, true);

                    let enum_metadata_input_ent = spawn_input_area_ent(ent.as_mut(), ent_value_container, enum_metadata.variant.clone().into(), material_panel.clone(), material_text.clone(), font.clone());

                    ent.add_component(ent_enum_field, SerializedEnumMetadataComponent { value_text: enum_metadata_input_ent, variants: enum_metadata.variants.clone() }).ok().unwrap();
                }

                ent.add_component(ent_root, SerializedValueComponent::Container {
                    container: ent_root,
                    ty: SerializedValueContainerType::Map,
                    enum_metadata: enum_metadata_ent
                }).ok().unwrap();

                for (field_name, field_value) in &serialized_map.values {
                    let ent_field = spawn_default_layout_ent(ent.as_mut(), ent_root, false);

                    let (ent_key, ent_key_text) = spawn_text_ent(ent.as_mut(), ent_field, field_name.clone().into(), material_panel.clone(), material_text.clone(), font.clone());

                    let ent_value_container = spawn_default_layout_ent(ent.as_mut(), ent_field, true);

                    spawn_serialized(ent.as_mut(), ent_value_container, field_value, material_panel.clone(), material_text.clone(), font.clone());
                    
                    ent.add_component(ent_field, SerializedFieldComponent { key_text: ent_key_text, value_container: ent_value_container }).ok().unwrap();
                }

                ent_root
            },
        },
    }
}

fn spawn_default_layout_ent(
    mut ent: EntitiesHolderMut,
    ent_parent: Entity,
    is_padded: bool,
) -> Entity {
    let entity = ent.create_entity();

    let mut local_rect = LocalRectComponent::default();
    local_rect.scale = Vec2::new(UiVal::pw(1.0).into(), None.into());
    if is_padded {
        local_rect.parent_padding_min = Vec2::new(UiVal::px(20.0), UiVal::px(0.0));
    }

    ent.add_component(entity, GlobalRectComponent::default()).ok().unwrap();
    ent.add_component(entity, local_rect).ok().unwrap();
    ent.add_component(entity, ChildComponent { parent: ent_parent }).ok().unwrap();
    ent.add_component(entity, ParentComponent { children: vec![].into() }).ok().unwrap();
    ent.add_component(entity, RectChildAlignComponent {
        direction: UiDirection::Vertical,
        spacing: UiSpacing::Chunk,
        anchor: Vec2::new(1.0, 0.0),
        min_gap: UiVal::px(0.0),
    }).ok().unwrap();

    if let Some(parent_c) = ent.get_component_mut::<ParentComponent>(ent_parent) {
        parent_c.children.push(entity);
    }

    entity
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
