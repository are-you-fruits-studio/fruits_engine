use std::{fmt::Write, time::Instant};

use fruits_engine::prelude::{fps_counter::FpsResource, *};

fn main() {
    let mut app = App::new();
    let world = app.ecs_mut();

    add_defult_modules_to(world);
    fps_counter::add_module_to(world);

    world.behavior_mut().get_mut(Schedule::Start).add_system(init);
    world.behavior_mut().get_mut(Schedule::Update).add_system(update_time);
    world.behavior_mut().get_mut(Schedule::Update).add_system(write_time_into_text);

    world.behavior_mut().get_mut(Schedule::Start).order_group(SYSTEM_GROUP_RENDER).before_system(init);

    let world_data = world.data_mut();

    let ec = world_data.entities_components_mut();

    let entity = ec.create_entity();
    ec.add_component(entity, GlobalTransform {
        scale_rotation: Mat::IDENTITY,
        position: Vec3::new(0.0_f32, 0.0_f32, -1.0f32),
    }).ok().unwrap();
    ec.add_component(entity, CameraComponent {
        near: 0.1_f32,
        far: 1_000_f32,
        fov: 90_f32.to_radians(),
    }).ok().unwrap();

    println!("start");
    app.run();
    println!("end");
}

#[derive(Resource)]
struct TimeResource {
    pub time: f32,
    pub frame: u64,
    pub start: Option<Instant>,
}

fn init(mut world: ExclusiveWorldAccess) {
    world.resources_mut().insert(TimeResource { time: 0.0_f32, frame: 0, start: None }).ok().unwrap();

    let standard_render_assets = world.resources().get::<StandardRenderAssetsResource>().unwrap();

    let texture_text = standard_render_assets.texture_text_px_5_7.clone();
    let font = standard_render_assets.font_px_5_7.clone();

    let material = StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::World,
        color_tex: Some(texture_text.clone()),
        ..Default::default()
    });

    let material = world.resources_mut().get_mut::<AssetStorageResource::<StandardMaterial>>().unwrap().insert(material);
    
    let ec = world.entities_components_mut();

    let ent_text = ec.create_entity();
    ec.add_component(ent_text, GlobalTransform::IDENTITY).ok().unwrap();
    ec.add_component(ent_text, LocalTransform {
        position: Vec3::new(0.0, 0.0, 0.0),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent_text, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent_text, StandardMaterialComponent { material: material.clone(), }).ok().unwrap();
    ec.add_component(ent_text, TextComponent {
        font: font.clone(),
        font_size: UiVal::Px(0.4),
        text: String::from("Dashunia - myla pinhvinka"),
        horizontal_align: HorizontalAlign::Middle,
        vertical_align: VerticalAlign::Middle,
        is_y_inverted: false,
        horizontal_spacing: 0.2,
        color: Vec4::splat(1.0),
    }).ok().unwrap();
}

fn write_time_into_text(
    fps_res: Res<FpsResource>,
    time: Res<TimeResource>,
    mut text_q: WorldQuery<&mut TextComponent>
) {
    let fps = (fps_res.last_frame_times_s().len() as f64 / fps_res.last_frame_times_s().iter().copied().sum::<f64>()).round() as usize;

    for text_c in text_q.iter_mut() {
        text_c.text.clear();

        write!(text_c.text, "{} | {:.2}", fps, time.time).unwrap();
    }
}

fn update_time(
    mut time: ResMut<TimeResource>,
) {
    let start = match time.start {
        Some(start) => start,
        None => {
            let new_start = Instant::now();
            time.start = Some(new_start);
            time.frame = 0;
            new_start
        },
    };

    time.frame += 1;
    time.time = start.elapsed().as_secs_f32();
}