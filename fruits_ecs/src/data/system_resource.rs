use std::{any::{Any, TypeId}, cell::UnsafeCell, collections::HashMap, sync::Mutex};

pub trait SystemResource : 'static + Send + Sync + Default { }

trait AbstractSystemResource {
    fn cell_as_any(&self) -> &dyn Any;
}

struct VirtualSystemResource<S: SystemResource> {
    pub sys_res: UnsafeCell<S>,
}
impl<S: SystemResource> Default for VirtualSystemResource<S> {
    fn default() -> Self {
        Self {
            sys_res: UnsafeCell::new(S::default()),
        }
    }
}
impl<S: SystemResource> AbstractSystemResource for VirtualSystemResource<S> {
    fn cell_as_any(&self) -> &dyn Any {
        &self.sys_res
    }
}

pub struct SystemResourcesHolder {
    resources: Mutex<HashMap<TypeId, Box<dyn AbstractSystemResource>>>,
}

impl SystemResourcesHolder {
    pub fn new() -> Self {
        Self {
            resources: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_create<S: SystemResource>(&self) -> *mut S {
        let mut resources = self.resources.lock().unwrap();

        resources
            .entry(TypeId::of::<S>())
            .or_insert_with(|| Box::new(VirtualSystemResource::<S>::default()))
            .cell_as_any()
            .downcast_ref::<UnsafeCell<S>>().unwrap().get()
    }
}

impl Default for SystemResourcesHolder {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for SystemResourcesHolder { }
unsafe impl Sync for SystemResourcesHolder { }