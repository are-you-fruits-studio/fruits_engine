use crate::*;

pub struct EntitiesInfo<'e> {
    ec: &'e EntitiesComponentsHolderUnsafe,
}

impl<'e> EntitiesInfo<'e> {
    pub fn exists(&self, entity: Entity) -> bool {
        self.ec.contains_entity(entity)
    }
    
    pub fn entities_count(&self) -> usize {
        self.ec.entities_count()
    }
}

unsafe impl<'e> SystemParam for EntitiesInfo<'e> {
    type Item<'b> = EntitiesInfo<'b>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_static::<Entity>(DataUsageDetails { is_mutable: false, is_required: true }));
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(EntitiesInfo {
            // Safety. Managed by caller
            ec: unsafe { &*input.world_data.entities_components() },
        })
    }
}
