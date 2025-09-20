use std::{any::TypeId, collections::HashMap, ffi::c_void, sync::{Arc, RwLock}};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OptionalStoredTypeData {
    pub data: StoredTypeData,
    pub is_init: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OptionalU64 {
    pub data: u64,
    pub is_init: bool,
}

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

//

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TypesRegistryRefRefFfi {
    data: *const c_void,
    len_fn: unsafe extern "C" fn(*const c_void) -> u64,
    get_fn: unsafe extern "C" fn(*const c_void, id: u64) -> OptionalStoredTypeData,
    get_by_name_fn: unsafe extern "C" fn(*const c_void, name_utf8_ref: *const c_void, name_utf8_len: u64) -> OptionalStoredTypeData,
    try_register_fn: unsafe extern "C" fn(*const c_void, name_utf8_ref: *const c_void, name_utf8_len: u64, data: TypeData) -> OptionalU64,
}

impl TypesRegistryRefRefFfi {
    pub unsafe fn from_registry(types: &TypesRegistryRef) -> Self {
        Self {
            data: &raw const *types as *const c_void,
            len_fn: Self::ffi_len,
            get_fn: Self::ffi_get,
            get_by_name_fn: Self::ffi_get_by_name,
            try_register_fn: Self::ffi_try_register,
        }
    }

    pub unsafe fn len(&self) -> u64 {
        unsafe {
            (self.len_fn)(&raw const *self as *mut c_void)
        }
    }

    pub unsafe fn get(&self, id: u64) -> Option<StoredTypeData> {
        unsafe {
            let this = &raw const *self as *const c_void;

            let data = (self.get_fn)(this, id);

            if data.is_init {
                Some(data.data)
            } else {
                None
            }
        }
    }

    pub unsafe fn get_by_name(&self, name: &str) -> Option<StoredTypeData> {
        unsafe {
            let this = &raw const *self as *const c_void;
            let name_utf8_len = name.len() as u64;
            let name_utf8_ref = name.as_ptr() as *const c_void;

            let data = (self.get_by_name_fn)(this, name_utf8_ref, name_utf8_len);

            if data.is_init {
                Some(data.data)
            } else {
                None
            }
        }
    }

    pub unsafe fn try_register(&self, name: &str, data: TypeData) -> Option<u64> {
        unsafe {
            let this = &raw const *self as *const c_void;
            let name_utf8_len = name.len() as u64;
            let name_utf8_ref = name.as_ptr() as *const c_void;

            let data = (self.try_register_fn)(this, name_utf8_ref, name_utf8_len, data);

            if data.is_init {
                Some(data.data)
            } else {
                None
            }
        }
    }

    
    unsafe extern "C" fn ffi_len(this: *const c_void) -> u64 {
        unsafe {
            let this = &*(this as *const TypesRegistryRef);

            let result = this.len();

            result as u64
        }
    }
    unsafe extern "C" fn ffi_get(this: *const c_void, id: u64) -> OptionalStoredTypeData {
        unsafe {
            let this = &*(this as *const TypesRegistryRef);

            let result = this.get(id);

            if let Some(result) = result {
                OptionalStoredTypeData {
                    data: result,
                    is_init: true,
                }
            } else {
                OptionalStoredTypeData {
                    data: StoredTypeData {
                        id: 0,
                        data: TypeData {
                            size: 0,
                            align: 0,
                            drop_fn: None,
                        },
                    },
                    is_init: false,
                }
            }
        }
    }
    unsafe extern "C" fn ffi_get_by_name(this: *const c_void, name_utf8_ref: *const c_void, name_utf8_len: u64) -> OptionalStoredTypeData {
        unsafe {
            let this = &*(this as *const TypesRegistryRef);
            let name_utf8 = std::slice::from_raw_parts(name_utf8_ref as *const u8, name_utf8_len as usize);
            let Ok(name) = str::from_utf8(name_utf8) else {
                return OptionalStoredTypeData {
                    data: StoredTypeData {
                        id: 0,
                        data: TypeData {
                            size: 0,
                            align: 0,
                            drop_fn: None,
                        },
                    },
                    is_init: false,
                }
            };

            let result = this.get_by_name(name);

            if let Some(result) = result {
                OptionalStoredTypeData {
                    data: result,
                    is_init: true,
                }
            } else {
                OptionalStoredTypeData {
                    data: StoredTypeData {
                        id: 0,
                        data: TypeData {
                            size: 0,
                            align: 0,
                            drop_fn: None,
                        },
                    },
                    is_init: false,
                }
            }
        }
    }
    unsafe extern "C" fn ffi_try_register(this: *const c_void, name_utf8_ref: *const c_void, name_utf8_len: u64, data: TypeData) -> OptionalU64 {
        unsafe {
            let this = &*(this as *const TypesRegistryRef);
            let name_utf8 = std::slice::from_raw_parts(name_utf8_ref as *const u8, name_utf8_len as usize);
            let Ok(name) = str::from_utf8(name_utf8) else {
                return OptionalU64 {
                    data: 0,
                    is_init: false,
                }
            };

            let result = this.try_register(name, data);

            if let Some(result) = result {
                OptionalU64 {
                    data: result,
                    is_init: true,
                }
            } else {
                OptionalU64 {
                    data: 0,
                    is_init: false,
                }
            }
        }
    }
    // todo
}

//

struct TypesRegistryData {
    types_by_string: HashMap<String, u64>,
    types: Vec<StoredTypeData>,
}

pub struct TypesRegistryRef {
    data: Arc<RwLock<TypesRegistryData>>,
}

impl Clone for TypesRegistryRef {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl TypesRegistryRef {
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

        self_data.types.len()
    }

    pub fn is_empty(&self) -> bool {
        let self_data = self.data.read().unwrap();

        self_data.types.is_empty()
    }

    pub fn get(&self, id: u64) -> Option<StoredTypeData> {
        let self_data = self.data.read().unwrap();

        self_data.types.get(id as usize).copied()
    }

    pub fn get_by_name(&self, name: &str) -> Option<StoredTypeData> {
        let self_data = self.data.read().unwrap();

        self.data.read().unwrap().types.get(*self_data.types_by_string.get(name)? as usize).copied()
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

pub struct TypesRegistryCache {
    cache: Arc<RwLock<HashMap<TypeId, u64>>>,
    registry: TypesRegistryRefRefFfi,
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
    pub fn new(registry: TypesRegistryRefRefFfi) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            registry,
        }
    }

    pub fn get<T: 'static>(&self) -> Option<u64> {
        let cache = self.cache.read().unwrap();

        cache.get(&TypeId::of::<T>()).copied()
    }

    pub fn try_register<T: 'static>(&self) -> Option<u64> {
        let mut cache = self.cache.write().unwrap();

        if cache.contains_key(&TypeId::of::<T>()) {
            return None;
        }

        unsafe extern "C" fn example_struct_drop_fn<D: 'static>(p: *mut c_void) {
            unsafe { std::ptr::drop_in_place(p as *mut D) }
        }

        let id = unsafe { self.registry.try_register(std::any::type_name::<T>(), TypeData {
            size: std::mem::size_of::<T>() as u64,
            align: std::mem::align_of::<T>() as u64,
            drop_fn: Some(example_struct_drop_fn::<T>),
        }) };

        if let Some(id) = id {
            cache.insert(TypeId::of::<T>(), id);
            
            return Some(id);
        }

        unsafe {
            if let Some(id) = self.registry.get_by_name(std::any::type_name::<T>()).map(|t| t.id) {
                cache.insert(TypeId::of::<T>(), id);
            
                return Some(id);
            }
        }

        None
    }

    pub fn get_or_register<T: 'static>(&self) -> u64 {
        self.get::<T>().unwrap_or_else(|| self.try_register::<T>().unwrap())
    }

    pub unsafe fn raw_registry(&self) -> TypesRegistryRefRefFfi {
        self.registry.clone()
    }
}