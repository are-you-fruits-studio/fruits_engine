mod components;
mod resources;
mod systems;
mod utils;
mod prefabs;
mod events;

use crate::{
    components::*,
    resources::*,
    systems::*,
    utils::*,
    events::*,
};

use fruits_engine::prelude::*;

fn main() {
    let mut app = App::new();

    add_defult_modules_to(app.ecs_mut());

    {
        let start = app.ecs_mut().behavior_mut().get_mut(Schedule::Start);

        start.add_system(init_system);
    }

    {
        let update = app.ecs_mut().behavior_mut().get_mut(Schedule::Update);

        update.add_system(update_project_window_content_system);
        update.add_system(update_project_window_content_system);

        update.order_system(update_project_window_content_system).before_group(SYSTEM_GROUP_TRANSFORM);
    }

    app.run();
}

fn init_system(
    mut world: ExclusiveWorldAccess,
) {
    let (res, ec, evt) = world.as_tuple_mut();

    let standard_render_assets_res = res.get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();
    let texture_text = standard_render_assets_res.texture_text_px_8_8.clone();

    let materials_res = res.get_mut::<AssetStorageResource<StandardMaterial>>().unwrap();

    let material_panel = materials_res.insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color: Vec4::splat(1.0),
        color_tex: None,
        alpha_threshold: 0.5,
    }));

    let material_text = materials_res.insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color: Vec4::splat(1.0),
        color_tex: Some(texture_text),
        alpha_threshold: 0.5,
    }));

    res.insert(UiInteractionResource::default()).ok().unwrap();
    res.insert(UiRaycastResource::default()).ok().unwrap();

    res.insert(StandardAssetsResource {
        material_panel: material_panel.clone(),
        material_text: material_text.clone(),
    }).ok().unwrap();

    prefabs::project_window(&mut world);
    prefabs::scene_window(&mut world);
}
//