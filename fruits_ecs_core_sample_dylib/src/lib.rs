use fruits_ecs_core::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_app(ctx: AppInitCtxFfi) {
    let world = unsafe { &mut *ctx.world_mut };
    let types_ref = unsafe { &*ctx.types_ref };

    let types = TypesRegistryCache::new(types_ref.clone());

    {
        let world = WorldBuilderMut::new(world, &types);

        app_custom_init(world);
    }
}

fn app_custom_init(mut world: WorldBuilderMut) {
    println!("hello from cdylib!");

    let mut world_data = world.data();

    let mut res = world_data.resources_mut();
    
    res.insert(AgeResource { age: 24 }).ok().unwrap();

    //

    let mut ent = world_data.entities_mut();
    
    let e1 = ent.create_entity();

    ent.add_component(e1, NameComponent { name: String::from("Serhii") });

    //

    let mut world_behavior = world.behavior();

    let mut update = world_behavior.get_mut(Schedule::Update);

    update.insert_system(hello_system);
    update.insert_system(ffi_system_wrapper);
}

// todo
// #[derive(Resource)]
pub struct AgeResource {
    age: u8,
}

// todo
// #[derive(Component)]
pub struct NameComponent {
    name: String,
}

impl Drop for AgeResource {
    fn drop(&mut self) {
        println!("AgeResource drop");
    }
}

pub fn hello_system() {
    println!("Hello from dll system!");
}

pub fn ffi_system_wrapper(mut world: ExclusiveWorldAccess) {
    world.resources_mut().get_mut::<AgeResource>().unwrap().age = 33;

    println!("age: {}", world.resources_mut().get::<AgeResource>().unwrap().age);

    world.resources_mut().get_mut::<AgeResource>().unwrap().age = 85;

    println!("age: {}", world.resources_mut().get::<AgeResource>().unwrap().age);
}
