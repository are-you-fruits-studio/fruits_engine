use std::collections::HashMap;

use fruits_serialization::JsonValue;

pub struct PrefabComponent {
    pub id: String,
    pub data: JsonValue,
}

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