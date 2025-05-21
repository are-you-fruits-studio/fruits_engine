use std::{
    any::{Any, TypeId}, collections::HashMap, marker::PhantomData, ops::{Deref, DerefMut}, sync::{
        Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard
    }
};

pub trait Resource : 'static + Send + Sync { }

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
        Self::from_lifetimeless(ResourcesHolder::new())
    }

    pub fn from_lifetimeless(resources: ResourcesHolder) -> Self {
        Self {
            resources,
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