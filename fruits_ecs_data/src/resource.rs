use std::{
    any::{Any, TypeId}, cell::UnsafeCell, collections::HashMap, marker::PhantomData, ops::{Deref, DerefMut}, sync::{
        Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard
    }
};

pub trait Resource : 'static + Send + Sync { }

pub struct ResourceHolderUnsafe {
    resources: HashMap<TypeId, Box<dyn Any>>,
}
impl ResourceHolderUnsafe {
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

        self.resources.insert(type_id, Box::new(UnsafeCell::new(r))).unwrap();
        Ok(())
    }

    /// Safety. Lifetimes and access sync should be managed by caller. Deallocation is managed by ResourceHolderUnsafe.
    pub unsafe fn get<R: Resource>(&self) -> Option<*mut R> {
        Some(self.resources.get(&TypeId::of::<R>())?.downcast_ref::<UnsafeCell<R>>().unwrap().get())
    }

    pub fn as_safe(&self) -> &ResourceHolderSafe {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&ResourceHolderUnsafe, &ResourceHolderSafe>(self) }
    }

    pub fn as_safe_mut(&mut self) -> &mut ResourceHolderSafe {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut ResourceHolderUnsafe, &mut ResourceHolderSafe>(self) }
    }
}
// Safety. Is safe itself. Ptr usage is managed by caller
unsafe impl Send for ResourceHolderUnsafe { }
unsafe impl Sync for ResourceHolderUnsafe { }

#[repr(transparent)]
pub struct ResourceHolderSafe {
    resources: ResourceHolderUnsafe,
}
impl ResourceHolderSafe {
    pub fn new() -> Self {
        Self {
            resources: ResourceHolderUnsafe::new(),
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

    // todo: extract from here
    pub unsafe fn as_unsafe(&mut self) -> &mut ResourceHolderUnsafe {
        &mut self.resources
    }
}

pub struct ResourcesHolder {
    resources: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}
impl ResourcesHolder {
    pub fn new() -> Self {
        Self {
            resources: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert<R: Resource + Any + Send + Sync>(&self, r: R) -> Result<(), R> {
        let mut resources = self.resources.lock().unwrap();

        let type_id = TypeId::of::<R>();

        if resources.contains_key(&type_id) {
            return Err(r);
        }

        resources.insert(type_id, Arc::new(RwLock::new(r))).map_or(Ok(()), Err).unwrap();
        Ok(())
    }
    pub fn get<R: Resource>(&self) -> Option<ResourceReadGuard<R>> {
        let resources = self.resources.lock().unwrap();

        Some(ResourceReadGuard::<R>::new(Arc::clone(resources.get(&TypeId::of::<R>())?)).unwrap())
    }
    pub fn get_mut<R: Resource>(&self) -> Option<ResourceWriteGuard<R>> {
        let resources = self.resources.lock().unwrap();

        Some(ResourceWriteGuard::<R>::new(Arc::clone(resources.get(&TypeId::of::<R>())?)).unwrap())
    }
}

pub struct ResourceReadGuard<R: Resource> {
    res: Option<Arc<dyn Any + Send + Sync>>,
    guard: Option<RwLockReadGuard<'static, R>>,
}
impl<R: Resource> ResourceReadGuard<R> {
    fn new(res: Arc<dyn Any + Send + Sync>) -> Option<Self> {
        let guard = res.downcast_ref::<RwLock<R>>()?.read().unwrap();
        // Safety. Safe if guard is dropped first.
        unsafe {
            let guard = std::mem::transmute::<_, RwLockReadGuard<'static, R>>(guard);

            Some(Self {
                res: Some(res),
                guard: Some(guard),
            })
        }
    }
}
impl<R: Resource> Deref for ResourceReadGuard<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}
impl<R: Resource> Drop for ResourceReadGuard<R> {
    fn drop(&mut self) {
        self.guard.take();
        self.res.take();
    }
}

pub struct ResourceWriteGuard<R: Resource> {
    res: Option<Arc<dyn Any + Send + Sync>>,
    guard: Option<RwLockWriteGuard<'static, R>>,
}
impl<R: Resource> ResourceWriteGuard<R> {
    fn new(res: Arc<dyn Any + Send + Sync>) -> Option<Self> {
        let guard = res.downcast_ref::<RwLock<R>>()?.write().unwrap();
        // Safety. Safe if guard is dropped first.
        unsafe {
            let guard = std::mem::transmute::<_, RwLockWriteGuard<'static, R>>(guard);

            Some(Self {
                res: Some(res),
                guard: Some(guard),
            })
        }
    }
}
impl<R: Resource> Deref for ResourceWriteGuard<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}
impl<R: Resource> DerefMut for ResourceWriteGuard<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
    }
}
impl<R: Resource> Drop for ResourceWriteGuard<R> {
    fn drop(&mut self) {
        self.guard.take();
        self.res.take();
    }
}

//

pub struct ResourcesLftHolder {
    resources: ResourcesHolder,
}
impl ResourcesLftHolder {
    pub fn new() -> Self {
        Self {
            resources: ResourcesHolder::new()
        }
    }

    pub fn into_lifetimeless(self) -> ResourcesHolder {
        self.resources
    }

    pub fn insert<R: Resource + Any + Send + Sync>(&mut self, r: R) -> Result<(), R> {
        self.resources.insert(r)
    }

    pub fn get<R: Resource>(&self) -> Option<ResourceLftReadGuard<R>> {
        self.resources.get().map(ResourceLftReadGuard::new)
    }

    pub fn get_mut<R: Resource>(&self) -> Option<ResourceLftWriteGuard<R>> {
        self.resources.get_mut().map(ResourceLftWriteGuard::new)
    }
}

pub struct ResourceLftReadGuard<'h, R: Resource> {
    guard: ResourceReadGuard<R>,
    _phantom: PhantomData<&'h ResourcesLftHolder>,
}
impl<'h, R: Resource> ResourceLftReadGuard<'h, R> {
    fn new(guard: ResourceReadGuard<R>) -> Self {
        Self {
            guard,
            _phantom: Default::default(),
        }
    }
}
impl<'h, R: Resource> Deref for ResourceLftReadGuard<'h, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

pub struct ResourceLftWriteGuard<'h, R: Resource> {
    guard: ResourceWriteGuard<R>,
    _phantom: PhantomData<&'h ResourcesLftHolder>,
}
impl<'h, R: Resource> ResourceLftWriteGuard<'h, R> {
    fn new(guard: ResourceWriteGuard<R>) -> Self {
        Self {
            guard,
            _phantom: Default::default(),
        }
    }
}
impl<'h, R: Resource> Deref for ResourceLftWriteGuard<'h, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}
impl<'h, R: Resource> DerefMut for ResourceLftWriteGuard<'h, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}