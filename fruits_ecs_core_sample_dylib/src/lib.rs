use fruits_ecs_core::*;
use fruits_engine::prelude::*;
use fruits_ffi::FfiOpaqueBox;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_app(ctx: AppInitCtxFfi) {
    let res_mut = unsafe { &mut *ctx.res_mut };
    let types_ref = unsafe { &*ctx.types_ref };
    let systems_mut = unsafe { &mut *ctx.systems_mut };
    let native_data = unsafe { &mut *ctx.native_data_mut };

    let types = TypesRegistryCache::new(types_ref.clone());

    {
        let resources_holder_ref = ResourcesHolderRef::from_ffi(res_mut, &types);

        app_custom_init(resources_holder_ref, systems_mut);
    }

    *native_data = Some(FfiOpaqueBox::new(types)).into();
}

fn app_custom_init(mut res: ResourcesHolderRef, systems: &mut SystemScheduleFfi) {
    println!("hello from cdylib!");

    res.insert(AgeResource { age: 24 }).ok().unwrap();

    systems.insert(SystemFfi::new(hello_system));
    systems.insert(SystemFfi::new(ffi_system_wrapper));
}

#[derive(Resource)]
pub struct AgeResource {
    age: u8,
}

impl Drop for AgeResource {
    fn drop(&mut self) {
        println!("AgeResource drop");
    }
}

pub fn hello_system(ctx: SystemCtxFfi) {
    println!("Hello from dll system!");
}

pub fn ffi_system_wrapper(ctx: SystemCtxFfi) {
    let res = unsafe { &mut *ctx.res_mut };
    let native_data = unsafe { &mut *ctx.native_data_mut };

    let Some(native_data) = native_data.as_mut() else {
        return;
    };

    let types = unsafe { &mut *(native_data.as_ptr() as *mut TypesRegistryCache) };

    let mut res = ResourcesHolderRef::from_ffi(res, types);

    customer_system(&mut res);
}

pub fn customer_system(res: &mut ResourcesHolderRef) {
    res.get_mut::<AgeResource>().unwrap().age = 33;

    println!("age: {}", res.get::<AgeResource>().unwrap().age);

    res.get_mut::<AgeResource>().unwrap().age = 85;

    println!("age: {}", res.get::<AgeResource>().unwrap().age);
}

// todo: potential data access for lib

// todo
// pub struct WorldRef<'r> { }
// impl<'r> WorldRef<'r> {
//     pub fn data(&'r self) -> WorldDataRef<'r> { todo!() }
//     pub fn behavior(&'r self) -> WorldBehaviorRef<'r> { todo!() }
// }

// pub struct WorldDataRef<'r> { }
// impl<'r> WorldDataRef<'r> {
//     pub fn resources(&'r self) -> ResourcesHolderRef<'r> { todo!() }
//     pub fn entities(&'r self) ->  { todo!() }
//     pub fn events(&'r self) ->  { todo!() }
// }

// pub struct WorldBehaviorRef<'r> { }
// impl<'r> WorldBehaviorRef<'r> {

// }