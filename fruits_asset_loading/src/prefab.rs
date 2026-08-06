use std::{collections::{HashMap, VecDeque}, path::Path};

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_ecs::*;
use fruits_ffi::FfiVec;
use fruits_prefab::{Prefab, PrefabComponent, PrefabDependencies, PrefabEntities, deserialize_component, deserialize_prefab_components, serialize_components, serialize_prefab_single_entity};
use fruits_serialization::*;
use fruits_transform::ParentComponent;

use crate::AssetLoader;

pub struct PrefabHandleLoader<'a> {
    pub prefabs: &'a mut AssetStorageResource<Prefab>,
}

impl<'a> PrefabHandleLoader<'a> {
    pub fn from_world(res: ResourcesHolderMut<'a>) -> Option<Self> {
        Some(Self {
            prefabs: res.into_get_mut::<AssetStorageResource<Prefab>>()?,
        })
    }
}
impl<'a> AssetLoader for PrefabHandleLoader<'a> {
    type Asset = Prefab;
    type SelfWithAnotherLifetime<'r> = PrefabHandleLoader<'r>;

    fn create_loader<'r>(res: ResourcesHolderMut<'r>) -> Option<Self::SelfWithAnotherLifetime<'r>> {
        Self::SelfWithAnotherLifetime::from_world(res)
    }
    
    fn get_related_asset_storage(&mut self) -> &mut AssetStorageResource<Self::Asset> {
        self.prefabs
    }
    
    fn load_from_serialized(&mut self, ctx: SerializerCtx, value: &SerializedValue, _assets_dir_path: impl AsRef<Path>) -> Option<Self::Asset> {
        PrefabLoader.load_from_serialized(ctx, value)
    }
}

//

