use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_ecs::*;
use fruits_ffi::FfiString;
use fruits_modules::SerializersResource;
use fruits_prefab::{Prefab, PrefabComponent, PrefabComponentsDeserializerResource};
use fruits_render::StandardMaterial;
use fruits_render_core::{RenderApiResource, StandardMesh, StandardTexture};
use fruits_serialization::{DeserializationError, SerializerCtx, TransSerializer, SerializationError, SerializerRegistry};

use crate::{get_or_load_material, get_or_load_mesh, get_or_load_texture};

pub fn get_or_load_prefab_from_world(res: ResourcesHolderMut, key: &str) -> Option<AssetHandle<Prefab>> {
    let (
        render_api,
        serializers,
        prefabs,
        textures,
        materials,
        meshes,
    ) = unsafe {
        (
            &*res.get_ptr::<RenderApiResource>().unwrap(),
            &*res.get_ptr::<SerializersResource>().unwrap(),
            &mut *res.get_ptr::<AssetStorageResource<Prefab>>().unwrap(),
            &mut *res.get_ptr::<AssetStorageResource<StandardTexture>>().unwrap(),
            &mut *res.get_ptr::<AssetStorageResource<StandardMaterial>>().unwrap(),
            &mut *res.get_ptr::<AssetStorageResource<StandardMesh>>().unwrap(),
        )
    };

    get_or_load_prefab(
        render_api,
        serializers,
        prefabs,
        textures,
        materials,
        meshes,
        key,
    )
}

pub fn get_or_load_prefab(
    render_api: &RenderApiResource,
    serializers: &SerializersResource,
    prefabs: &mut AssetStorageResource<Prefab>,
    textures: &mut AssetStorageResource<StandardTexture>,
    materials: &mut AssetStorageResource<StandardMaterial>,
    meshes: &mut AssetStorageResource<StandardMesh>,
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

    let raw_prefab = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_err) => return None,
    };

    let Some(prefab) = deserialize_prefab(&raw_prefab) else {
        return None;
    };

    load_prefab_dependencies(
        &prefab,
        render_api,
        serializers,
        prefabs,
        textures,
        materials,
        meshes,
    );

    let prefab_handle = prefabs.insert(prefab);

    prefabs.register(FfiString::from_string(key.to_string()), prefab_handle.clone());

    Some(prefab_handle)
}

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

fn load_prefab_dependencies(
    prefab: &Prefab,
    render_api: &RenderApiResource,
    serializers: &SerializersResource,
    prefabs: &mut AssetStorageResource<Prefab>,
    textures: &mut AssetStorageResource<StandardTexture>,
    materials: &mut AssetStorageResource<StandardMaterial>,
    meshes: &mut AssetStorageResource<StandardMesh>,
) {
    let (
        prefabs,
        textures,
        materials,
        meshes,
    ) = (
        Mutex::new(prefabs),
        Mutex::new(textures),
        Mutex::new(materials),
        Mutex::new(meshes),
    );

    let entities_deserialized = HashMap::new();
    let entities_serialized = HashMap::new();

    let mut serializer_local = SerializerRegistry::new();

    serializer_local.register(EntityTransSerializer::new(&entities_deserialized, &entities_serialized, true));
    serializer_local.register(TextureLoadTransSerializer(render_api, &textures));
    serializer_local.register(MaterialLoadTransSerializer(render_api, &materials, &textures));
    serializer_local.register(MeshLoadTransSerializer(render_api, &meshes));
    serializer_local.register(PrefabLoadTransSerializer(render_api, serializers, &prefabs, &textures, &materials, &meshes));

    let serializer_ctx = SerializerCtx::new(serializers.registry(), Some(&serializer_local));

    for prefab_components in prefab.entities.values() {
        for prefab_component in prefab_components {
            if let Err(err) = serializer_ctx.deserialize_any(&prefab_component.component_id, &prefab_component.data) {
                println!("failed to deserialize component: {}. {}", prefab_component.component_id, err);
            }
        }
    }
}

pub fn instantiate_prefab(mut world: WorldDataMut, prefab: AssetHandle<Prefab>) -> Option<Entity> {
    let (res, mut ent, _evt) = world.as_tuple_mut();

    let res = res.as_ref();

    let prefabs = res.get::<AssetStorageResource<Prefab>>()?;
    let textures = res.get::<AssetStorageResource<StandardTexture>>()?;
    let materials = res.get::<AssetStorageResource<StandardMaterial>>()?;
    let meshes = res.get::<AssetStorageResource<StandardMesh>>()?;
    let components_deserializer = res.get::<PrefabComponentsDeserializerResource>().unwrap();
    let serializers = res.get::<SerializersResource>().unwrap();

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

    let entities_serialized = HashMap::new();

    let mut serializer_local = SerializerRegistry::new();

    serializer_local.register(EntityTransSerializer::new(&ctx.entities, &entities_serialized, false));
    serializer_local.register(AssetGetTransSerializer::new(&textures));
    serializer_local.register(AssetGetTransSerializer::new(&materials));
    serializer_local.register(AssetGetTransSerializer::new(&meshes));

    let serializer_ctx = SerializerCtx::new(serializers.registry(), Some(&serializer_local));

    for (&entity_id, prefab_components) in &prefab.entities {
        let entity = *ctx.entities.get(&entity_id).unwrap();

        for prefab_component in prefab_components {
            let did_deserialize_component = components_deserializer.deserialize(
                &prefab_component.component_id,
                prefab_component.data.clone(),
                entity,
                &serializer_ctx,
                ent.as_mut(),
                res,
            );

            if !did_deserialize_component {
                println!("failed to deserialize component: {}", prefab_component.component_id);
            }
        }
    }

    ctx.entities.get(&ctx.root_entity_id).copied()
}

