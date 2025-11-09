// #![windows_subsystem = "windows"]

mod components;
mod resources;
mod systems;
mod utils;
mod prefabs;
mod events;

use std::path::PathBuf;

use crate::{
    components::*,
    resources::*,
    systems::*,
    utils::*,
    events::*,
};

use fruits_engine::prelude::*;

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
        build_app(&args[2]);
    }

    if args[1].as_str() == "run" {
        todo!();
    }

    if args[1].as_str() == "edit" {
        // todo: pass project path
        run_editor_app();
    }
}

fn build_app(project_path: &str) {
    let project_name = PathBuf::from(project_path);
    let project_name = project_name.file_name().unwrap().to_str().unwrap();

    {
        let mut path = PathBuf::from(project_path);
        path.push("scripts/Cargo.toml");

        let _status = std::process::Command::new("cargo")
            .args(["build", "--manifest-path", path.to_str().unwrap()])
            .stderr(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .status().expect("failed to execute process");

        if !_status.success() {
            panic!("unsuccessful exit");
        }
    }

    {
        let mut src_path = PathBuf::from(project_path);
        src_path.push("scripts");
        src_path.push("target");
        src_path.push("debug");
        src_path.push(format!("{}.dll", project_name));

        
        let mut dst_path = PathBuf::from(project_path);
        dst_path.push("builds");

        let dst_dir_path = dst_path.clone();

        dst_path.push("app.dll");

        _ = std::fs::create_dir(dst_dir_path);
        std::fs::copy(src_path, dst_path).unwrap();
    }

    //

    {
        let mut path = PathBuf::from(project_path);
        path.push("launcher");

        if !std::fs::exists(&path).unwrap_or(true) {
            std::fs::create_dir(&path).unwrap();

            let mut cargo_path = path.clone();
            cargo_path.push("Cargo.toml");

            std::fs::write(cargo_path, r#"
[package]
name = "launcher"
version = "0.1.0"
edition = "2024"

[dependencies]
fruits_engine = { git = "https://github.com/unknownMusician/fruits_engine.git", branch = "wip/project-editor" }
            "#.trim()).unwrap();
            
            let mut src_path = path.clone();
            src_path.push("src");

            std::fs::create_dir(&src_path).unwrap();
            
            let mut main_path = src_path.clone();
            main_path.push("main.rs");

            std::fs::write(main_path, r#"
fn main() {
    fruits_engine::app::launch_app();
}
            "#.trim()).unwrap()
        }
    }

    {
        let mut path = PathBuf::from(project_path);
        path.push("launcher/Cargo.toml");

        let _status = std::process::Command::new("cargo")
            .args(["build", "--manifest-path", path.to_str().unwrap()])
            .stderr(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .status().expect("failed to execute process");

        if !_status.success() {
            panic!("unsuccessful exit");
        }
    }

    {
        let mut src_path = PathBuf::from(project_path);
        src_path.push("launcher");
        src_path.push("target");
        src_path.push("debug");
        src_path.push("launcher.exe");

        
        let mut dst_path = PathBuf::from(project_path);
        dst_path.push("builds");

        let dst_dir_path = dst_path.clone();

        dst_path.push(format!("{}.exe", project_name));

        _ = std::fs::create_dir(dst_dir_path);
        std::fs::copy(src_path, dst_path).unwrap();
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

        update.insert_system(update_project_window_content_system);

        update.group(SYSTEM_GROUP)
            .insert_child_system(prepare_ui_raycast_system)
            .insert_child_system(check_button_system)
            .insert_child_system(select_file_system)
            .insert_child_system(update_project_entry_selection_system)
            .insert_child_system(inspect_file_system);

        update.order_system(update_project_window_content_system).before_group(SYSTEM_GROUP_TRANSFORM);
        update.order_group(SYSTEM_GROUP).before_group(SYSTEM_GROUP_RENDER);
        update.order_group(SYSTEM_GROUP_TRANSFORM).before_group(SYSTEM_GROUP);
        update.order_system(update_project_window_content_system).before_system(prepare_ui_raycast_system);
        update.order_system(prepare_ui_raycast_system).before_system(check_button_system);
        update.order_system(check_button_system).before_system(select_file_system);
        update.order_system(select_file_system).before_system(update_project_entry_selection_system);
    }

    app.run();
}

fn init_system(
    mut world: ExclusiveWorldAccess,
) {
    let (mut res, ec, evt) = world.as_tuple_mut();

    let standard_render_assets_res = res.get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();
    let texture_text = standard_render_assets_res.texture_text_px_8_8.clone();

    let materials_res = res.get_mut::<AssetStorageResource<StandardMaterial>>().unwrap();

    let material_panel = materials_res.insert(StandardMaterial {
        is_lit: false,
        space: RenderSpace::Window,
        color: Vec4::splat(1.0),
        color_tex: None,
        alpha_threshold: Some(0.5),
        ..Default::default()
    });

    let material_text = materials_res.insert(StandardMaterial {
        is_lit: false,
        space: RenderSpace::Window,
        color: Vec4::splat(1.0),
        color_tex: Some(texture_text),
        alpha_threshold: Some(0.5),
        ..Default::default()
    });

    res.insert(UiInteractionResource::default()).ok().unwrap();
    res.insert(UiRaycastResource::default()).ok().unwrap();
    res.insert(SelectedFileResource::default()).ok().unwrap();
    res.insert(InspectedFileResource::default()).ok().unwrap();

    res.insert(StandardAssetsResource {
        material_panel: material_panel.clone(),
        material_text: material_text.clone(),
    }).ok().unwrap();

    prefabs::project_window(world.as_mut());
    prefabs::scene_window(world.as_mut());
}
//