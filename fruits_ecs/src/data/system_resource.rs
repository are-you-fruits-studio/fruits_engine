use std::{any::{Any, TypeId}, collections::HashMap, ops::{Deref, DerefMut}, sync::{Arc, Mutex, MutexGuard}};

pub trait SystemResource : 'static + Send + Sync + Default { }

pub struct SystemResourcesHolder {
    resources: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl SystemResourcesHolder {
    pub fn new() -> Self {
        Self {
            resources: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_create<S: SystemResource>(&self) -> SystemResourceGuard<S> {
        let mut resources = self.resources.lock().unwrap();

        let type_id = TypeId::of::<S>();

        let res = resources.get(&type_id).cloned().unwrap_or_else(|| {
            let res = Arc::new(Mutex::new(S::default()));
            resources.insert(type_id, Arc::clone(&res) as _);
            res
        });
        
        SystemResourceGuard::<S>::new(res).unwrap()
    }
}
impl Default for SystemResourcesHolder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SystemResourceGuard<S: SystemResource> {
    res: Option<Arc<dyn Any + Send + Sync>>,
    guard: Option<MutexGuard<'static, S>>,
}
impl<S: SystemResource> SystemResourceGuard<S> {
    fn new(res: Arc<dyn Any + Send + Sync>) -> Option<Self> {
        let guard = res.downcast_ref::<Mutex<S>>()?.lock().unwrap();
        // Safety. Safe if guard is dropped first.
        unsafe {
            let guard = std::mem::transmute::<MutexGuard<'_, S>, MutexGuard<'static, S>>(guard);

            Some(Self {
                res: Some(res),
                guard: Some(guard),
            })
        }
    }
}
impl<S: SystemResource> Deref for SystemResourceGuard<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}
impl<S: SystemResource> DerefMut for SystemResourceGuard<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
    }
}
impl<S: SystemResource> Drop for SystemResourceGuard<S> {
    fn drop(&mut self) {
        self.guard.take();
        self.res.take();
    }
}