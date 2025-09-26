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
        let mut res = ResourcesHolderUnsafeFfi::new(types.clone());
        let mut systems = SystemScheduleFfi::new();
        let mut native_data = None.into();

        unsafe {
            let ctx = AppInitCtxFfi {
                res_mut: &raw mut res,
                types_ref: &raw const types,
                systems_mut: &raw mut systems,
                native_data_mut: &raw mut native_data,
            };

            init_app_symbol(ctx);
        }

        {
            let system_ctx = SystemCtxFfi {
                res_mut: &raw mut res,
                types_ref: &raw const types,
                native_data_mut: &raw mut native_data,
            };

            for _ in 0..3 {
                systems.execute(system_ctx);
            }
        }
    }

    lib.close().unwrap();
}

fn safe_example() {
    let types = TypesRegistryAccessFfi::new();
    let types_cache = TypesRegistryCache::new(types.clone());
    
    let mut res = ResourcesHolderUnsafeFfi::new(types.clone());
    let mut res_ref = ResourcesHolderRef::from_ffi(&mut res, &types_cache);
    
    res_ref.insert(ExampleStruct {
        name: String::from("Peter"),
        age: 44,
    }).ok().unwrap();
    
    {
        let example = res_ref.get::<ExampleStruct>().unwrap();

        dbg!(&example.age);
        dbg!(&example.name);
    };

    drop(res_ref);
}

pub struct ExampleStruct {
    pub name: String,
    pub age: u8,
}

impl Drop for ExampleStruct {
    fn drop(&mut self) {
        println!("ExampleStruct drop");
    }
}