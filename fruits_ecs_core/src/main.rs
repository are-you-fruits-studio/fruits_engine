use std::marker::PhantomData;

use fruits_ecs_core::*;
use fruits_ffi::FfiVec;

pub fn main() {
    // let mut vec = FfiVec::with_capacity(3);

    // vec.push(Loud);
    // vec.push(Loud);
    // vec.push(Loud);

    let vec: FfiVec<_> = vec![
        Loud::new(),
        Loud::new(),
        Loud::new(),
    ].into();

    let vec = vec.clone();

    for i in &vec {
        println!("{:?}", i);
    }

    println!();

    for i in 0..10 {
        println!("{:?}", vec.get(i));
    }
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
    
    let symbol = unsafe {
        lib.get::<unsafe extern "C" fn(*mut ResourcesHolderUnsafeRefFfi)>(b"init_app").unwrap()
    };

    let types = TypesRegistryRef::new();
    let mut res = ResourcesHolderUnsafe::new(types);
    
    unsafe {
        let mut res_ffi = ResourcesHolderUnsafeRefFfi::from_unsafe(&mut res);

        symbol(&raw mut res_ffi);
    }

    lib.close().unwrap();
}

fn unsafe_example() {
    let types = TypesRegistryRef::new();

    unsafe extern "C" fn example_struct_drop_fn(p: *mut std::ffi::c_void) {
        unsafe { std::ptr::drop_in_place(p as *mut ExampleStruct) }
    }
    let example_struct_id = types.try_register(std::any::type_name::<ExampleStruct>(), TypeData {
        size: std::mem::size_of::<ExampleStruct>() as u64,
        align: std::mem::align_of::<ExampleStruct>() as u64,
        drop_fn: Some(example_struct_drop_fn),
    }).unwrap();

    let mut res = ResourcesHolderUnsafe::new(types);

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
    let types = TypesRegistryRef::new();
    let types_ffi = unsafe { TypesRegistryRefRefFfi::from_registry(&types) };
    let types_cache = TypesRegistryCache::new(types_ffi);

    let mut res_unsafe = ResourcesHolderUnsafe::new(types.clone());
    let res_ffi = unsafe { ResourcesHolderUnsafeRefFfi::from_unsafe(&mut res_unsafe) };
    let mut res = unsafe { ResourcesHolderRef::from_ffi(res_ffi, types_cache) };

    res.insert(ExampleStruct {
        name: String::from("Peter"),
        age: 44,
    }).ok().unwrap();

    {
        let example = res.get::<ExampleStruct>().unwrap();

        dbg!(&example.age);
        dbg!(&example.name);
    };

    drop(res);
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