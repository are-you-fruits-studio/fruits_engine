use fruits_app::App;
use fruits_ecs::{AppInitCtxFfi, WorldBuilderMut};

pub fn launch_app_statically(f: impl FnOnce(WorldBuilderMut)) {
    let mut app = App::new();

    fruits_modules::add_defult_modules_to(app.ecs_mut().as_mut());

    f(app.ecs_mut().as_mut());

    app.run();
}

pub fn launch_app_dynamically() {
    // library_filename: Windows → "app_lib.dll", Linux → "libapp_lib.so"
    // On Linux the dynamic linker doesn't search the exe dir, so we resolve explicitly.
    let lib_filename = libloading::library_filename("app_lib");
    let lib_path = if cfg!(windows) {
        std::path::PathBuf::from(lib_filename)
    } else {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        exe_dir.join(lib_filename)
    };
    let lib = unsafe { libloading::Library::new(&lib_path).unwrap() };

    {
        let init_app_symbol = unsafe { lib.get::<unsafe extern "C" fn(AppInitCtxFfi)>(b"fruits_entry_point").unwrap() };

        let mut app = App::new();

        {
            fruits_modules::add_defult_modules_to(app.ecs_mut().as_mut());
        }

        unsafe {
            let (world_builder_ffi, types) = app.ecs_mut().into_raw_parts();

            let types = types.registry();

            let ctx = AppInitCtxFfi {
                world_mut: &raw mut *world_builder_ffi,
                types_ref: &raw const *types,
            };

            init_app_symbol(ctx);
        }

        app.run();
    }

    lib.close().unwrap();
}
