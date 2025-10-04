use std::marker::PhantomData;

use fruits_ecs_core::*;

pub fn main() {
    lib_loading_example();
}

#[derive(Debug)]
pub struct Loud {
    _phantom: PhantomData<()>,
}

impl Loud {
    pub fn new() -> Self {
        println!("Loud.new()");

        Self { _phantom: PhantomData }
    }
}

impl Drop for Loud {
    fn drop(&mut self) {
        println!("Loud.drop()");
    }
}

impl Clone for Loud {
    fn clone(&self) -> Self {
        println!("Loud.clone()");

        Self { _phantom: PhantomData }
    }
}

fn lib_loading_example() {
    let lib = unsafe {
        libloading::Library::new("D:/Projects/Hobby/fruits_engine/target/debug/fruits_ecs_core_sample_dylib").unwrap()
    };
    
    {
        let init_app_symbol = unsafe {
            lib.get::<unsafe extern "C" fn(AppInitCtxFfi)>(b"init_app").unwrap()
        };

        let types = TypesRegistryAccessFfi::new();

        let mut world = WorldBuilderUnsafeFfi::new(types.clone());
    
        unsafe {
            let ctx = AppInitCtxFfi {
                world_mut: &raw mut world,
                types_ref: &raw const types,
            };

            init_app_symbol(ctx);
        }

        {
            let mut world = world.build();

            world.execute_iteration(Schedule::Start);

            for _ in 0..100 {
                world.execute_iteration(Schedule::Update);
            }
        }
    }

    lib.close().unwrap();
}