mod bit_array;
pub mod exec_on_drop;
pub mod graph;
pub mod index_version_collection;
pub mod mem;
pub mod morph_vec;
pub mod same_type;
pub mod stack_vec;
pub mod thread_pool;
pub mod typed_map;
pub mod close_int_map;

pub struct AssumeSend<T>(T);

impl<T> AssumeSend<T> {
    pub unsafe fn new(v: T) -> Self {
        Self(v)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

unsafe impl<T> Send for AssumeSend<T> {}
