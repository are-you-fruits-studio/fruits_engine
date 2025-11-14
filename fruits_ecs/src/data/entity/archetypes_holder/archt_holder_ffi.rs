use std::ffi::c_void;

use fruits_ffi::{FfiDroppable, FfiOption, FfiStaticRef, FfiVec};
use fruits_math::Vec2;

use crate::*;

#[repr(C)]
struct ArchetypesHolderFfiVTable {
    len_fn: unsafe extern "C" fn(*const c_void) -> u64,
    by_id_ref_fn: unsafe extern "C" fn(*const c_void, id: u64) -> *const ArchetypeUnsafeFfi,
    by_id_mut_fn: unsafe extern "C" fn(*mut c_void, id: u64) -> *mut ArchetypeUnsafeFfi,
    by_2_ids_ref_fn: unsafe extern "C" fn(*const c_void, id: Vec2<u64>) -> FfiOption<[*const ArchetypeUnsafeFfi; 2]>,
    by_2_ids_mut_fn: unsafe extern "C" fn(*mut c_void, id: Vec2<u64>) -> FfiOption<[*mut ArchetypeUnsafeFfi; 2]>,
    by_components_ref_fn:
        unsafe extern "C" fn(*const c_void, components_ref: *const UniqueComponentsSet) -> *const ArchetypeUnsafeFfi,
    by_components_mut_fn: unsafe extern "C" fn(*mut c_void, components_ref: *const UniqueComponentsSet) -> *mut ArchetypeUnsafeFfi,
    id_by_components_fn: unsafe extern "C" fn(*const c_void, components_ref: *const UniqueComponentsSet) -> FfiOption<u64>,
    ids_by_component_fn: unsafe extern "C" fn(*const c_void, component: u64) -> *const FfiVec<u64>,
    id_by_components_or_create_fn: unsafe extern "C" fn(*mut c_void, components: UniqueComponentsSet) -> u64,
}

#[repr(C)]
pub struct ArchetypesHolderFfi {
    data: FfiDroppable,
    vtable: FfiStaticRef<ArchetypesHolderFfiVTable>,
}

