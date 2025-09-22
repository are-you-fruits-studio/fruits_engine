use fruits_ecs_core::*;
use fruits_engine::prelude::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_app(ctx: AppInitCtxFfi) {
    let res_ref = unsafe { &mut *ctx.res_ref };

    let types = TypesRegistryCache::new(ctx.types);

    let resources_holder_ref = ResourcesHolderRef::from_ffi(res_ref, types);

    app_custom_init(resources_holder_ref);
}

fn app_custom_init(mut res: ResourcesHolderRef) {
    println!("hello from cdylib!");
    
    res.insert(AgeResource { age: 24 }).ok().unwrap();

    println!("age: {}", res.get::<AgeResource>().unwrap().age);

    res.get_mut::<AgeResource>().unwrap().age = 85;

    println!("age: {}", res.get::<AgeResource>().unwrap().age);
}

#[derive(Resource)]
pub struct AgeResource {
    age: u8,
}