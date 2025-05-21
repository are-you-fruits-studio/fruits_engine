pub fn as_bytes<T>(slice: &[T]) -> &[u8] {
    // Safety. It gives readonly access and all bit variations of u8 are valid.
    unsafe { slice.align_to::<u8>().1 }
}