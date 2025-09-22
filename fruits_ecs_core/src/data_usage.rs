use std::collections::HashMap;

use fruits_ffi::FfiVec;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DataUsageDetails {
    pub is_mutable: bool,
    pub is_required: bool,
}

pub struct DataUsageEntry {
    pub type_id: u64,
    pub details: DataUsageDetails,
}

#[repr(C)]
pub struct DataUsage {
    elements: FfiVec<DataUsageEntry>,
    is_global: bool,
}

impl DataUsage {
    pub fn global_mut() -> Self {
        Self {
            elements: FfiVec::new(),
            is_global: true,
        }
    }
}

pub struct DataUsageBuilder {
    details: HashMap<u64, DataUsageDetails>
}
impl DataUsageBuilder {
    pub fn new() -> Self {
        Self {
            details: HashMap::new(),
        }
    }

    pub fn add(&mut self, usage: DataUsageEntry) {
        let Some(value) = self.details.get_mut(&usage.type_id) else {
            self.details.insert(usage.type_id, usage.details);
            return;
        };

        if value.is_mutable || usage.details.is_mutable {
            // todo
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }

        value.is_required |= usage.details.is_required;
    }

    pub fn build(&self) -> DataUsage {
        DataUsage {
            is_global: false,
            elements: self.details.iter().map(|(&type_id, &details)| DataUsageEntry {
                details,
                type_id,
            }).collect::<Vec<_>>().into()
        }
    }
}