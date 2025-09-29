use std::{
    collections::BTreeMap,
    hash::Hash
};

use fruits_ffi::FfiVec;

use crate::*;

#[repr(transparent)]
#[derive(Clone, Default, Debug)]
pub struct UniqueComponentsSet {
    components: FfiVec<StoredTypeData>,
}

impl UniqueComponentsSet {
    pub fn components(&self) -> &[StoredTypeData] {
        &self.components
    }
}

impl Hash for UniqueComponentsSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for component_id in &self.components {
            component_id.id.hash(state);
        }
    }
}

impl PartialEq for UniqueComponentsSet {
    fn eq(&self, other: &Self) -> bool {
        if self.components.len() != other.components.len() {
            return false;
        }

        for i in 0..(self.components.len() as usize) {
            if self.components[i].id != other.components[i].id {
                return false;
            }
        }

        return true;
    }
}

impl Eq for UniqueComponentsSet { }

//

#[derive(Clone, Default, Debug)]
pub struct UniqueComponentsSetBuilder {
    components: BTreeMap<u64, TypeData>,
}

impl UniqueComponentsSetBuilder {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
        }
    }

    pub fn components(&self) -> &BTreeMap<u64, TypeData> {
        &self.components
    }

    pub fn insert(&mut self, type_data: StoredTypeData) -> bool {
        self.components.insert(type_data.id, type_data.data).is_none()
    }

    pub fn remove(&mut self, type_id: u64) -> bool {
        self.components.remove(&type_id).is_some()
    }
}

impl Hash for UniqueComponentsSetBuilder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for component_id in self.components.keys() {
            component_id.hash(state);
        }
    }
}

impl PartialEq for UniqueComponentsSetBuilder {
    fn eq(&self, other: &Self) -> bool {
        if self.components.len() != other.components.len() {
            return false;
        }

        self.components.keys().all(|c| other.components.contains_key(c))
    }
}

impl Eq for UniqueComponentsSetBuilder { }
