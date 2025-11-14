use std::{
    any::TypeId,
    collections::HashMap,
    ffi::c_void,
    sync::{Arc, RwLock},
};

use fruits_ffi::FfiOption;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct StoredTypeData {
    pub id: u64,
    pub data: TypeData,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TypeData {
    // todo: name & ffi
    pub size: u64,
    pub align: u64,
    pub drop_fn: Option<unsafe extern "C" fn(*mut c_void)>,
}

impl TypeData {
    pub fn of<T: 'static>() -> Self {
        unsafe extern "C" fn example_struct_drop_fn<D: 'static>(p: *mut c_void) {
            unsafe { std::ptr::drop_in_place(p as *mut D) }
        }

        Self {
            size: std::mem::size_of::<T>() as u64,
            align: std::mem::align_of::<T>() as u64,
            drop_fn: Some(example_struct_drop_fn::<T>),
        }
    }
}

//

#[repr(C)]
pub struct TypesRegistryAccessFfi {
    data: *mut c_void,
    clone_fn: unsafe extern "C" fn(*const c_void) -> *mut c_void,
    len_fn: unsafe extern "C" fn(*const c_void) -> u64,
    get_fn: unsafe extern "C" fn(*const c_void, id: u64) -> FfiOption<TypeData>,
    get_by_name_fn:
        unsafe extern "C" fn(*const c_void, name_utf8_ref: *const c_void, name_utf8_len: u64) -> FfiOption<StoredTypeData>,
    try_register_fn:
        unsafe extern "C" fn(*const c_void, name_utf8_ref: *const c_void, name_utf8_len: u64, data: TypeData) -> FfiOption<u64>,
    drop_fn: unsafe extern "C" fn(*mut c_void),
}

//

impl TypesRegistryAccessFfi {
    pub fn new() -> Self {
        let types = TypesRegistryAccessNative::new();

        Self {
            data: Box::into_raw(Box::new(types)) as *mut c_void,
            clone_fn: Self::ffi_clone,
            len_fn: Self::ffi_len,
            get_fn: Self::ffi_get,
            get_by_name_fn: Self::ffi_get_by_name,
            try_register_fn: Self::ffi_try_register,
            drop_fn: Self::ffi_drop,
        }
    }

    pub fn len(&self) -> u64 {
        unsafe { (self.len_fn)((&raw const *self) as *const c_void) }
    }

    pub fn get(&self, id: u64) -> Option<TypeData> {
        unsafe {
            let this = self.data;

            let data = (self.get_fn)(this, id);

            data.into_option()
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<StoredTypeData> {
        unsafe {
            let this = self.data;
            let name_utf8_len = name.len() as u64;
            let name_utf8_ref = name.as_ptr() as *const c_void;

            let data = (self.get_by_name_fn)(this, name_utf8_ref, name_utf8_len);

            data.into_option()
        }
    }

    pub fn try_register(&self, name: &str, data: TypeData) -> Option<u64> {
        unsafe {
            let this = self.data;
            let name_utf8_len = name.len() as u64;
            let name_utf8_ref = name.as_ptr() as *const c_void;

            let data = (self.try_register_fn)(this, name_utf8_ref, name_utf8_len, data);

            data.into_option()
        }
    }

    unsafe extern "C" fn ffi_clone(this: *const c_void) -> *mut c_void {
        unsafe {
            let this = &*(this as *const TypesRegistryAccessNative);

            let cloned = this.clone();

            Box::into_raw(Box::new(cloned)) as *mut c_void
        }
    }
    unsafe extern "C" fn ffi_len(this: *const c_void) -> u64 {
        unsafe {
            let this = &*(this as *const TypesRegistryAccessNative);

            let result = this.len();

            result as u64
        }
    }
    unsafe extern "C" fn ffi_get(this: *const c_void, id: u64) -> FfiOption<TypeData> {
        unsafe {
            let this = &*(this as *const TypesRegistryAccessNative);

            let result = this.get(id);

            FfiOption::from_option(result)
        }
    }
    unsafe extern "C" fn ffi_get_by_name(
        this: *const c_void,
        name_utf8_ref: *const c_void,
        name_utf8_len: u64,
    ) -> FfiOption<StoredTypeData> {
        unsafe {
            let this = &*(this as *const TypesRegistryAccessNative);
            let name_utf8 = std::slice::from_raw_parts(name_utf8_ref as *const u8, name_utf8_len as usize);
            let Ok(name) = str::from_utf8(name_utf8) else {
                return FfiOption::from_option(None);
            };

            let result = this.get_by_name(name);

            FfiOption::from_option(result)
        }
    }
    unsafe extern "C" fn ffi_try_register(
        this: *const c_void,
        name_utf8_ref: *const c_void,
        name_utf8_len: u64,
        data: TypeData,
    ) -> FfiOption<u64> {
        unsafe {
            let this = &*(this as *const TypesRegistryAccessNative);
            let name_utf8 = std::slice::from_raw_parts(name_utf8_ref as *const u8, name_utf8_len as usize);
            let Ok(name) = str::from_utf8(name_utf8) else {
                return None.into();
            };

            let result = this.try_register(name, data);

            result.into()
        }
    }
    pub unsafe extern "C" fn ffi_drop(this_ref: *mut c_void) {
        // todo
        unsafe {
            drop(Box::from_raw(this_ref as *mut TypesRegistryAccessNative));
        }
    }
    // todo
}

impl Drop for TypesRegistryAccessFfi {
    fn drop(&mut self) {
        // todo
        unsafe {
            (self.drop_fn)(self.data);
        }
    }
}

impl Clone for TypesRegistryAccessFfi {
    fn clone(&self) -> Self {
        unsafe {
            let data = (self.clone_fn)(self.data);

            Self {
                data,
                clone_fn: self.clone_fn,
                len_fn: self.len_fn,
                get_fn: self.get_fn,
                get_by_name_fn: self.get_by_name_fn,
                try_register_fn: self.try_register_fn,
                drop_fn: self.drop_fn,
            }
        }
    }
}

unsafe impl Send for TypesRegistryAccessFfi {}
unsafe impl Sync for TypesRegistryAccessFfi {}

//

struct TypesRegistryData {
    types_by_string: HashMap<String, u64>,
    types: Vec<StoredTypeData>,
}

struct TypesRegistryAccessNative {
    data: Arc<RwLock<TypesRegistryData>>,
}

impl Clone for TypesRegistryAccessNative {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl TypesRegistryAccessNative {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(TypesRegistryData {
                types_by_string: HashMap::new(),
                types: Vec::new(),
            })),
        }
    }

