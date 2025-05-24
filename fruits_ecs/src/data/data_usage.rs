use std::{any::TypeId, collections::HashMap};

pub enum DataUsage {
    PerType(PerTypeDataUsage),
    // todo: global immutable?
    GlobalMutable,
}
impl DataUsage {
    pub fn new() -> Self {
        DataUsage::PerType(PerTypeDataUsage::new())
    }

    pub fn add(&mut self, usage: DataUsageEntry) {
        let DataUsage::PerType(per_type) = self else {
            panic_invalid_usage()
        };

        per_type.add(usage);
    }

    pub fn add_all_mut(&mut self) {
        let DataUsage::PerType(per_type_usage) = self else {
            panic_invalid_usage()
        };

        if per_type_usage.values().len() != 0 {
            panic_invalid_usage()
        }

        *self = DataUsage::GlobalMutable;
    }

}

fn panic_invalid_usage() -> ! {
    // todo
    panic!("fruits: Invalid system DataUsage.");
}

pub struct DataUsageEntry {
    pub data_type: TypeId,
    pub is_mutable: bool,
}
impl DataUsageEntry {
    pub fn new(data_type: TypeId, is_mutable: bool) -> Self {
        Self {
            data_type,
            is_mutable,
        }
    }
    pub fn new_mutable(type_id: TypeId) -> Self {
        Self::new(type_id, true)
    }
    pub fn new_readonly(type_id: TypeId) -> Self {
        Self::new(type_id, false)
    }
}

pub struct PerTypeDataUsage {
    is_mutable: HashMap<TypeId, bool>
}
impl PerTypeDataUsage {
    pub fn new() -> Self {
        Self {
            is_mutable: HashMap::new(),
        }
    }

    pub fn add(&mut self, usage: DataUsageEntry) {
        self.is_mutable.entry(usage.data_type)
            .and_modify(|v| if *v && usage.is_mutable { panic_invalid_usage() })
            .or_insert(usage.is_mutable);
    }

    pub fn values(&self) -> &HashMap<TypeId, bool> {
        &self.is_mutable
    }

    pub fn into_values(self) -> HashMap<TypeId, bool> {
        self.is_mutable
    }
}