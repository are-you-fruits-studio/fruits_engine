use fruits_engine::prelude::*;

fn main() {
    let mut app = App::new();
    let world = app.ecs_mut();

    add_defult_modules_to(world);
    fps_counter::add_module_to(world);

    world.behavior_mut().get_mut(Schedule::Start).add_system(init);

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

fn init(mut world: ExclusiveWorldAccess) {
    let standard_render_assets = world.resources().get::<StandardRenderAssetsResource>().unwrap();

    let texture_text = standard_render_assets.texture_text_px_8_12.clone();
    let font = standard_render_assets.font_px_8_12.clone();

    let texture_white = standard_render_assets.texture_white.clone();

    let material_white = world.resources_mut().get_mut::<AssetStorageResource::<StandardMaterial>>().unwrap().insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color_tex: Some(texture_white.clone()),
        color: Vec4::splat(1.0),
        alpha_threshold: 0.5,
    }));
    
    let material_text = world.resources_mut().get_mut::<AssetStorageResource::<StandardMaterial>>().unwrap().insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color_tex: Some(texture_text.clone()),
        color: Vec4::splat(1.0),
        alpha_threshold: 0.5,
    }));
    
    let ec = world.entities_components_mut();

    let ent1 = ec.create_entity();
    let ent2 = ec.create_entity();
    let ent3 = ec.create_entity();
    let ent4 = ec.create_entity();

    ec.add_component(ent1, ParentComponent { children: vec![ent2].into(), }).ok().unwrap();
    ec.add_component(ent1, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent1, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent1, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent1, LocalRectComponent {
        parent_padding_min: Vec2::new(UiVal::pw(0.5), UiVal::ph(0.5)),
        parent_padding_max: Vec2::new(UiVal::pw(0.5), UiVal::ph(0.5)),
        z: 0.0,
        ..Default::default()
    }).ok().unwrap();

    ec.add_component(ent2, ChildComponent { parent: ent1, }).ok().unwrap();
    ec.add_component(ent2, ParentComponent { children: vec![ent3].into(), }).ok().unwrap();
    ec.add_component(ent2, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent2, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent2, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent2, LocalRectComponent {
        parent_padding_min: Vec2::new(UiVal::px(-100.0), UiVal::px(-100.0)),
        parent_padding_max: Vec2::new(UiVal::px(-100.), UiVal::px(-100.0)),
        z: -10.0,
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent2, ImageComponent::default()).ok().unwrap();
    ec.add_component(ent2, ChildrenRectMaskComponent).ok().unwrap();

    ec.add_component(ent3, ChildComponent { parent: ent2, }).ok().unwrap();
    ec.add_component(ent3, ParentComponent { children: vec![ent4].into(), }).ok().unwrap();
    ec.add_component(ent3, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent3, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent3, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent3, LocalRectComponent {
        parent_padding_min: Vec2::new(UiVal::px(10.0), UiVal::px(10.0)),
        parent_padding_max: Vec2::new(UiVal::vmin(0.1), UiVal::vmin(0.1)),
        z: -10.0,
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent3, ImageComponent {
        color: Vec4::splat(0.6),
        ..Default::default()
    }).ok().unwrap();

    ec.add_component(ent4, ChildComponent { parent: ent3, }).ok().unwrap();
    ec.add_component(ent4, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent4, StandardMaterialComponent { material: material_text.clone(), }).ok().unwrap();
    ec.add_component(ent4, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent4, LocalRectComponent {
        z: -10.0,
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent4, TextComponent {
        font: font.clone(),
        font_size: UiVal::vh(0.15),
        text: String::from("Dashunia - myla pinhvinka"),
        horizontal_align: HorizontalAlign::Middle,
        vertical_align: VerticalAlign::Bottom,
        is_y_inverted: true,
        horizontal_spacing: 0.0,
        color: Vec4::new(1.0, 0.0, 1.0, 1.0),
    }).ok().unwrap();

    //

    let ent10 = ec.create_entity();
    let ent11 = ec.create_entity();
    let ent12 = ec.create_entity();
    let ent13 = ec.create_entity();

    ec.add_component(ent10, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent10, LocalRectComponent {
        pivot: Vec2::splat(0.0),
        anchor: Vec2::splat(0.0),
        scale: Vec2::new(None, None),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent10, RectChildAlignComponent {
        direction: UiDirection::Vertical,
        anchor: Vec2::new(0.5, 0.5),
        min_gap: UiVal::vd(0.1),
        spacing: UiSpacing::Chunk,
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent10, ParentComponent { children: vec![ent11, ent12, ent13].into() }).ok().unwrap();
    ec.add_component(ent10, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent10, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent10, ImageComponent {
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        ..Default::default()
    }).ok().unwrap();

    ec.add_component(ent11, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent11, LocalRectComponent {
        scale: Vec2::splat(Some(UiVal::px(50.0))),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent11, ChildComponent { parent: ent10 }).ok().unwrap();
    ec.add_component(ent11, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent11, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent11, ImageComponent {
        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        ..Default::default()
    }).ok().unwrap();

    ec.add_component(ent12, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent12, LocalRectComponent {
        scale: Vec2::splat(Some(UiVal::px(40.0))),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent12, ChildComponent { parent: ent10 }).ok().unwrap();
    ec.add_component(ent12, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent12, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent12, ImageComponent {
        color: Vec4::new(0.0, 1.0, 0.0, 1.0),
        ..Default::default()
    }).ok().unwrap();

    ec.add_component(ent13, GlobalRectComponent::default()).ok().unwrap();
    ec.add_component(ent13, LocalRectComponent {
        scale: Vec2::splat(Some(UiVal::px(70.0))),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent13, ChildComponent { parent: ent10 }).ok().unwrap();
    ec.add_component(ent13, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent13, StandardMaterialComponent { material: material_white.clone(), }).ok().unwrap();
    ec.add_component(ent13, ImageComponent {
        color: Vec4::new(0.0, 0.0, 1.0, 1.0),
        ..Default::default()
    }).ok().unwrap();
}