
use crate::{SystemScheduleFfi, TypesRegistryAccessFfi, WorldDataUnsafeFfi};

#[repr(C)]
pub struct AppInitCtxFfi {
    pub types_ref: *const TypesRegistryAccessFfi,
    pub world_mut: *mut WorldDataUnsafeFfi,
    pub systems_mut: *mut SystemScheduleFfi,
}