pub struct EntityTransSerializer<'brw> {
    entities_deserialized: &'brw HashMap<usize, Entity>,
    entities_serialized: &'brw HashMap<Entity, usize>,
    should_ignore_errors: bool,
}

impl<'brw> EntityTransSerializer<'brw> {
    pub fn new(
        entities_deserialized: &'brw HashMap<usize, Entity>,
        entities_serialized: &'brw HashMap<Entity, usize>,
        should_ignore_errors: bool,
    ) -> Self {
        Self {
            entities_deserialized,
            entities_serialized,
            should_ignore_errors,
        }
    }
}

impl<'brw> TransSerializer for EntityTransSerializer<'brw> {
    type Deserialized = Entity;

    fn serialize(&self, _ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        let Some(value) = self.entities_serialized.get(value) else {
            return Ok(serde_json::Value::Null);
        };

        Ok(serde_json::Value::Number((*value).into()))
    }

    fn deserialize(&self, _ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        let result = match value {
            serde_json::Value::Null => Some(Entity::EMPTY),
            serde_json::Value::Number(number) => number.as_u64().map(|n| self.entities_deserialized.get(&(n as usize))).flatten().copied(),
            _ => None,
        };

        if self.should_ignore_errors {
            Ok(result.unwrap_or_else(|| Entity::EMPTY))
        } else {
            result.ok_or_else(|| DeserializationError::InvalidInput)
        }
    }
}

pub struct AssetGetTransSerializer<'brw, T: 'static> {
    res: &'brw AssetStorageResource<T>,
}
impl<'brw, T: 'static> AssetGetTransSerializer<'brw, T> {
    pub fn new(res: &'brw AssetStorageResource<T>) -> Self {
        Self {
            res,
        }
    }
}
impl<'brw, T> TransSerializer for AssetGetTransSerializer<'brw, T> {
    type Deserialized = AssetHandle<T>;

    fn serialize(&self, _ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        todo!()
    }

    fn deserialize(&self, _ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        let serde_json::Value::String(value) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        self.res.get_registered(value).cloned().ok_or_else(|| DeserializationError::InvalidInput)
    }
}

pub struct TextureLoadTransSerializer<'m, 'brw: 'm>(
    &'m RenderApiResource,
    &'m Mutex<&'brw mut AssetStorageResource<StandardTexture>>,
);
impl<'m, 'brw: 'm> TransSerializer for TextureLoadTransSerializer<'m, 'brw> {
    type Deserialized = AssetHandle<StandardTexture>;

    fn serialize(&self, _ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        todo!()
    }

    fn deserialize(&self, _ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        let serde_json::Value::String(value) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        get_or_load_texture(
            self.0,
            &mut self.1.lock().unwrap(),
            value,
        ).ok_or_else(|| DeserializationError::InvalidInput)
    }
}

pub struct MaterialLoadTransSerializer<'m, 'brw: 'm>(
    &'m RenderApiResource,
    &'m Mutex<&'brw mut AssetStorageResource<StandardMaterial>>,
    &'m Mutex<&'brw mut AssetStorageResource<StandardTexture>>,
);
impl<'m, 'brw: 'm> TransSerializer for MaterialLoadTransSerializer<'m, 'brw> {
    type Deserialized = AssetHandle<StandardMaterial>;

    fn serialize(&self, _ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        todo!()
    }

    fn deserialize(&self, _ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        let serde_json::Value::String(value) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        get_or_load_material(
            self.0,
            &mut self.1.lock().unwrap(),
            &mut self.2.lock().unwrap(),
            value,
        ).ok_or_else(|| DeserializationError::InvalidInput)
    }
}

pub struct MeshLoadTransSerializer<'m, 'brw: 'm>(
    &'m RenderApiResource,
    &'m Mutex<&'brw mut AssetStorageResource<StandardMesh>>,
);
impl<'m, 'brw: 'm> TransSerializer for MeshLoadTransSerializer<'m, 'brw> {
    type Deserialized = AssetHandle<StandardMesh>;

    fn serialize(&self, _ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        todo!()
    }

    fn deserialize(&self, _ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        let serde_json::Value::String(value) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        get_or_load_mesh(
            self.0,
            &mut self.1.lock().unwrap(),
            value,
        ).ok_or_else(|| DeserializationError::InvalidInput)
    }
}

pub struct PrefabLoadTransSerializer<'m, 'brw: 'm>(
    &'m RenderApiResource,
    &'m SerializersResource,
    &'m Mutex<&'brw mut AssetStorageResource<Prefab>>,
    &'m Mutex<&'brw mut AssetStorageResource<StandardTexture>>,
    &'m Mutex<&'brw mut AssetStorageResource<StandardMaterial>>,
    &'m Mutex<&'brw mut AssetStorageResource<StandardMesh>>,
);
impl<'m, 'brw: 'm> TransSerializer for PrefabLoadTransSerializer<'m, 'brw> {
    type Deserialized = AssetHandle<Prefab>;

    fn serialize(&self, _ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        todo!()
    }

    fn deserialize(&self, _ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        let serde_json::Value::String(value) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        get_or_load_prefab(
            self.0,
            self.1,
            &mut self.2.lock().unwrap(),
            &mut self.3.lock().unwrap(),
            &mut self.4.lock().unwrap(),
            &mut self.5.lock().unwrap(),
            value,
        ).ok_or_else(|| DeserializationError::InvalidInput)
    }
}

// todo: prefabs instantiation needs to:
// + spawn entities
// + spawn components on the entities
// + access to other entities inside the prefab
// - access to materials
// - access to meshes
// - access to textures
// - access to fonts
// - access to other prefabs
// - access to other assets

const _EXAMPLE: &str = r#"
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
