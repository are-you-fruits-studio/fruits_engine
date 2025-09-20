use std::collections::HashMap;

pub struct TypesRegistry {
    types: HashMap<String, u64>,
    free_id: u64,
}

impl TypesRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            free_id: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn get_or_register(&mut self, type_name: &str) -> u64 {
        if let Some(&type_id) = self.types.get(type_name) {
            return type_id;
        }

        let new_id = self.free_id;
        self.free_id += 1;

        self.types.insert(type_name.to_string(), new_id);

        new_id
    }

    pub fn get(&self, type_name: &str) -> Option<u64> {
        self.types.get(type_name).copied()
    }
}