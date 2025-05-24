use std::any::TypeId;

#[derive(Clone, Copy, Debug)]
pub struct TypeInfo {
    id: TypeId,
    name: &'static str,
    size: usize,
    align: usize,
    dropper: unsafe fn(*mut u8)
}

impl TypeInfo {
    pub fn new<T: 'static>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            // Safety. It's ok to use unsafe code, because dropper field is an unsafe fn.
            dropper: |ptr| { unsafe { std::ptr::drop_in_place(ptr as *mut T) } },
        }
    }

    pub const fn id(&self) -> &TypeId { &self.id }
    pub const fn name(&self) -> &'static str { &self.name }
    pub const fn size(&self) -> usize { self.size }
    pub const fn align(&self) -> usize { self.align }
    pub unsafe fn drop(&self, ptr: *mut u8) { unsafe { (self.dropper)(ptr) } }
}
