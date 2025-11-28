macro_rules! mod_and_pub_use {
    ($i: ident) => {
        mod $i;
        pub use $i::*;
    };
}

mod_and_pub_use!(debug_layout_system);
mod_and_pub_use!(update_scene_entries_system);
