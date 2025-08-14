use fruits_prelude::*;
use fruits_modules::{asset::*, render::*, transform::*};

fn main() {
    let mut app = App::new();
    let world = app.ecs_mut();

    fruits_modules::render::add_module_to(world);
    fruits_modules::transform::add_module_to(world);
    fruits_modules::fps_counter::add_module_to(world);

    world.behavior_mut().get_mut(Schedule::Start).add_system(init);
    world.behavior_mut().get_mut(Schedule::Update).add_system(create_test_entities);
    world.behavior_mut().get_mut(Schedule::Update).add_system(destroy_test_entities);

    world.behavior_mut().get_mut(Schedule::Start).order_group(fruits_modules::render::SYSTEM_GROUP).before_system(init);
    world.behavior_mut().get_mut(Schedule::Update).order_group(fruits_modules::transform::SYSTEM_GROUP).before_group(fruits_modules::render::SYSTEM_GROUP);

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

fn init(mut world: ExclusiveWorldAccess) {
    let standard_render_assets = world.resources().get::<StandardRenderAssetsResource>().unwrap();

    let texture_text = standard_render_assets.texture_text_px_8_12.clone();
    let font = standard_render_assets.font_px_8_12.clone();

    let texture_white = standard_render_assets.texture_white.clone();

    let material_white = world.resources_mut().get_mut::<AssetStorageResource::<StandardMaterial>>().unwrap().insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color_tex: Some(texture_white.clone()),
        color: Vec4::with_all(1.0),
        alpha_threshold: 0.5,
    }));
    
    let material_text = world.resources_mut().get_mut::<AssetStorageResource::<StandardMaterial>>().unwrap().insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color_tex: Some(texture_text.clone()),
        color: Vec4::with_all(1.0),
        alpha_threshold: 0.5,
    }));
    
    let ec = world.entities_components_mut();

    let ent1 = ec.create_entity();
    let ent2 = ec.create_entity();
    let ent3 = ec.create_entity();

    ec.add_component(ent1, GlobalTransform::IDENTITY).ok().unwrap();
    ec.add_component(ent1, LocalTransform {
        position: Vec3::new(0.0, 0.0, 0.0),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent1, ParentComponent { children: vec![ent2], }).ok().unwrap();
    ec.add_component(ent1, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent1, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent1, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent1, LocalRectComponent {
        anchor_min: Vec2::new(0.5, 0.5),
        anchor_max: Vec2::new(0.5, 0.5),
        offset_min: Vec2::new(UiVal::Pmin(-0.25), UiVal::Pmin(-0.25)),
        offset_max: Vec2::new(UiVal::Pmin(0.25), UiVal::Pmin(0.25)),
    }).ok().unwrap();
    ec.add_component(ent1, ImageComponent::default()).ok().unwrap();

    ec.add_component(ent2, GlobalTransform::IDENTITY).ok().unwrap();
    ec.add_component(ent2, LocalTransform {
        position: Vec3::new(0.0, 0.0, -10.0),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent2, ChildComponent { parent: ent1, }).ok().unwrap();
    ec.add_component(ent2, ParentComponent { children: vec![ent3], }).ok().unwrap();
    ec.add_component(ent2, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent2, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent2, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent2, LocalRectComponent {
        anchor_min: Vec2::new(0.0, 0.0),
        anchor_max: Vec2::new(1.0, 1.0),
        offset_min: Vec2::new(UiVal::Px(10.0), UiVal::Px(10.0)),
        offset_max: Vec2::new(UiVal::Vmin(-0.1), UiVal::Vmin(-0.1)),
    }).ok().unwrap();
    ec.add_component(ent2, ImageComponent {
        color: Vec4::with_all(0.6),
        ..Default::default()
    }).ok().unwrap();

    ec.add_component(ent3, GlobalTransform::IDENTITY).ok().unwrap();
    ec.add_component(ent3, LocalTransform {
        position: Vec3::new(0.0, 0.0, -10.0),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent3, ChildComponent { parent: ent2, }).ok().unwrap();
    ec.add_component(ent3, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent3, StandardMaterialComponent { material: material_text.clone(), }).ok().unwrap();
    ec.add_component(ent3, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent3, LocalRectComponent {
        anchor_min: Vec2::new(0.0, 0.0),
        anchor_max: Vec2::new(1.0, 1.0),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent3, TextComponent {
        font: font.clone(),
        font_size: UiVal::Ph(0.1),
        text: String::from("Dashunia - myla pinhvinka"),
        horizontal_align: HorizontalAlign::Middle,
        vertical_align: VerticalAlign::Bottom,
        is_y_inverted: true,
        horizontal_spacing: 0.0,
        color: Vec4::new(1.0, 0.0, 1.0, 1.0),
    }).ok().unwrap();
}

fn create_test_entities(
    mut world: ExclusiveWorldAccess,
) {
    let ec = world.entities_components_mut();

    for _ in 0..100 {
        let ent = ec.create_entity();
        ec.add_component(ent, TestComponent(vec![1, 21, 3])).ok().unwrap();
    }
}

fn destroy_test_entities(
    mut world: ExclusiveWorldAccess,
) {
    let ec = world.entities_components_mut();

    for (ent, num) in ec.query::<(Entity, &TestComponent)>().iter().map(|(e, t)| (e, t.0[1])).collect::<Vec<_>>() {
        ec.destroy_entity(ent);
    }
}


#[derive(Component)]
pub struct TestComponent(Vec<i32>);