// #![windows_subsystem = "windows"]

mod components;
mod features;
mod prefabs;
mod resources;
mod systems;
mod utils;

use std::path::PathBuf;

use crate::{
    components::*,
    features::{project_window_selection::select_file_system, ui_interaction::*},
    resources::*,
    systems::*,
    utils::*,
};

use fruits_engine::*;

mod app_building;

const SYSTEM_GROUP: &'static str = "fruits_editor";

fn main() {
    let args = std::env::args()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    if args.len() < 3 {
        return;
    }

    if args[1].as_str() == "build" {
        app_building::build_app(&args[2]);
    }

    if args[1].as_str() == "run" {
        todo!();
    }

    if args[1].as_str() == "edit" {
        // todo: pass project path
        run_editor_app();
    }
}

fn run_editor_app() {
    let mut app = App::new();

    let world = app.ecs_mut();

    add_defult_modules_to(world.as_mut());

    let mut world_behavior = world.behavior_mut();

    {
        let mut start = world_behavior.get_mut(Schedule::Start);

        start.insert_system(init_system);
    }

    {
        let mut update = world_behavior.get_mut(Schedule::Update);

        update.group(SYSTEM_GROUP).insert_child_system(update_scene_entries_system);

        update.order_group(SYSTEM_GROUP).before_group(SYSTEM_GROUP_RENDER);
        update.order_group(SYSTEM_GROUP).before_group(SYSTEM_GROUP_TRANSFORM);
        update
            .order_system(select_file_system)
            .before_system(update_scene_entries_system);
    }

    features::scroll::register_feature(world.as_mut());
    features::ui_interaction::register_feature(world.as_mut());
    features::project_window_entries::register_feature(world.as_mut());
    features::project_window_selection::register_feature(world.as_mut());
    features::inspector_window::register_feature(world.as_mut());
    features::input_field::register_feature(world.as_mut());
    features::dropdown::register_feature(world.as_mut());
    features::project_window_parsing::register_feature(world.as_mut());
    features::project_window_saving::register_feature(world.as_mut());
    features::serialization::register_feature(world.as_mut());

    app.run();
}

fn init_system(mut world: ExclusiveWorldAccess) {
    let (mut res, ec, evt) = world.as_tuple_mut();

    let standard_render_assets_res = res.get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_12.clone();
    let texture_text = standard_render_assets_res.texture_text_px_8_8.clone();

    let materials_res = res.get_mut::<AssetStorageResource<StandardMaterial>>().unwrap();

    let material_panel = materials_res.insert(StandardMaterial {
        is_lit: false,
        space: RenderSpace::Window,
        color: Vec4::splat(1.0),
        color_tex: None.into(),
        alpha_threshold: Some(0.5).into(),
        ..Default::default()
    });

    let material_text = materials_res.insert(StandardMaterial {
        is_lit: false,
        space: RenderSpace::Window,
        color: Vec4::splat(1.0),
        color_tex: Some(texture_text).into(),
        alpha_threshold: Some(0.5).into(),
        ..Default::default()
    });

    res.insert(UiInteractionResource::default()).ok().unwrap();

    res.insert(StandardAssetsResource {
        material_panel: material_panel.clone(),
        material_text: material_text.clone(),
        font,
    })
    .ok()
    .unwrap();

    prefabs::project_window(world.as_mut());
    prefabs::scene_window(world.as_mut());
    prefabs::inspector_window(world.as_mut());
}
//
