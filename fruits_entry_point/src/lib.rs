//! # fruits_entry_point
//!
//! Connects a game built as a dynamic library to the engine launcher that loads
//! it, so the launcher can hand a freshly built world to the game's setup code.
//!
//! # How to use
//!
//! #### Export a setup function to the launcher
//!
//! Mark the function that populates the world so the launcher can find and call it
//! after loading the game library. The function takes a
//! [`WorldBuilderMut`](https://docs.rs/fruits_ecs) and registers the game's
//! components, systems, and resources.
//!
//! ```ignore
//! use fruits_engine::WorldBuilderMut;
//!
//! fn setup(world: WorldBuilderMut) {
//!     // Register your components, systems, and resources on `world`.
//! }
//!
//! fruits_engine::fruits_entry_point!(setup);
//! ```
//!
//! The launcher binary then loads the library and runs the host with
//! `fruits_engine::launch_app_dynamically()`. Only one `fruits_entry_point!`
//! invocation belongs in a game library, since it defines the single exported
//! symbol the launcher looks up.
//!
//! # How to maintain
//!
//! The crate is a single function-like procedural macro. It takes the caller's
//! argument verbatim as the name of the setup function and emits a fixed
//! `#[unsafe(no_mangle)] pub unsafe extern "C-unwind" fn fruits_entry_point` with C ABI.
//! That exported symbol is the contract with `fruits_app_launcher`, which resolves
//! it by the literal name `fruits_entry_point` and calls it with an
//! `AppInitCtxFfi`.
//!
//! Inside the generated function, the two raw pointers carried by `AppInitCtxFfi`
//! (`world_mut` and `types_ref`) are dereferenced back into references, the type
//! registry is wrapped in a `TypesRegistryCache`, and a safe `WorldBuilderMut` is
//! reconstructed from the world and registry. The caller's setup function is then
//! invoked with that builder inside an inner scope, so the builder's borrow ends
//! before the function returns. All of these names (`AppInitCtxFfi`,
//! `TypesRegistryCache`, `WorldBuilderMut`) are resolved through `::fruits_engine`
//! in the expanded code, so a game crate must depend on `fruits_engine` for the
//! macro output to compile.
//!
//! Caveats for maintainers: the macro performs no validation — it pastes the
//! argument string directly into a call expression, so a malformed argument
//! surfaces only as an error in the expanded code. The generated symbol name, ABI,
//! and `AppInitCtxFfi` layout must stay in lockstep with the lookup in
//! [`launch_app_dynamically`](https://docs.rs/fruits_app_launcher); changing either
//! side alone breaks dynamic loading at runtime rather than at compile time.

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn fruits_entry_point(item: TokenStream) -> TokenStream {
    let fn_name = item.to_string();
    // let fn_name = fn_name.chars().filter(|c| !c.is_whitespace()).collect::<String>();

    format!(
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn fruits_entry_point(ctx: ::fruits_engine::AppInitCtxFfi) {{
    let world = unsafe {{ &mut *ctx.world_mut }};
    let types_ref = unsafe {{ &*ctx.types_ref }};

    let types = ::fruits_engine::TypesRegistryCache::new(types_ref.clone());

    {{
        let world = ::fruits_engine::WorldBuilderMut::new(world, &types);

        let _init_result: () = {{ {fn_name}(world) }};
    }}
}}
    "#
    )
    .parse()
    .unwrap()
}