pub struct PrefabLoader;
impl PrefabLoader {
    pub fn load_from_serialized(&mut self, mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Prefab> {
        let prefab_entities = match deserialize_prefab_no_deps(ctx.as_mut(), value) {
            Some(prefab_entities) => prefab_entities,
            None => {
                ctx.report_err(SerializationError::InvalidInput { message: "invalid prefab structure".into() });
                PrefabEntities([(0, FfiVec::new())].into_iter().collect())
            }
        };
        
        let prefab = Prefab {
            dependencies: load_prefab_dependencies(ctx, &prefab_entities),
            entities: prefab_entities,
        };

        Some(prefab)
    }
}

#[derive(Serializable)]
struct TestSerializedPrefab {
    entities: FfiVec<SerializedPrefabEntity>,
}
#[derive(Serializable)]
struct SerializedPrefabEntity {
    entity_id: u64,
    components: FfiVec<PrefabComponent>,
}

pub fn deserialize_prefab_no_deps(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<PrefabEntities> {
    let serialized_prefab = TestSerializedPrefab::deserialize(ctx.as_pure(), value)?;

    Some(PrefabEntities(serialized_prefab.entities.into_iter().map(|e| {
        (
            e.entity_id,
            e.components,
        )
    }).collect()))
}

pub fn serialize_prefab_no_deps(mut ctx: SerializerCtx, value: &PrefabEntities) -> SerializedValue {
    let serialized_prefab = TestSerializedPrefab {
        entities: value.0.iter().map(|(&entity_id, components)| SerializedPrefabEntity {
            entity_id,
            components: components.clone(),
        }).collect(),
    };

    serialized_prefab.serialize(ctx.as_pure())
}



//

#[derive(Default)]
struct PrefabComponentSpawnCtx {
    root_entity_id: u64,
    entities: HashMap<u64, EntityId>,
}

fn load_prefab_dependencies(mut ctx: SerializerCtx, prefab: &PrefabEntities) -> PrefabDependencies {
    // todo
    let mut deps = PrefabDependencies::default();

    for prefab_components in prefab.0.values() {
        for prefab_component in prefab_components {
            _ = ctx.as_mut().deserialize_any(&prefab_component.component_id, &prefab_component.data);
        }
    }

    deps
}

pub fn instantiate_prefab(res: ResourcesHolderRef, mut ent: EntitiesHolderMut, prefab: AssetHandle<Prefab>) -> Option<EntityId> {
    let prefabs = res.get::<AssetStorageResource<Prefab>>()?;
    let serializers = res.get::<SerializersResource>().unwrap();

    let prefab = prefabs.get(&prefab)?;

    let mut ctx = PrefabComponentSpawnCtx::default();
    let mut is_root_init = false;

    for &entity_id in prefab.entities.0.keys() {
        let entity = ent.create_entity();

        ctx.entities.insert(entity_id, entity);

        if !std::mem::replace(&mut is_root_init, true) {
            ctx.root_entity_id = entity_id;
        }
    }

    let entities_serialized = HashMap::new();

    let deps = &prefab.dependencies;

    let mut serializer_local = SerializerRegistry::new();

    serializer_local.register(EntityTransSerializer::new(&ctx.entities, &entities_serialized));
    serializer_local.register(PrefabAssetInstantiateTransSerializer::new(deps, |deps, key| deps.textures.get(key).cloned()));
    serializer_local.register(PrefabAssetInstantiateTransSerializer::new(deps, |deps, key| deps.materials.get(key).cloned()));
    serializer_local.register(PrefabAssetInstantiateTransSerializer::new(deps, |deps, key| deps.meshes.get(key).cloned()));
    serializer_local.register(PrefabAssetInstantiateTransSerializer::new(deps, |deps, key| deps.audio_clips.get(key).cloned()));
    serializer_local.register(PrefabAssetInstantiateTransSerializer::new(deps, |deps, key| deps.fonts.get(key).cloned()));
    serializer_local.register(PrefabAssetInstantiateTransSerializer::new(deps, |deps, key| deps.prefabs.get(key).cloned()));

    let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
    let mut serializer_ctx = serializers.to_ctx(Some(&serializer_local), &mut err_handler);

    for (&entity_id, prefab_components) in &prefab.entities.0 {
        let entity = *ctx.entities.get(&entity_id).unwrap();

        for prefab_component in prefab_components {
            let did_deserialize_component = deserialize_component(
                &prefab_component.component_id,
                &prefab_component.data,
                entity,
                serializer_ctx.as_mut(),
                ent.as_mut(),
            );

            if !did_deserialize_component {
                println!("failed to deserialize component: {}", prefab_component.component_id);
            }
        }
    }

    ctx.entities.get(&ctx.root_entity_id).copied()
}

// todo: not full
pub fn record_into_prefab_components(
    res: ResourcesHolderRef,
    ent: EntitiesHolderRef,
    entity: EntityId,
    ent_to_id: &HashMap<EntityId, u64>,
) -> FfiVec<PrefabComponent> {
    let serializers = res.get::<SerializersResource>().unwrap();

    let entities_deserialized = HashMap::<u64, EntityId>::new();

    let mut local_serializers = SerializerRegistry::new();

    local_serializers.register(EntityTransSerializer {
        entities_deserialized: &entities_deserialized,
        entities_serialized: ent_to_id,
    });

    let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
    let serializer_ctx = serializers.0.to_ctx(Some(&local_serializers), &mut err_handler);

    serialize_components(entity, serializer_ctx, ent)
}

pub fn record_into_prefab(
    res: ResourcesHolderRef,
    ent: EntitiesHolderRef,
    entity: EntityId,
) -> Option<Prefab> {
    if !ent.contains_entity(entity) {
        return None;
    }

    let serializers = res.get::<SerializersResource>().unwrap();

    let entities_deserialized = HashMap::<u64, EntityId>::new();

    let mut ent_serialize_queue = VecDeque::new();
    ent_serialize_queue.push_back(entity);
    
    let mut ent_to_id = HashMap::new();
    let mut entities_ordered = Vec::new();

    while let Some(entity) = ent_serialize_queue.pop_front() {
        let id = entity.version_index().index + 1;
        if ent_to_id.insert(entity, id).is_some() {
            continue;
        }

        entities_ordered.push((entity, id));

        if let Some(parent_c) = ent.get_component::<ParentComponent>(entity) {
            ent_serialize_queue.extend(parent_c.children.iter().copied());
        }
    }

    let mut local_serializers = SerializerRegistry::new();
    local_serializers.register(EntityTransSerializer {
        entities_deserialized: &entities_deserialized,
        entities_serialized: &ent_to_id,
    });

    let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
    let mut serializer_ctx = serializers.0.to_ctx(Some(&local_serializers), &mut err_handler);

    let mut prefab = Prefab::empty();

    for (entity, id) in entities_ordered {
        // todo: asset references
        let components = serialize_components(entity, serializer_ctx.as_mut(), ent);

        prefab.entities.0.insert(id, components);
    }

    Some(prefab)
}

// todo
pub fn override_entity_components_from_prefab(
    res: ResourcesHolderRef,
    mut ent: EntitiesHolderMut,
    entity: EntityId,
    components: &[PrefabComponent],
    id_to_ent: &HashMap<u64, EntityId>,
) {
    let mut components_to_remove = Vec::new();
    ent.get_all_components(entity, |c| components_to_remove.push(c.type_info().short().name()));
    for component_name in components_to_remove {
        ent.remove_component_any(entity, component_name);
    }

    let serializers = res.get::<SerializersResource>().unwrap();

    let entities_serialized = HashMap::<EntityId, u64>::new();

    let mut local_serializers = SerializerRegistry::new();

    local_serializers.register(EntityTransSerializer {
        entities_deserialized: id_to_ent,
        entities_serialized: &entities_serialized,
    });

    let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
    let serializer_ctx = serializers.0.to_ctx(Some(&local_serializers), &mut err_handler);
    deserialize_prefab_components(components, entity, serializer_ctx, ent)
}

pub struct EntityTransSerializer<'brw> {
    entities_deserialized: &'brw HashMap<u64, EntityId>,
    entities_serialized: &'brw HashMap<EntityId, u64>,
}

impl<'brw> EntityTransSerializer<'brw> {
    pub fn new(
        entities_deserialized: &'brw HashMap<u64, EntityId>,
        entities_serialized: &'brw HashMap<EntityId, u64>,
    ) -> Self {
        Self {
            entities_deserialized,
            entities_serialized,
        }
    }
}

impl<'brw> TransSerializer for EntityTransSerializer<'brw> {
    type Deserialized = EntityId;