    pub fn len(&self) -> usize {
        let self_data = self.data.read().unwrap();

        let result = self_data.types.len();

        result
    }

    pub fn is_empty(&self) -> bool {
        let self_data = self.data.read().unwrap();

        let result = self_data.types.is_empty();

        result
    }

    pub fn get(&self, id: u64) -> Option<TypeData> {
        let self_data = self.data.read().unwrap();

        let result = self_data.types.get(id as usize).copied();

        result.map(|t| t.data)
    }

    pub fn get_by_name(&self, name: &str) -> Option<StoredTypeData> {
        let self_data = self.data.read().unwrap();

        let result = self_data.types.get(*self_data.types_by_string.get(name)? as usize).copied();

        result
    }

    pub fn try_register(&self, name: &str, data: TypeData) -> Option<u64> {
        let mut self_data = self.data.write().unwrap();

        if self_data.types_by_string.get(name).is_some() {
            return None;
        }

        let id = self_data.types.len() as u64;
        self_data.types_by_string.insert(name.to_string(), id);
        self_data.types.push(StoredTypeData { id, data });

        Some(id)
    }
}

//

#[derive(Clone)]
pub struct TypesRegistryCache {
    cache: Arc<RwLock<HashMap<TypeId, u64>>>,
    registry: TypesRegistryAccessFfi,
}

// impl Clone for TypesRegistryCache {
//     fn clone(&self) -> Self {
//         Self {
//             cache: Arc::clone(&self.cache),
//             registry: self.registry.clone(),
//         }
//     }
// }

impl TypesRegistryCache {
    pub fn new(registry: TypesRegistryAccessFfi) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            registry,
        }
    }

    pub fn get_with_type_id(&self, type_id: &TypeId) -> Option<u64> {
        self.cache.read().unwrap().get(type_id).copied()
    }

    pub fn get<T: 'static>(&self) -> Option<u64> {
        self.get_with_type_id(&TypeId::of::<T>())
    }

    pub fn try_register<T: 'static>(&self) -> Option<u64> {
        let mut cache = self.cache.write().unwrap();

        if cache.contains_key(&TypeId::of::<T>()) {
            return None;
        }

        let id = self.registry.try_register(std::any::type_name::<T>(), TypeData::of::<T>());

        if let Some(id) = id {
            cache.insert(TypeId::of::<T>(), id);

            return Some(id);
        }

        if let Some(id) = self.registry.get_by_name(std::any::type_name::<T>()).map(|t| t.id) {
            cache.insert(TypeId::of::<T>(), id);

            return Some(id);
        }

        None
    }

    pub fn get_or_register<T: 'static>(&self) -> u64 {
        self.get::<T>().unwrap_or_else(|| self.try_register::<T>().unwrap())
    }

    pub unsafe fn registry(&self) -> &TypesRegistryAccessFfi {
        &self.registry
    }

    // todo
    // pub unsafe fn raw_registry(&self) -> TypesRegistryAccessRefFfi {
    //     self.registry.clone()
    // }
}
