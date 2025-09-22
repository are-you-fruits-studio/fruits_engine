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

        unsafe {
            let ctx = AppInitCtxFfi {
                res_ref: &raw mut res,
                types_ref: &raw const types,
            };

            init_app_symbol(ctx);
        }
    }

    lib.close().unwrap();
}

fn unsafe_example() {
    let types = TypesRegistryAccessFfi::new();

    unsafe extern "C" fn example_struct_drop_fn(p: *mut std::ffi::c_void) {
        unsafe { std::ptr::drop_in_place(p as *mut ExampleStruct) }
    }
    let example_struct_id = types.try_register(std::any::type_name::<ExampleStruct>(), TypeData {
        size: std::mem::size_of::<ExampleStruct>() as u64,
        align: std::mem::align_of::<ExampleStruct>() as u64,
        drop_fn: Some(example_struct_drop_fn),
    }).unwrap();

    let mut res = ResourcesHolderUnsafeFfi::new(types);

    unsafe {
        (res.insert(example_struct_id).unwrap().as_ptr() as *mut ExampleStruct).write(ExampleStruct {
            name: String::from("Peter"),
            age: 44,
        });
    }

    unsafe {
        let example = &*(res.get(example_struct_id).unwrap().as_ptr() as *mut ExampleStruct);

        dbg!(&example.age);
        dbg!(&example.name);
    };

    drop(res);
}

fn safe_example() {
    let types = TypesRegistryAccessFfi::new();
    let types_cache = TypesRegistryCache::new(types.clone());
    
    let mut res = ResourcesHolderUnsafeFfi::new(types.clone());
    let mut res_ref = ResourcesHolderRef::from_ffi(&mut res, types_cache);
    
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