    fn serialize(&self, _ctx: SerializerCtx, value: &Self::Deserialized) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::Int(self.entities_serialized.get(value).copied().unwrap_or(0) as i128))
    }

    fn deserialize(&self, mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self::Deserialized> {
        let result = match value {
            SerializedValue::Primitive(SerializedPrimitive::Int(number)) => {
                match *number {
                    0 => Some(EntityId::EMPTY),
                    number => self.entities_deserialized.get(&(number as u64)).copied(),
                }
            },
            _ => None,
        };

        if result.is_none() {
            ctx.report_err(SerializationError::InvalidInput { message: "failed to parse entity".into() });
        }

        Some(result.unwrap_or_else(|| EntityId::EMPTY))
    }
}

pub struct PrefabAssetInstantiateTransSerializer<'a, T: 'static> {
    deps: &'a PrefabDependencies,
    extractor: fn(&PrefabDependencies, &str) -> Option<AssetHandle<T>>,
}
impl<'a, T: 'static> PrefabAssetInstantiateTransSerializer<'a, T> {
    pub fn new(deps: &'a PrefabDependencies, extractor: fn(&PrefabDependencies, &str) -> Option<AssetHandle<T>>) -> Self {
        Self {
            deps,
            extractor,
        }
    }
}
impl<'a, T> TransSerializer for PrefabAssetInstantiateTransSerializer<'a, T> {
    type Deserialized = AssetHandle<T>;

    fn serialize(&self, mut _ctx: SerializerCtx, _value: &Self::Deserialized) -> SerializedValue {
        unimplemented!("PrefabAssetInstantiateTransSerializer is for instantiation only");
    }

    fn deserialize(&self, mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self::Deserialized> {
        let SerializedValue::Primitive(SerializedPrimitive::String(value)) = value else {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle can only be deserialized from string".into() });
            return None;
        };

        let Some(asset_handle) = (self.extractor)(self.deps, value.as_str()) else {
            ctx.report_err(SerializationError::InvalidInput { message: format!("AssetHandle {value} cannot be loaded").into() });
            return None;
        };

        Some(asset_handle.clone())
    }
}

// todo: prefabs instantiation needs to:
// + spawn entities
// + spawn components on the entities
// + access to other entities inside the prefab
// + access to materials
// + access to meshes
// + access to textures
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
