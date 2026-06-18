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

use std::{collections::HashMap, marker::PhantomData};

use fruits_ecs::*;
use fruits_ffi::{FfiIndexMap, FfiString, FfiVec};
use fruits_serialization::{SerializedValue, SerializerCtx};

#[repr(C)]
#[derive(Debug)]
pub struct PrefabComponent {
    pub component_id: FfiString,
    pub data: SerializedValue,
}

#[repr(C)]
#[derive(Debug)]
pub struct Prefab {
    pub entities: FfiIndexMap<usize, FfiVec<PrefabComponent>>,
}

impl Prefab {
    pub fn empty() -> Self {
        Self {
            entities: FfiIndexMap::new(),
        }
    }
}

#[repr(C)]
pub struct PrefabComponentDeserializer<C: Component> {
    _phantom: PhantomData<fn(C) -> C>,
}

impl<C: Component> Default for PrefabComponentDeserializer<C> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

trait AbstractPrefabComponentDeserializer {
    fn deserialize(
        &self,
        data: SerializedValue,
        entity: EntityId,
        serializer_ctx: &SerializerCtx,
        entities: EntitiesHolderMut,
        res: ResourcesHolderRef,
    ) -> bool;
}

impl<C: Component> AbstractPrefabComponentDeserializer for PrefabComponentDeserializer<C> {
    fn deserialize(
        &self,
        data: SerializedValue,
        entity: EntityId,
        serializer_ctx: &SerializerCtx,
        mut entities: EntitiesHolderMut,
        _res: ResourcesHolderRef,
    ) -> bool {
        let Some(component) = serializer_ctx.deserialize::<C>(&data).result else {
            return false;
        };

        entities.add_component(entity, component).is_ok()
    }
}

// todo: ffi
#[derive(Resource, Default)]
pub struct PrefabComponentsDeserializerResource {
    deserializers: HashMap<String, Box<dyn AbstractPrefabComponentDeserializer + Send + Sync>>,
}

impl PrefabComponentsDeserializerResource {
    pub fn register<C: Component>(&mut self) {
        self.deserializers.insert(
            std::any::type_name::<C>().to_string(),
            Box::new(PrefabComponentDeserializer::<C>::default()),
        );
    }

    pub fn deserialize(
        &self,
        id: &str,
        data: SerializedValue,
        entity: EntityId,
        serializer_ctx: &SerializerCtx,
        entities: EntitiesHolderMut,
        res: ResourcesHolderRef,
    ) -> bool {
        let Some(deserializer) = self.deserializers.get(id) else {
            return false;
        };

        deserializer.deserialize(data, entity, serializer_ctx, entities, res)
    }
}
