use std::{ffi::c_void, fmt::Debug};

use fruits_ffi::{FfiDroppable, FfiOptionCopy, FfiTypedDroppable};
use fruits_utils::index_version_collection::{VersionCollection, VersionIndex};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(VersionIndex);

impl Entity {
    pub const EMPTY: Entity = Entity(VersionIndex {
        index: 0,
        version: 0,
    });

    pub fn version_index(&self) -> VersionIndex {
        self.0
    }

    pub fn from_version_index(vi: VersionIndex) -> Self {
        Self(vi)
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity")
            .field("i", &self.0.index)
            .field("v", &self.0.version)
            .finish()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EntityLocation {
    pub archetype_id: u64,
    pub entity_archetype_index: u64,
}

#[derive(Default)]
pub struct EntitiesMetadataNative(VersionCollection<EntityLocation>);

impl EntitiesMetadataNative {
    pub fn new() -> Self {
        Self(VersionCollection::new())
    }

    pub fn insert(&mut self, location: EntityLocation) -> Entity {
        Entity(self.0.insert(location))
    }

    pub fn remove(&mut self, entity: Entity) -> Option<EntityLocation> {
        self.0.remove(entity.0)
    }

    pub fn get(&self, entity: Entity) -> Option<&EntityLocation> {
        self.0.get(entity.0)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityLocation> {
        self.0.get_mut(entity.0)
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.0.contains_index(entity.0)
    }

    pub fn len(&self) -> u64 {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

//

struct EntitiesMetadataFfiVTable {
    insert_fn: unsafe extern "C" fn(*mut c_void, location: EntityLocation) -> Entity,
    remove_fn: unsafe extern "C" fn(*mut c_void, entity: Entity) -> FfiOptionCopy<EntityLocation>,
    get_fn: unsafe extern "C" fn(*const c_void, entity: Entity) -> *const EntityLocation,
    get_mut_fn: unsafe extern "C" fn(*mut c_void, entity: Entity) -> *mut EntityLocation,
    contains_fn: unsafe extern "C" fn(*const c_void, entity: Entity) -> bool,
    len_fn: unsafe extern "C" fn(*const c_void) -> u64,
    is_empty_fn: unsafe extern "C" fn(*const c_void) -> bool,
}

#[repr(C)]
pub struct EntitiesMetadataFfi {
    data: FfiDroppable,
    vtable: FfiTypedDroppable<EntitiesMetadataFfiVTable>,
}

impl EntitiesMetadataFfi {
    pub fn new() -> Self {
        unsafe extern "C" fn ffi_insert(this: *mut c_void, location: EntityLocation) -> Entity {
            unsafe {
                let this = &mut *(this as *mut EntitiesMetadataNative);

                this.insert(location)
            }
        }
        unsafe extern "C" fn ffi_remove(
            this: *mut c_void,
            entity: Entity,
        ) -> FfiOptionCopy<EntityLocation> {
            unsafe {
                let this = &mut *(this as *mut EntitiesMetadataNative);

                let result = this.remove(entity);

                FfiOptionCopy::from_option(result)
            }
        }
        unsafe extern "C" fn ffi_get(this: *const c_void, entity: Entity) -> *const EntityLocation {
            unsafe {
                let this = &*(this as *const EntitiesMetadataNative);

                let result = this.get(entity);

                fruits_ffi::ref_into_nullable_ptr(result)
            }
        }
        unsafe extern "C" fn ffi_get_mut(this: *mut c_void, entity: Entity) -> *mut EntityLocation {
            unsafe {
                let this = &mut *(this as *mut EntitiesMetadataNative);

                let result = this.get_mut(entity);

                fruits_ffi::mut_into_nullable_ptr(result)
            }
        }
        unsafe extern "C" fn ffi_contains(this: *const c_void, entity: Entity) -> bool {
            unsafe {
                let this = &*(this as *const EntitiesMetadataNative);

                this.contains(entity)
            }
        }
        unsafe extern "C" fn ffi_len(this: *const c_void) -> u64 {
            unsafe {
                let this = &*(this as *const EntitiesMetadataNative);

                this.len()
            }
        }
        unsafe extern "C" fn ffi_is_empty(this: *const c_void) -> bool {
            unsafe {
                let this = &*(this as *const EntitiesMetadataNative);

                this.is_empty()
            }
        }

        Self {
            data: FfiDroppable::new(EntitiesMetadataNative::new()),
            vtable: FfiTypedDroppable::new(EntitiesMetadataFfiVTable {
                insert_fn: ffi_insert,
                remove_fn: ffi_remove,
                get_fn: ffi_get,
                get_mut_fn: ffi_get_mut,
                contains_fn: ffi_contains,
                len_fn: ffi_len,
                is_empty_fn: ffi_is_empty,
            }),
        }
    }

    pub fn insert(&mut self, location: EntityLocation) -> Entity {
        unsafe {
            let this = self.data.get();

            (self.vtable.get().insert_fn)(this, location)
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<EntityLocation> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.get().remove_fn)(this, entity);

            result.into_option()
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&EntityLocation> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.get().get_fn)(this, entity);

            fruits_ffi::ref_from_nullable_ptr(result)
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityLocation> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.get().get_mut_fn)(this, entity);

            fruits_ffi::mut_from_nullable_ptr(result)
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        unsafe {
            let this = self.data.get();

            (self.vtable.get().contains_fn)(this, entity)
        }
    }

    pub fn len(&self) -> u64 {
        unsafe {
            let this = self.data.get();

            (self.vtable.get().len_fn)(this)
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            let this = self.data.get();

            (self.vtable.get().is_empty_fn)(this)
        }
    }
}

impl Default for EntitiesMetadataFfi {
    fn default() -> Self {
        Self::new()
    }
}