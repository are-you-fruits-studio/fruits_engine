use std::collections::{HashMap, HashSet};

use fruits_ffi::FfiVec;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DataUsageDetails {
    pub is_mutable: bool,
    pub is_required: bool,
}

#[repr(C)]
pub struct DataUsageEntry {
    pub type_id: u64,
    pub details: DataUsageDetails,
}

#[repr(C)]
pub struct DataUsage {
    world_elements: FfiVec<DataUsageEntry>,
    system_elements: FfiVec<u64>,
    is_global: bool,
}

impl DataUsage {
    pub fn global_mut() -> Self {
        Self {
            world_elements: FfiVec::new(),
            system_elements: FfiVec::new(),
            is_global: true,
        }
    }

    pub fn into_elements(self) -> Option<FfiVec<DataUsageEntry>> {
        if self.is_global { None } else { Some(self.world_elements) }
    }

    pub fn as_elements(&self) -> Option<&FfiVec<DataUsageEntry>> {
        if self.is_global { None } else { Some(&self.world_elements) }
    }
}

// todo: ffi
pub struct DataUsageBuilder {
    world_details: HashMap<u64, DataUsageDetails>,
    system_details: HashSet<u64>,
    is_global: bool,
}
impl DataUsageBuilder {
    pub fn new() -> Self {
        Self {
            world_details: HashMap::new(),
            system_details: HashSet::new(),
            is_global: false,
        }
    }

    pub fn add_system(&mut self, type_id: u64) {
        if !self.system_details.insert(type_id) {
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }
    }

    pub fn add_world(&mut self, usage: DataUsageEntry) {
        if self.is_global {
            // todo
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }

        let Some(value) = self.world_details.get_mut(&usage.type_id) else {
            self.world_details.insert(usage.type_id, usage.details);
            return;
        };

        if value.is_mutable || usage.details.is_mutable {
            // todo
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }

        value.is_required |= usage.details.is_required;
    }

    pub fn can_add_anything_to_world(&self) -> bool {
        !self.is_global
    }

    pub fn add_all_mutable_to_world(&mut self) {
        if !self.world_details.is_empty() || self.is_global {
            // todo
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }

        self.is_global = true;
    }

    pub fn build(&self) -> DataUsage {
        DataUsage {
            is_global: self.is_global,
            world_elements: self
                .world_details
                .iter()
                .map(|(&type_id, &details)| DataUsageEntry { details, type_id })
                .collect::<Vec<_>>()
                .into(),
            system_elements: self.system_details.iter().copied().collect(),
        }
    }
}
