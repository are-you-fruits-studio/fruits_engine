use std::collections::HashMap;

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

    pub fn into_elements(self) -> Option<FfiVec<DataUsageEntry>> {
        if self.is_global {
            None
        } else {
            Some(self.elements)
        }
    }

    pub fn as_elements(&self) -> Option<&FfiVec<DataUsageEntry>> {
        if self.is_global {
            None
        } else {
            Some(&self.elements)
        }
    }
}

pub struct DataUsageBuilder {
    details: HashMap<u64, DataUsageDetails>,
    is_global: bool,
}
impl DataUsageBuilder {
    pub fn new() -> Self {
        Self {
            details: HashMap::new(),
            is_global: false,
        }
    }

    pub fn add(&mut self, usage: DataUsageEntry) {
        if self.is_global {
            // todo
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }

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

    pub fn can_add_anything(&self) -> bool {
        !self.is_global
    }

    pub fn add_all_mutable(&mut self) {
        if !self.details.is_empty() || self.is_global {
            // todo
            println!("fruits: Invalid system DataUsage.");
            panic!("fruits: Invalid system DataUsage.");
        }

        self.is_global = true;
    }

    pub fn build(&self) -> DataUsage {
        DataUsage {
            is_global: self.is_global,
            elements: self.details.iter().map(|(&type_id, &details)| DataUsageEntry {
                details,
                type_id,
            }).collect::<Vec<_>>().into()
        }
    }
}