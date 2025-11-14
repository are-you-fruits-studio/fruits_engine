use std::{
    alloc::{GlobalAlloc, System},
    sync::atomic::{AtomicUsize, Ordering},
};

#[global_allocator]
static ALLOC: AllocMonitor = AllocMonitor;
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

pub fn allocated() -> usize {
    ALLOCATED.load(Ordering::Relaxed)
}

struct AllocMonitor;

unsafe impl GlobalAlloc for AllocMonitor {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        // Safety. AllocMonitor is just a wrapper.
        // Without it all the allocator calls would be called on the System allocator.
        let result = unsafe { System.alloc(layout) };

        ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);

        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        // Safety. AllocMonitor is just a wrapper.
        // Without it all the allocator calls would be called on the System allocator.
        let result = unsafe { System.dealloc(ptr, layout) };

        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);

        result
    }
}
