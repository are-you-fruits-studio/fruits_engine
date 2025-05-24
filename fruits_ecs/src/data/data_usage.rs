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

pub struct DataUsageDetails {
    pub is_mutable: bool,
    pub is_required: bool,
}

pub struct DataUsageEntry {
    pub data_type: TypeId,
    pub details: DataUsageDetails,
}
impl DataUsageEntry {
    pub fn new(data_type: TypeId, details: DataUsageDetails) -> Self {
        Self {
            data_type,
            details,
        }
    }
    pub fn new_static<T: ?Sized + 'static>(details: DataUsageDetails) -> Self {
        Self::new(TypeId::of::<T>(), details)
    }
}

pub struct PerTypeDataUsage {
    details: HashMap<TypeId, DataUsageDetails>
}
impl PerTypeDataUsage {
    pub fn new() -> Self {
        Self {
            details: HashMap::new(),
        }
    }

    pub fn add(&mut self, usage: DataUsageEntry) {
        let Some(value) = self.details.get_mut(&usage.data_type) else {
            self.details.insert(usage.data_type, usage.details);
            return;
        };

        if value.is_mutable || usage.details.is_mutable {
            panic_invalid_usage();
        }

        value.is_required |= usage.details.is_required;
    }

    pub fn values(&self) -> &HashMap<TypeId, DataUsageDetails> {
        &self.details
    }

    pub fn into_values(self) -> HashMap<TypeId, DataUsageDetails> {
        self.details
    }
}