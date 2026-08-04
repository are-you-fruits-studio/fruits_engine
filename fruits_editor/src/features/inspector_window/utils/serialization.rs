use crate::{
    features::asset_serialization::{InspectedAsset, get_asset_type},
    *,
};

pub fn save_asset_from_world_res(res: ResourcesHolderRef, serializer: &GlobalSerializer, asset_key: &str) -> Option<SerializedValue> {
    let asset_type = get_asset_type(res, asset_key)?;

    // todo
    let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
    // let mut err_handler = |_| ();

    let inspected_asset = InspectedAsset {
        asset_key: asset_key.into(),
        asset_type,
    };

    save_with_asset_serializers_from_world(res, None, |local_serializer| {
        inspected_asset.to_serialized(serializer.to_ctx(Some(&local_serializer), &mut err_handler))
    })
}

pub fn load_asset_to_world_res(
    res: ResourcesHolderMut,
    asset_key: &str,
    value: &SerializedValue,
    asset_type: AssetType,
    assets_dir_path: &str,
) -> bool {
    load_asset_single_from_world(
        res,
        assets_dir_path,
        asset_key,
        None,
        |local_serializer, serializer| {
            let mut err_handler = |err| println!("{err}");
            let mut serializer_ctx = serializer.to_ctx(Some(&local_serializer), &mut err_handler);
            match asset_type {
                AssetType::Texture => serializer_ctx.deserialize::<DirectDeserializedAsset<StandardTexture>>(value).map(|_| ()),
                AssetType::Material => serializer_ctx.deserialize::<DirectDeserializedAsset<StandardMaterial>>(value).map(|_| ()),
                AssetType::Mesh => serializer_ctx.deserialize::<DirectDeserializedAsset<StandardMesh>>(value).map(|_| ()),
                AssetType::Font => serializer_ctx.deserialize::<DirectDeserializedAsset<Font>>(value).map(|_| ()),
                AssetType::AudioClip => serializer_ctx.deserialize::<DirectDeserializedAsset<AudioClip>>(value).map(|_| ()),
                AssetType::Prefab => serializer_ctx.deserialize::<DirectDeserializedAsset<Prefab>>(value).map(|_| ()),
            };
        },
    ).is_some()
}

pub fn enrich_serialized_with_asset_type(serialized: &mut SerializedValue, asset_type: AssetType) {
    if let SerializedValue::Composite(serialized) = serialized {
        if let SerializedCompositeValues::Map(serialized) = &mut serialized.values {
            let mut map = FfiIndexMap::new();

            map.insert(
                FfiString::from("asset_type"),
                SerializedValue::Primitive(SerializedPrimitive::String(asset_type.serialized_str().into())),
            );
            for (k, v) in &serialized.values {
                map.insert(k.clone(), v.clone());
            }

            serialized.values = map;
        }
    }
}

pub fn are_components_slices_similar(l: &[PrefabComponent], r: &[PrefabComponent]) -> bool {
    if l.len() != r.len() {
        return false;
    }

    for (l, r) in l.iter().zip(r.iter()) {
        if l.component_id != r.component_id {
            return false;
        }

        if !l.data.similar(&r.data) {
            return false;
        }
    }

    true
}