impl ArchetypesHolderFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        unsafe extern "C" fn ffi_len(this: *const c_void) -> u64 {
            unsafe {
                let this = &*(this as *const ArchetypesHolderNative);

                this.len()
            }
        }
        unsafe extern "C" fn ffi_by_id_ref(this: *const c_void, id: u64) -> *const ArchetypeUnsafeFfi {
            unsafe {
                let this = &*(this as *const ArchetypesHolderNative);

                let result = this.by_id_ref(id);

                match result {
                    Some(result) => &raw const *result,
                    None => std::ptr::null(),
                }
            }
        }
        unsafe extern "C" fn ffi_by_id_mut(this: *mut c_void, id: u64) -> *mut ArchetypeUnsafeFfi {
            unsafe {
                let this = &mut *(this as *mut ArchetypesHolderNative);

                let result = this.by_id_mut(id);

                match result {
                    Some(result) => &raw mut *result,
                    None => std::ptr::null_mut(),
                }
            }
        }
        unsafe extern "C" fn ffi_by_2_ids_ref(this: *const c_void, id: Vec2<u64>) -> FfiOption<[*const ArchetypeUnsafeFfi; 2]> {
            unsafe {
                let this = &*(this as *const ArchetypesHolderNative);

                let result = this.by_2_ids_ref(id.into_array());

                FfiOption::from_option(result.map(|a| a.map(|r| &raw const *r)))
            }
        }
        unsafe extern "C" fn ffi_by_2_ids_mut(this: *mut c_void, id: Vec2<u64>) -> FfiOption<[*mut ArchetypeUnsafeFfi; 2]> {
            unsafe {
                let this = &mut *(this as *mut ArchetypesHolderNative);

                let result = this.by_2_ids_mut(id.into_array());

                FfiOption::from_option(result.map(|a| a.map(|r| &raw mut *r)))
            }
        }
        unsafe extern "C" fn ffi_by_components_ref(
            this: *const c_void,
            components_ref: *const UniqueComponentsSet,
        ) -> *const ArchetypeUnsafeFfi {
            unsafe {
                let this = &*(this as *const ArchetypesHolderNative);

                let result = this.by_components_ref(&*components_ref);

                match result {
                    Some(result) => &raw const *result,
                    None => std::ptr::null(),
                }
            }
        }
        unsafe extern "C" fn ffi_by_components_mut(
            this: *mut c_void,
            components_ref: *const UniqueComponentsSet,
        ) -> *mut ArchetypeUnsafeFfi {
            unsafe {
                let this = &mut *(this as *mut ArchetypesHolderNative);

                let result = this.by_components_mut(&*components_ref);

                match result {
                    Some(result) => &raw mut *result,
                    None => std::ptr::null_mut(),
                }
            }
        }
        unsafe extern "C" fn ffi_id_by_components(
            this: *const c_void,
            components_ref: *const UniqueComponentsSet,
        ) -> FfiOption<u64> {
            unsafe {
                let this = &*(this as *const ArchetypesHolderNative);

                let result = this.id_by_components(&*components_ref);

                FfiOption::from_option(result)
            }
        }
        unsafe extern "C" fn ffi_ids_by_component(this: *const c_void, component: u64) -> *const FfiVec<u64> {
            unsafe {
                let this = &*(this as *const ArchetypesHolderNative);

                let result = this.ids_by_component(component);

                match result {
                    Some(result) => &raw const *result,
                    None => std::ptr::null(),
                }
            }
        }
        unsafe extern "C" fn ffi_id_by_components_or_create(this: *mut c_void, components: UniqueComponentsSet) -> u64 {
            unsafe {
                let this = &mut *(this as *mut ArchetypesHolderNative);

                let result = this.id_by_components_or_create(components);

                result
            }
        }

        Self {
            data: FfiDroppable::new(ArchetypesHolderNative::new(types)),
            vtable: FfiStaticRef::new(&ArchetypesHolderFfiVTable {
                len_fn: ffi_len,
                by_id_ref_fn: ffi_by_id_ref,
                by_id_mut_fn: ffi_by_id_mut,
                by_2_ids_ref_fn: ffi_by_2_ids_ref,
                by_2_ids_mut_fn: ffi_by_2_ids_mut,
                by_components_ref_fn: ffi_by_components_ref,
                by_components_mut_fn: ffi_by_components_mut,
                id_by_components_fn: ffi_id_by_components,
                ids_by_component_fn: ffi_ids_by_component,
                id_by_components_or_create_fn: ffi_id_by_components_or_create,
            }),
        }
    }
    pub fn len(&self) -> u64 {
        unsafe {
            let this = self.data.get();

            (self.vtable.len_fn)(this)
        }
    }
    pub fn by_id_ref(&self, id: u64) -> Option<&ArchetypeUnsafeFfi> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.by_id_ref_fn)(this, id);

            if result.is_null() { None } else { Some(&*result) }
        }
    }
    pub fn by_id_mut(&mut self, id: u64) -> Option<&mut ArchetypeUnsafeFfi> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.by_id_mut_fn)(this, id);

            if result.is_null() { None } else { Some(&mut *result) }
        }
    }
    pub fn by_2_ids_ref(&self, id: [u64; 2]) -> Option<[&ArchetypeUnsafeFfi; 2]> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.by_2_ids_ref_fn)(this, Vec2::from_array(id));

            result.into_option().map(|a| a.map(|p| &*p))
        }
    }
    pub fn by_2_ids_mut(&mut self, id: [u64; 2]) -> Option<[&mut ArchetypeUnsafeFfi; 2]> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.by_2_ids_mut_fn)(this, Vec2::from_array(id));

            result.into_option().map(|a| a.map(|p| &mut *p))
        }
    }
    pub fn by_components_ref(&self, components: &UniqueComponentsSet) -> Option<&ArchetypeUnsafeFfi> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.by_components_ref_fn)(this, &raw const *components);

            if result.is_null() { None } else { Some(&*result) }
        }
    }
    pub fn by_components_mut(&mut self, components: &UniqueComponentsSet) -> Option<&mut ArchetypeUnsafeFfi> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.by_components_mut_fn)(this, &raw const *components);

            if result.is_null() { None } else { Some(&mut *result) }
        }
    }
    pub fn id_by_components(&self, components: &UniqueComponentsSet) -> Option<u64> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.id_by_components_fn)(this, &raw const *components);

            result.into_option()
        }
    }
    pub fn ids_by_component(&self, component: u64) -> Option<&FfiVec<u64>> {
        unsafe {
            let this = self.data.get();

            let result = (self.vtable.ids_by_component_fn)(this, component);

            if result.is_null() { None } else { Some(&*result) }
        }
    }
    pub fn id_by_components_or_create(&mut self, components: UniqueComponentsSet) -> u64 {
        unsafe {
            let this = self.data.get();

            (self.vtable.id_by_components_or_create_fn)(this, components)
        }
    }
}
