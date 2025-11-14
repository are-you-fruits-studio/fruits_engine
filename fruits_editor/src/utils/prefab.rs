use crate::*;

#[derive(Debug)]
pub struct Prefab {
    pub entities: Vec<PrefabEntity>,
}

#[derive(Debug)]
pub struct PrefabEntity {
    pub components: Vec<PrefabComponent>,
}

#[derive(Debug, Clone)]
pub struct PrefabComponent {
    pub type_name: String,
    pub data: ReflRepr,
}

impl Prefab {
    pub fn instantiate(world: WorldDataMut) -> Entity {
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

            result_entities.push(PrefabEntity { components: Vec::new() });
        }

        Some(Prefab {
            entities: result_entities,
        })
    }
}
