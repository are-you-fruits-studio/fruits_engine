use crate::{features::{project_window_selection::{FileSelectedEvent, SelectedFileResource}, serialization::SerializerResource}, *};

pub fn register_feature(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(InspectedAssetResource::default())
        .ok()
        .unwrap();

    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(parse_selected_file_system);

    update.order_system(select_file_system)
        .before_system(parse_selected_file_system);
}

#[derive(Resource, Default)]
pub struct InspectedAssetResource {
    pub data: Option<InspectedAsset>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum InspectedAsset {
    Material(StandardMaterial),
}

impl InspectedAsset {
    pub(crate) fn to_asset_type(&self) -> &'static str {
        match self {
            InspectedAsset::Material { .. } => "material",
        }
    }

    pub(crate) fn to_serialized(&self, ctx: &SerializerCtx) -> SerializedValue {
        let mut serialized = match self {
            InspectedAsset::Material(material) => material.serialize(ctx),
        };

        if let SerializedValue::Composite(serialized_composite) = &mut serialized.result {
            if let SerializedCompositeValues::Map(serialized_map) = &mut serialized_composite.values {
                serialized_map.values.insert_before(0, String::from("asset_type"), SerializedValue::Primitive(SerializedPrimitive::String(String::from(self.to_asset_type()))));
            } else {
                eprintln!("Strange InspectedAsset serialization, failed to enrich with asset_type");
            }
        } else {
            eprintln!("Strange InspectedAsset serialization, failed to enrich with asset_type");
        }

        serialized.result
    }

    pub(crate) fn from_serialized(ctx: &SerializerCtx, serialized: &SerializedValue) -> Option<Self> {
        let SerializedValue::Composite(serialized_composite) = serialized else {
            return None;
        };

        let SerializedCompositeValues::Map(serialized_values) = &serialized_composite.values else {
            return None;
        };

        let mut asset_type = None;
        let mut reserialized_map = SerializedMap::default();
        reserialized_map.enum_metadata = serialized_values.enum_metadata.clone();

        for (key, serialized_value) in &serialized_values.values {
            if key.as_str() == "asset_type" {
                if let SerializedValue::Primitive(SerializedPrimitive::String(serialized_value)) = &serialized_value {
                    asset_type = Some(serialized_value.clone());
                }

                continue;
            }

            reserialized_map.values.insert(key.clone(), serialized_value.clone());
        }

        let Some(asset_type) = asset_type else {
            return None;
        };

        let serialized_asset = SerializedValue::Composite(SerializedComposite {
            is_rigid: serialized_composite.is_rigid,
            values: SerializedCompositeValues::Map(reserialized_map)
        });

        match asset_type.as_str() {
            "material" => StandardMaterial::deserialize(ctx, &serialized_asset).result.map(Self::Material),
            _ => None,
        }
    }
}

pub fn parse_selected_file_system(
    file_selected_evt: Evt<FileSelectedEvent>,
    selected_file: Res<SelectedFileResource>,
    serializer: Res<SerializerResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
) {
    return_if!(file_selected_evt.is_empty());

    inspected_asset.data = 'parsing: {
        let Ok(file_text) = String::from_utf8(selected_file.file_data.clone()) else {
            break 'parsing None;
        };

        let Ok(json) = serde_json::from_str::<serde_json::Value>(&file_text) else {
            break 'parsing None;
        };

        let serialized_value = SerializedValue::from_json(&json);
        InspectedAsset::from_serialized(&serializer.0.to_ctx(None), &serialized_value)
    };
}

//

//

fn material_to_json(material: &StandardMaterial) -> JsonObject {
    // todo
    JsonObject::new()
        .with_field("asset_type", String::from("material")).ok().unwrap()
        // .with_field("space", material.space)
        // .with_field("color", material.color)
        // .with_field("emission_color", material.emission_color)
        .with_field("metallic", material.metallic).ok().unwrap()
        .with_field("roughness", material.roughness).ok().unwrap()
        // .with_field("alpha_threshold", material.alpha_threshold)
        // .with_field("color_tex", material.color_tex)
        .with_field("is_lit", material.is_lit).ok().unwrap()
}

fn material_from_json(json: &JsonObject) -> StandardMaterial {
    // todo
    let mut material = StandardMaterial::default();

        // .with_field("space", material.space)
        // .with_field("color", material.color)
        // .with_field("emission_color", material.emission_color)
    if let Some(JsonValue::Number(metallic)) = json.get_value("metallic") {
        material.metallic = metallic.to_f() as f32;
    }
    if let Some(JsonValue::Number(roughness)) = json.get_value("roughness") {
        material.roughness = roughness.to_f() as f32;
    }
        // .with_field("alpha_threshold", material.alpha_threshold)
        // .with_field("color_tex", material.color_tex)
    if let Some(JsonValue::Bool(is_lit)) = json.get_value("is_lit") {
        material.is_lit = *is_lit;
    }

    material
}