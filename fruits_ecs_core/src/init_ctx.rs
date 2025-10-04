
use crate::{TypesRegistryAccessFfi, WorldBuilderUnsafeFfi};

#[repr(C)]
pub struct AppInitCtxFfi {
    pub types_ref: *const TypesRegistryAccessFfi,
    pub world_mut: *mut WorldBuilderUnsafeFfi,
}