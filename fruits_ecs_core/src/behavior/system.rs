use std::ffi::c_void;

use fruits_ffi::{FfiDroppable, FfiOpaqueBox, FfiOption, FfiString};

use crate::*;

pub struct SystemInput<'a> {
    pub world_data: WorldDataUnsafeRef<'a>,
    pub system_data: SystemResourcesUnsafeHolderRef<'a>,
}

//

fn system_with_marker_into_system_ffi<S: SystemWithMarker<M>, M: 'static>(system: S) -> SystemFfi {
    SystemFfi::new(move |ctx| {
        unsafe {
            let world = &mut *ctx.world_mut;
            let system_data = &mut *ctx.system_data;
            let types = &*((&mut *ctx.native_data_mut).as_mut().unwrap().as_ptr() as *const TypesRegistryCache);

            let system_input = SystemInput {
                world_data: WorldDataUnsafeRef::new(world, types),
                system_data: SystemResourcesUnsafeHolderRef::new(system_data, types),
            };

            system.execute(system_input);
        }
    })
}

//

/// # Safety
/// 
/// Implemented automatically.
pub unsafe trait SystemParam {
    type Item<'e> : 'e + SystemParam;

    fn fill_data_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache);
    /// # Safety
    /// 
    /// Should be managed by system scheduler and data usage.
    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str>;
}

/// # Safety
/// 
/// Implemented automatically.
pub unsafe trait SystemWithMarker<M: 'static> : 'static + Send + Sync {
    fn fill_data_usage(&self, usage: &mut DataUsageBuilder, types: &TypesRegistryCache);
    /// # Safety
    /// 
    /// Should be managed by system scheduler and data usage.
    unsafe fn execute<'e>(&self, data: SystemInput<'e>);
    fn system_name(&self) -> &'static str;
}

//

macro_rules! system_with_marker_impl {
    ($($P: ident),*) => {
        unsafe impl<F, R, $($P),*> SystemWithMarker<fn($($P),*)> for F
        where
            F: 'static + Send + Sync + Fn($($P),*) -> R + for<'a> Fn($($P::Item<'a>),*) -> R,
            fn($($P),*): 'static,
            $($P: SystemParam),*
        {
            #[allow(redundant_semicolons)]
            fn fill_data_usage(&self, _usage: &mut DataUsageBuilder, _types: &TypesRegistryCache) {
                $($P::fill_data_usage(_usage, _types));*;
            }
        
            unsafe fn execute<'e>(&self, _data: SystemInput<'e>) {
                self(
                    // Safety. Managed by caller.
                    $(unsafe { get_param_or_panic::<F, $P>(&_data) },)*
                );
            }

            fn system_name(&self) -> &'static str {
                std::any::type_name::<F>()
            }
        }
    };
}

fn panic_cannot_obtain_param<F, P>(msg: &'static str) -> ! {
    panic!(
        "System cannot obtain its parameters. System: {}. Parameter: {}. Message: {}",
        std::any::type_name::<F>(),
        std::any::type_name::<P>(),
        msg,
    )
}

unsafe fn get_param_or_panic<'e, F, P: SystemParam>(data: &'e SystemInput<'e>) -> P::Item<'e> {
    // Safety. Managed by caller.
    unsafe { P::new(data) }.unwrap_or_else(|m| panic_cannot_obtain_param::<F, P>(m))
}

system_with_marker_impl!();
system_with_marker_impl!(P0);
system_with_marker_impl!(P0, P1);
system_with_marker_impl!(P0, P1, P2);
system_with_marker_impl!(P0, P1, P2, P3);
system_with_marker_impl!(P0, P1, P2, P3, P4);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14);

//


#[repr(C)]
pub struct SystemFfi {
    system_name: FfiString,
    data_usage: DataUsage,
    system_data: FfiDroppable,
    execute_fn: unsafe extern "C" fn(*const c_void, SystemCtxFfi),
}

impl SystemFfi {
    pub fn new<F: Fn(SystemCtxFfi)>(f: F) -> Self {
        unsafe extern "C" fn ffi_execute<D: Fn(SystemCtxFfi)>(system_data: *const c_void, ctx: SystemCtxFfi) {
            unsafe {
                (&*(system_data as *const D))(ctx);
            }
        }

        Self {
            // todo: use actual data usage.
            data_usage: DataUsage::global_mut(),
            execute_fn: ffi_execute::<F>,
            system_data: FfiDroppable::new(f),
            system_name: FfiString::from_string(String::from(std::any::type_name::<F>())),
        }
    }

    pub fn data_usage(&self) -> &DataUsage {
        &self.data_usage
    }
    // todo:

    // /// # Safety
    // /// 
    // /// Should be managed by system scheduler and data usage.
    // pub unsafe fn execute<'e>(&self, data: &SystemInput<'e>) {

    // }

    pub fn execute(&self, ctx: SystemCtxFfi) {
        unsafe {
            (self.execute_fn)(self.system_data.get(), ctx)
        }
    }

    pub fn system_name(&self) -> &str {
        self.system_name.as_str()
    }
}

// todo
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SystemCtxFfi {
    pub world_mut: *mut WorldDataUnsafeFfi,
    pub system_data: *mut SystemResourcesHolderUnsafeFfi,
    pub types_ref: *const TypesRegistryAccessFfi,
    pub native_data_mut: *mut FfiOption<FfiOpaqueBox>,
}