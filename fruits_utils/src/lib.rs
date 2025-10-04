pub mod graph;
pub mod index_version_collection;
pub mod thread_pool;
pub mod typed_map;
pub mod same_type;
pub mod morph_vec;
pub mod mem;
pub mod stack_vec;
pub mod exec_on_drop;
mod bit_array;
pub mod types_registry;

pub struct AssumeSend<T>(T);

impl<T> AssumeSend<T> {
    pub unsafe fn new(v: T) -> Self {
        Self(v)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

unsafe impl<T> Send for AssumeSend<T> { }