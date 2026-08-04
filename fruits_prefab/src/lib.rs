//! # fruits_prefab
//!
//! Defines the engine's in-memory prefab format — a reusable template of entities and
//! their serialized components — together with the registry that turns those serialized
//! components back into live ECS components when a prefab is instantiated.
//!
//! # How to use
//!
//! #### Opting a component into prefabs
//!
//! Register a component type so prefab instantiation can reconstruct it. Until a type is
//! registered, prefab entries naming it are skipped. Components are keyed by their full
//! Rust type name ([`std::any::type_name`]), so the same string must appear as the
//! `component_id` in the prefab data.
//!
//! ```ignore
//! use fruits_prefab::PrefabComponentsDeserializerResource;
//!
//! // `deserializers` is the world's `PrefabComponentsDeserializerResource`.
//! fn register_my_components(deserializers: &mut PrefabComponentsDeserializerResource) {
//!     deserializers.register::<MyComponent>();
//! }
//! ```
//!
//! #### Building a prefab in memory
//!
//! Assemble a [`Prefab`] directly — useful for tools and tests that produce prefabs
//! without reading a file. Each entry maps a prefab-local entity id to the list of
//! components that entity carries.
//!
//! ```
//! use fruits_prefab::{Prefab, PrefabComponent};
//! use fruits_serialization::SerializedValue;
//!
//! let mut prefab = Prefab::empty();
//! prefab.entities.insert(0, vec![
//!     PrefabComponent {
//!         component_id: "my_crate::MyComponent".to_string(),
//!         data: SerializedValue::Null,
//!     },
//! ]);
//!
//! assert_eq!(prefab.entities.len(), 1);
//! ```
//!
//! # How to maintain
//!
//! #### Data model
//!
//! A [`Prefab`] is a `HashMap` from a prefab-local entity id (`usize`) to a `Vec` of
//! [`PrefabComponent`], each pairing a `component_id` string with the component's
//! [`SerializedValue`] payload. The ids are local
//! to the prefab: instantiation creates one real [`Entity`] per id
//! and resolves cross-references (an `Entity`-typed field pointing at another entity in
//! the same prefab) by mapping those local ids to the freshly created entities.
//!
//! #### The component id is the type name
//!
//! [`register`](PrefabComponentsDeserializerResource::register) inserts a deserializer
//! under `std::any::type_name::<C>()`, and
//! [`deserialize`](PrefabComponentsDeserializerResource::deserialize) looks one up by the
//! `component_id` carried on a [`PrefabComponent`]. For a prefab entry to resolve, its
//! `component_id` must equal the component's full type name exactly; an id with no
//! matching registration makes `deserialize` return `false` and the component is dropped.
//!
//! #### Type erasure
//!
//! Each registered type gets a [`PrefabComponentDeserializer<C>`], which implements the
//! private `AbstractPrefabComponentDeserializer` trait so deserializers for unrelated
//! component types can share one `HashMap` of boxed trait objects. On a hit, `deserialize`
//! runs the serialized payload through the supplied
//! [`SerializerCtx`] to produce a `C`, then calls
//! `add_component` on the entity; it returns `false` if either the deserialization or the
//! component insertion fails. The [`ResourcesHolderRef`]
//! parameter is threaded through but currently unused by the concrete implementation.
//!
//! #### Where the rest of the pipeline lives
//!
//! This crate only owns the format and the registry. Reading a prefab from JSON on disk
//! and instantiating one into the world live in `fruits_asset_loading`
//! (`deserialize_prefab`, `get_or_load_prefab_from_world`, `instantiate_prefab`), and the
//! [`PrefabComponentsDeserializerResource`] is inserted into the world by
//! `fruits_modules::add_defult_modules_to`. No engine code calls
//! [`register`](PrefabComponentsDeserializerResource::register) yet, so an app must
//! register its own component types before prefab components of those types can be
//! instantiated. A `// todo: ffi` on the resource marks FFI exposure as still pending.

use fruits_asset_storage::AssetHandle;
use fruits_audio::AudioClip;
use fruits_ecs::*;
use fruits_ffi::{FfiIndexMap, FfiString, FfiVec};
use fruits_render::{Font, StandardMaterial};
use fruits_render_core::{StandardMesh, StandardTexture};
use fruits_serialization::*;

#[repr(C)]
#[derive(Debug, Clone, Serializable)]
pub struct PrefabComponent {
    pub component_id: FfiString,
    pub data: SerializedValue,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct PrefabEntities(pub FfiIndexMap<u64, FfiVec<PrefabComponent>>);

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct PrefabDependencies {
    pub textures: FfiIndexMap<FfiString, AssetHandle<StandardTexture>>,
    pub meshes: FfiIndexMap<FfiString, AssetHandle<StandardMesh>>,
    pub materials: FfiIndexMap<FfiString, AssetHandle<StandardMaterial>>,
    pub audio_clips: FfiIndexMap<FfiString, AssetHandle<AudioClip>>,
    pub fonts: FfiIndexMap<FfiString, AssetHandle<Font>>,
    pub prefabs: FfiIndexMap<FfiString, AssetHandle<Prefab>>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Prefab {
    pub entities: PrefabEntities,
    pub dependencies: PrefabDependencies,
}

impl Prefab {
    pub fn empty() -> Self {
        Self {
            entities: Default::default(),
            dependencies: Default::default(),
        }
    }
}

// todo
pub fn serialize_prefab_single_entity(
    entity: EntityId,
    serializer_ctx: SerializerCtx,
    entities: EntitiesHolderRef,
) -> Prefab {
    Prefab {
        entities: PrefabEntities([(0, serialize_components(entity, serializer_ctx, entities))].into_iter().collect()),
        // todo
        dependencies: PrefabDependencies::default(),
    }
}

// todo
pub fn deserialize_prefab_components(
    components: &[PrefabComponent],
    entity: EntityId,
    mut serializer_ctx: SerializerCtx,
    mut entities: EntitiesHolderMut,
) {
    for component in components {
        let was_deserialized = deserialize_component(
            component.component_id.as_str(),
            &component.data,
            entity,
            serializer_ctx.as_mut(),
            entities.as_mut()
        );

        if !was_deserialized {
            println!("failed to deserialize component: {}", component.component_id);
        }
    }
}

pub fn serialize_components(
    entity: EntityId,
    mut serializer_ctx: SerializerCtx,
    entities: EntitiesHolderRef,
) -> FfiVec<PrefabComponent> {
    let mut components = Vec::new();

    entities.get_all_components(entity, |component| {
        components.push(PrefabComponent {
            component_id: component.type_info().short().name().into(),
            data: serializer_ctx.serialize_any(component),
        });
    });

    components.sort_by(|l, r| l.component_id.cmp(&r.component_id));

    components.into()
}

pub fn deserialize_component(
    id: &str,
    data: &SerializedValue,
    entity: EntityId,
    mut serializer_ctx: SerializerCtx,
    mut entities: EntitiesHolderMut,
) -> bool {
    let Some(component) = serializer_ctx.deserialize_any(id, &data) else {
        return false;
    };
    entities.add_component_any(entity, component).is_ok()
}
