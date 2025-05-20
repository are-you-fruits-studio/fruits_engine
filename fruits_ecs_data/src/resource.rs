use std::{
    any::{Any, TypeId}, collections::HashMap, ops::{Deref, DerefMut}, sync::{
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