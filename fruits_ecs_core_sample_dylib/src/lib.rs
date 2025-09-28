use fruits_ecs_core::*;
use fruits_engine::prelude::*;
use fruits_ffi::FfiOpaqueBox;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_app(ctx: AppInitCtxFfi) {
    let world_mut = unsafe { &mut *ctx.world_mut };
    let types_ref = unsafe { &*ctx.types_ref };
    let systems_mut = unsafe { &mut *ctx.systems_mut };
    let native_data = unsafe { &mut *ctx.native_data_mut };

    let types = TypesRegistryCache::new(types_ref.clone());

    {
        let world = WorldDataUnsafeMut::new(world_mut, types.clone()).into_safe();

        app_custom_init(world, systems_mut);
    }

    *native_data = Some(FfiOpaqueBox::new(types)).into();
}

fn app_custom_init(mut world: WorldDataMut, systems: &mut SystemScheduleFfi) {
    println!("hello from cdylib!");

    world.resources_mut().insert(AgeResource { age: 24 }).ok().unwrap();

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
    let world = unsafe { &mut *ctx.world_mut };
    let native_data = unsafe { &mut *ctx.native_data_mut };

    let Some(native_data) = native_data.as_mut() else {
        return;
    };

    let types = unsafe { &mut *(native_data.as_ptr() as *mut TypesRegistryCache) };

    let world = WorldDataUnsafeMut::new(world, types.clone()).into_safe();

    customer_system(world);
}

pub fn customer_system(mut world: WorldDataMut) {
    world.resources_mut().get_mut::<AgeResource>().unwrap().age = 33;

    println!("age: {}", world.resources_mut().get::<AgeResource>().unwrap().age);

    world.resources_mut().get_mut::<AgeResource>().unwrap().age = 85;

    println!("age: {}", world.resources_mut().get::<AgeResource>().unwrap().age);
}
