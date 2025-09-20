use fruits_ecs_core::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_app(res: *mut ResourcesHolderUnsafeRefFfi) {
    println!("hello from cdylib!");
}