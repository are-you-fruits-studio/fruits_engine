mod vec;
mod alloc;
mod slice;
mod string;
mod option;

pub use vec::*;
pub use alloc::*;
pub use slice::*;
pub use string::*;
pub use option::*;

// example
#[unsafe(no_mangle)]
pub extern "C" fn init_fruits_app() -> SystemsInitData {
    let mut ctx = FruitsAppInitCtx::new();

    ctx.insert_system();

    ctx.into_systems_init_data()
}
//

pub struct FruitsAppInitCtx {
    systems: Vec<SystemInitData>,
    ordering: Vec<SystemOrderingEntry>,
}

impl FruitsAppInitCtx {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            ordering: Vec::new(),
        }
    }
    pub fn into_systems_init_data(self) -> SystemsInitData {
        let systems = self.systems.into_boxed_slice();
        let systems_len = systems.len() as u64;
        let systems_ptr = Box::into_raw(systems) as *mut SystemInitData;
        
        let ordering = self.ordering.into_boxed_slice();
        let ordering_len = ordering.len() as u64;
        let ordering_ptr = Box::into_raw(ordering) as *mut SystemOrderingEntry;

        SystemsInitData {
            systems_ptr,
            systems_len,
            ordering_ptr,
            ordering_len,
        }
    }
    pub fn insert_system(&mut self, /* todo: system */) {

    }
    pub fn insert_group(&mut self, /* todo: group */) {

    }
    pub fn order_system(&mut self, /* todo: system */) {
        
    }
    pub fn order_group(&mut self, /* todo: group */) {
        
    }
}

#[repr(C)]
pub struct SystemsInitData {
    systems_ptr: *mut SystemInitData,
    systems_len: u64,
    ordering_ptr: *mut SystemOrderingEntry,
    ordering_len: u64,
}

#[repr(C)]
pub struct SystemInitData {
    // todo: params
    callable_data: *mut std::ffi::c_void,
    callable: extern "C" fn(*mut std::ffi::c_void),
    system_id: u64,
    system_name_ptr: *mut u8,
    system_name_len: u64,
}

#[repr(C)]
pub struct SystemOrderingEntry {
    prev_id: u64,
    next_id: u64,
}