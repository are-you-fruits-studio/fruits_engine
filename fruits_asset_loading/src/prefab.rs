use std::{collections::HashMap, marker::PhantomData, path::PathBuf};

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_ecs::*;
use fruits_ffi::FfiString;
use fruits_prefab::{Prefab, PrefabComponent, PrefabComponentsDeserializerResource};
use fruits_json::{JsonObject, JsonSerializable, JsonValue};

//

pub fn get_or_load_prefab_from_world(mut res: ResourcesHolderMut, key: &str) -> Option<AssetHandle<Prefab>> {
    let (prefabs, /*deserializer*/) = unsafe {
        (
            // &*res.get_ptr::<RenderApiResource>()?,
            // &mut *res.get_ptr::<AssetStorageResource<StandardMaterial>>()?,
            // &mut *res.get_ptr::<AssetStorageResource<StandardTexture>>()?,
            &mut *res.get_ptr::<AssetStorageResource<Prefab>>()?,
            // &*res.get_ptr::<PrefabComponentsDeserializerResource>()?,
        )
    };

    get_or_load_prefab(prefabs, /*deserializer, */key)
}

pub fn get_or_load_prefab(
    prefabs: &mut AssetStorageResource<Prefab>,
    // deserializer: &PrefabComponentsDeserializerResource,
    // todo
    // render_api: &RenderApiResource,
    // materials: &mut AssetStorageResource<StandardMaterial>,
    // textures: &mut AssetStorageResource<StandardTexture>,
    // meshes: &mut AssetStorageResource<StandardMesh>,
    key: &str,
) -> Option<AssetHandle<Prefab>> {
    if let Some(stored_prefab) = prefabs.get_registered(key) {
        if prefabs.get(stored_prefab).is_some() {
            return Some(stored_prefab.clone());
        }

        prefabs.unregister(key);
    }

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(key);

    // todo: make copying assets folder into the build directory a part of the build process
    let raw_prefab = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => return None,
    };

    let Some(prefab) = deserialize_prefab(&raw_prefab) else {
        return None;
    };

    let prefab_handle = prefabs.insert(prefab);

    prefabs.register(FfiString::from_string(key.to_string()), prefab_handle.clone());

    Some(prefab_handle)
}

//

fn deserialize_prefab(
    // todo
    // textures: &mut AssetStorageResource<StandardTexture>,
    // render_api: &RenderApiResource,
    data: &str,
) -> Option<Prefab> {
    let Ok(raw_prefab) = serde_json::from_str::<serde_json::Value>(data) else {
        return None;
    };

    let serde_json::Value::Object(raw_prefab) = raw_prefab else {
        return None;
    };

    let Some(serde_json::Value::String(asset_type)) = raw_prefab.get("asset_type") else {
        return None;
    };

    if asset_type != "prefab" {
        return None;
    }

    let Some(serde_json::Value::Array(prefab_entities)) = raw_prefab.get("entities") else {
        return None;
    };

    let mut prefab = Prefab::empty();

    for prefab_entity in prefab_entities {
        let serde_json::Value::Object(prefab_entity) = prefab_entity else {
            continue;
        };

        let Some(serde_json::Value::Number(prefab_entity_id)) = prefab_entity.get("entity_id") else {
            continue;
        };

        let Some(serde_json::Value::Array(prefab_components)) = prefab_entity.get("components") else {
            continue;
        };

        let mut components = Vec::new();

        for prefab_component in prefab_components {
            let serde_json::Value::Object(prefab_component) = prefab_component else {
                continue;
            };

            let Some(serde_json::Value::String(prefab_component_id)) = prefab_component.get("component_id") else {
                continue;
            };

            let Some(prefab_component_data) = prefab_component.get("data") else {
                continue;
            };

            components.push(PrefabComponent {
                component_id: prefab_component_id.clone(),
                data: prefab_component_data.clone(),
            });
        }

        prefab.entities.insert(prefab_entity_id.as_i128().unwrap() as usize, components);
    }

    Some(prefab)
}

//

#[derive(Default)]
struct PrefabComponentSpawnCtx {
    root_entity_id: usize,
    entities: HashMap<usize, Entity>,
}

pub fn instantiate_prefab(mut world: WorldDataMut, prefab: AssetHandle<Prefab>) -> Option<Entity> {
    let (res, mut ent, evt) = world.as_tuple_mut();

    let res = res.as_ref();

    let prefabs = res.get::<AssetStorageResource<Prefab>>()?;
    let deserializer = res.get::<PrefabComponentsDeserializerResource>().unwrap();

    let prefab = prefabs.get(&prefab)?;

    let mut ctx = PrefabComponentSpawnCtx::default();
    let mut is_root_init = false;

    for &entity_id in prefab.entities.keys() {
        let entity = ent.create_entity();

        ctx.entities.insert(entity_id, entity);

        if !std::mem::replace(&mut is_root_init, true) {
            ctx.root_entity_id = entity_id;
        }
    }

    for (&entity_id, prefab_components) in &prefab.entities {
        let entity = *ctx.entities.get(&entity_id).unwrap();

        for prefab_component in prefab_components {
            deserializer.deserialize(&prefab_component.component_id, prefab_component.data.clone(), entity, ent.as_mut(), res);
        }
    }

    ctx.entities.get(&ctx.root_entity_id).copied()
}

// todo: prefabs instantiation needs to:
// + spawn entities
// + spawn components on the entities
// - access to other entities inside the prefab
// - access to materials
// - access to meshes
// - access to textures
// - access to fonts
// - access to other prefabs
// - access to other assets

const EXAMPLE: &str = r#"
[
  {
    "entity_id": 0,
    "components": [
      {
        "component_id": GlobalTransform,
        "data": {
          
        }
      }
    ]
  }
]
"#;
