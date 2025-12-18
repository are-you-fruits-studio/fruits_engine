extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn fruits_entry_point(item: TokenStream) -> TokenStream {
    let fn_name = item.to_string();
    // let fn_name = fn_name.chars().filter(|c| !c.is_whitespace()).collect::<String>();

    format!(
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fruits_entry_point(ctx: ::fruits_engine::AppInitCtxFfi) {{
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
