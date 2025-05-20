use std::{alloc::Layout, sync::{LazyLock, Mutex}};

pub const CHUNK_SIZE: usize = 1024 * 12;

pub fn chunk_layout() -> Layout {
    static CHUNK_LAYOUT: LazyLock<Layout> = LazyLock::new(|| Layout::from_size_align(CHUNK_SIZE, 1).unwrap());

    *CHUNK_LAYOUT
}

pub struct ChunkNew(*mut u8);

unsafe impl Send for ChunkNew { }
unsafe impl Sync for ChunkNew { }

impl ChunkNew {
    pub fn new() -> Self {
        // Safety. Unsafe for access - missing lifetimes. But leak-safe - has Drop impl.
        unsafe {
            ChunkNew(std::alloc::alloc(chunk_layout()))
        }
    }

    /// Safety. Unsafe for access - missing lifetimes. But leak-safe - has Drop impl.
    pub unsafe fn get(&self) -> *mut u8 {
        self.0
    }
}
impl Drop for ChunkNew {
    fn drop(&mut self) {
        // Safety. Unsafe for access - missing lifetimes. But leak-safe.
        unsafe {
            std::alloc::dealloc(self.0, chunk_layout());
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ArchetypeItemPhysicalLocation {
    pub chunk_index: usize,
    pub memory_offset: usize,
    pub memory_size: usize,
    pub memory_align: usize,
}

pub struct UnsafeArchetype {
    chunks: Mutex<Vec<ChunkNew>>,
}

unsafe impl Send for UnsafeArchetype { }
unsafe impl Sync for UnsafeArchetype { }

impl UnsafeArchetype {
    pub fn new() -> Self {
        Self {
            chunks: Mutex::new(Vec::new()),
        }
    }

    /// Safety. Memory is deallocated automatically, but there's no lifetime-checks.
    pub unsafe fn get_memory(&self, location: &ArchetypeItemPhysicalLocation) -> (*mut u8, usize) {
        let chunk = &self.chunks.lock().unwrap()[location.chunk_index];

        // Safety. Safe, because all the offsets are precalculated.
        unsafe {
            let chunk_ptr = chunk.get();

            let mut memory_ptr = chunk_ptr.add(location.memory_offset);

            memory_ptr = memory_ptr.add(memory_ptr.align_offset(location.memory_align));

            let memory_addr = memory_ptr as usize;
            let chunk_addr = chunk_ptr as usize;

            if (memory_addr < chunk_addr) || (memory_addr + location.memory_size - 1 > chunk_addr + CHUNK_SIZE) {
                panic!("fruits: memory out of bounds access");
            }

            (memory_ptr, location.memory_size)
        }
    }

    pub fn chunks_count(&self) -> usize {
        self.chunks.lock().unwrap().len()
    }

    /// Safety. Structural changes can only be made when no references to chunks exist.
    pub unsafe fn push_chunk(&mut self) {
        self.chunks.lock().unwrap().push(ChunkNew::new());
    }

    /// Safety. Structural changes can only be made when no references to chunks exist.
    pub unsafe fn pop_chunk(&mut self) {
        self.chunks.lock().unwrap().pop();
    }
}