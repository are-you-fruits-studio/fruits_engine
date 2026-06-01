use crate::{features::project_window_selection::{FileSelectedEvent, SelectedFileResource}, *};

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
    Material(SerializedMaterial),
    Texture(SerializedTexture),
    AudioClip(SerializedAudioClip),
    Mesh(SerializedMesh),
}

impl InspectedAsset {
    pub(crate) fn to_asset_type(&self) -> &'static str {
        match self {
            InspectedAsset::Material { .. } => "material",
            InspectedAsset::Texture { .. } => "texture",
            InspectedAsset::AudioClip { .. } => "audio_clip",
            InspectedAsset::Mesh { .. } => "mesh",
        }
    }

    pub(crate) fn to_serialized(&self, ctx: &SerializerCtx) -> SerializedValue {
        let mut serialized = match self {
            InspectedAsset::Material(asset) => asset.serialize(ctx),
            InspectedAsset::Texture(asset) => asset.serialize(ctx),
            InspectedAsset::AudioClip(asset) => asset.serialize(ctx),
            InspectedAsset::Mesh(asset) => asset.serialize(ctx),
        };

        if let SerializedValue::Composite(serialized_composite) = &mut serialized.result {
            if let SerializedCompositeValues::Map(serialized_map) = &mut serialized_composite.values {
                let mut map = FfiIndexMap::new();

                map.insert(FfiString::from("asset_type"), SerializedValue::Primitive(SerializedPrimitive::String(self.to_asset_type().into())));
                for (k, v) in &serialized_map.values {
                    map.insert(k.clone(), v.clone());
                }

                serialized_map.values = map;
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
            "material" => SerializedMaterial::deserialize(ctx, &serialized_asset).result.map(Self::Material),
            "texture" => SerializedTexture::deserialize(ctx, &serialized_asset).result.map(Self::Texture),
            "audio_clip" => SerializedAudioClip::deserialize(ctx, &serialized_asset).result.map(Self::AudioClip),
            "mesh" => SerializedMesh::deserialize(ctx, &serialized_asset).result.map(Self::Mesh),
            _ => None,
        }
    }
}

pub fn parse_selected_file_system(
    file_selected_evt: Evt<FileSelectedEvent>,
    selected_file: Res<SelectedFileResource>,
    serializer: Res<SerializersResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
    render_api: Res<RenderApiResource>,
    mut prefabs: ResMut<AssetStorageResource<Prefab>>,
    mut textures: ResMut<AssetStorageResource<StandardTexture>>,
    mut materials: ResMut<AssetStorageResource<StandardMaterial>>,
    mut meshes: ResMut<AssetStorageResource<StandardMesh>>,
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

        deserialize_with_assets(
            &*render_api,
            &*serializer,
            &mut *prefabs,
            &mut *textures,
            &mut *materials,
            &mut *meshes,
            |local_serializer, _| {
                InspectedAsset::from_serialized(&serializer.to_ctx(Some(&local_serializer)), &serialized_value)
            }
        )
    };
}

//

#[derive(Clone, Debug, PartialEq, PartialOrd, TransSerializable)]
pub(crate) struct SerializedMaterial {
    pub space: RenderSpace,
    pub color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: FfiOption<f32>,
    pub color_tex: FfiOption<FfiString>,
    pub is_lit: bool,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, TransSerializable)]
pub(crate) struct SerializedTexture {
    raw_texture: FfiString,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, TransSerializable)]
pub(crate) struct SerializedAudioClip {
    raw_audio: FfiString,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, TransSerializable)]
pub(crate) struct SerializedMesh {
    raw_mesh: FfiString,
    coordinate_space_type: CoordinateSpaceType,
    has_clockwise_winding: bool,
    has_inverted_u: bool,
    has_inverted_v: bool,
}