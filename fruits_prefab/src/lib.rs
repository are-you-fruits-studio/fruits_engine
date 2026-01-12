use std::{collections::HashMap, marker::PhantomData};

use fruits_ecs::*;
use fruits_serialization::SerializerCtx;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrefabComponent {
    pub component_id: String,
    pub data: serde_json::Value,
}

#[derive(Debug)]
pub struct Prefab {
    pub entities: HashMap<usize, Vec<PrefabComponent>>,
}

impl Prefab {
    pub fn empty() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
}

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
        data: serde_json::Value,
        entity: Entity,
        serializer_ctx: &SerializerCtx,
        entities: EntitiesHolderMut,
        res: ResourcesHolderRef,
    ) -> bool;
}

impl<C: Component> AbstractPrefabComponentDeserializer for PrefabComponentDeserializer<C> {
    fn deserialize(
        &self,
        data: serde_json::Value,
        entity: Entity,
        serializer_ctx: &SerializerCtx,
        mut entities: EntitiesHolderMut,
        _res: ResourcesHolderRef,
    ) -> bool {
        let Ok(component) = serializer_ctx.deserialize::<C>(&data) else {
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
        data: serde_json::Value,
        entity: Entity,
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
