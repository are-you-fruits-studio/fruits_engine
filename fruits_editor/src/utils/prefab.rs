use crate::*;

// todo: remove in favor of Prefab
#[derive(Debug)]
pub struct InspectorPrefab {
    pub entities: Vec<InspectorPrefabEntity>,
}

#[derive(Debug)]
pub struct InspectorPrefabEntity {
    pub components: Vec<InspectorPrefabComponent>,
}

#[derive(Debug, Clone)]
pub struct InspectorPrefabComponent {
    pub type_name: String,
    pub data: ReflRepr,
}

impl InspectorPrefab {
    pub fn instantiate(world: WorldDataMut) -> EntityId {
        todo!()
    }

    pub fn deserialize(data: &str) -> Option<Self> {
        let json = JsonValue::parse(&mut data.chars())?;

        let JsonValue::Object(prefab) = json else {
            return None;
        };

        let Some(JsonValue::Array(entities)) = prefab.get_value("entities") else {
            return None;
        };

        let mut result_entities = Vec::new();

        for entity in entities {
            let JsonValue::Object(entity) = entity else {
                return None;
            };

            // todo: id.
            let Some(JsonValue::String(id)) = entity.get_value("id") else {
                return None;
            };

            // todo: components.
            let Some(JsonValue::Array(components)) = entity.get_value("components") else {
                return None;
            };

            result_entities.push(InspectorPrefabEntity { components: Vec::new() });
        }

        Some(InspectorPrefab {
            entities: result_entities,
        })
    }
}
