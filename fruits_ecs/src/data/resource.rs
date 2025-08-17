use std::{any::{Any, TypeId}, cell::UnsafeCell, collections::HashMap};

pub trait Resource : 'static + Send + Sync { }

#[derive(Default)]
pub struct ResourcesHolderUnsafe {
    resources: HashMap<TypeId, Box<dyn Any>>,
}
impl ResourcesHolderUnsafe {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R: Resource + Any + Send + Sync>(&mut self, r: R) -> Result<(), R> {
        let type_id = TypeId::of::<R>();

        if self.resources.contains_key(&type_id) {
            return Err(r);
        }

        self.resources.insert(type_id, Box::new(UnsafeCell::new(r))).map_or(Ok(()), Err).unwrap();
        Ok(())
    }

    /// Safety. Lifetimes and access sync should be managed by caller. Deallocation is managed by ResourceHolderUnsafe.
    pub unsafe fn get<R: Resource>(&self) -> Option<*mut R> {
        Some(self.resources.get(&TypeId::of::<R>())?.downcast_ref::<UnsafeCell<R>>().unwrap().get())
    }

    pub fn as_safe(&self) -> &ResourcesHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&ResourcesHolderUnsafe, &ResourcesHolder>(self) }
    }

    pub fn as_safe_mut(&mut self) -> &mut ResourcesHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut ResourcesHolderUnsafe, &mut ResourcesHolder>(self) }
    }
}
// Safety. Is safe itself. Ptr usage is managed by caller
unsafe impl Send for ResourcesHolderUnsafe { }
unsafe impl Sync for ResourcesHolderUnsafe { }

#[repr(transparent)]
pub struct ResourcesHolder {
    resources: ResourcesHolderUnsafe,
}
impl ResourcesHolder {
    pub fn new() -> Self {
        Self {
            resources: ResourcesHolderUnsafe::new(),
        }
    }

    pub fn insert<R: Resource + Any + Send + Sync>(&mut self, r: R) -> Result<(), R> {
        self.resources.insert(r)
    }
    pub fn get<R: Resource>(&self) -> Option<&R> {
        // Safety. Lifetimes manage the access and syncing.
        unsafe { self.resources.get().map(|r| &*r) }
    }
    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        // Safety. Lifetimes manage the access and syncing.
        unsafe { self.resources.get().map(|r| &mut *r) }
    }
}
