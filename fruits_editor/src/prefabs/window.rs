use fruits_engine::*;

use crate::{features::{scroll::ScrollHandleAreaComponent, ui_interaction::ButtonComponent}, resources::StandardAssetsResource};

pub struct WindowComponent {
    pub title: EntityId,
    pub content_container: EntityId,
}

pub fn window(mut world: WorldDataMut) -> EntityId {
    let (res, mut ec, evt) = world.as_tuple_mut();

    let assets_res = res.as_ref().get::<StandardAssetsResource>().unwrap();

    let font = assets_res.font.clone();

    let standard_assets_res = res.as_ref().get::<StandardAssetsResource>().unwrap();

    let material_panel = standard_assets_res.material_panel.clone();
    let material_text = standard_assets_res.material_text.clone();

    let ent_root = ec.create_entity();
    let ent_bordered_root = ec.create_entity();
    let ent_header = ec.create_entity();
    let ent_header_text = ec.create_entity();
    let ent_scroll = ec.create_entity();
    let ent_scroll_handle_area = ec.create_entity();
    let ent_scroll_handle = ec.create_entity();
    let ent_scroll_view = ec.create_entity();
    let ent_scroll_content = ec.create_entity();

    ec.as_mut().set_components(ent_root, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            anchor: Vec2::new(0.0, 0.5),
            pivot: Vec2::new(0.0, 0.5),
            scale: Vec2::new(Some(UiVal::pd(0.333)).into(), Some(UiVal::pd(1.0)).into()),
            ..Default::default()
        },
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            ..Default::default()
        },
        WindowComponent {
            content_container: ent_scroll_content,
            title: ent_header_text,
        }
    ));

    ec.as_mut().set_components(ent_bordered_root, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            parent_padding_min: Vec2::splat(UiVal::px(1.0)),
            parent_padding_max: Vec2::splat(UiVal::px(1.0)),
            ..Default::default()
        },
        ChildComponent { parent: ent_root },
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#adadadff").unwrap()),
            ..Default::default()
        },
    ));

    ec.as_mut().set_components(ent_header, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            anchor: Vec2::new(0.5, 0.0),
            pivot: Vec2::new(0.5, 0.0),
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), Some(UiVal::px(20.0)).into()),
            ..Default::default()
        },
        ChildComponent {
            parent: ent_bordered_root,
        },
        ChildrenRectMaskComponent,
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#929292ff").unwrap()),
            ..Default::default()
        },
    ));

    ec.as_mut().set_components(ent_header_text, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            parent_padding_min: Vec2::splat(UiVal::px(1.0)),
            parent_padding_max: Vec2::splat(UiVal::px(1.0)),
            ..Default::default()
        },
        ChildComponent { parent: ent_header },
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_text.clone(),
        },
        TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            font: font.clone(),
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text: String::from("Project").into(),
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        },
    ));

    ec.as_mut().set_components(ent_scroll, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            parent_padding_min: Vec2::new(UiVal::px(0.0), UiVal::px(20.0)),
            ..Default::default()
        },
        ChildComponent {
            parent: ent_bordered_root,
        },
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#757575ff").unwrap()),
            ..Default::default()
        },
    ));

    ec.as_mut().set_components(ent_scroll_handle_area, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            anchor: Vec2::new(1.0, 0.5),
            pivot: Vec2::new(1.0, 0.5),
            scale: Vec2::new(UiVal::px(20.0).into(), UiVal::pd(1.0).into()),
            ..Default::default()
        },
        ChildComponent { parent: ent_scroll },
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#686868ff").unwrap()),
            ..Default::default()
        },
        ButtonComponent,
        ScrollHandleAreaComponent {
            content: ent_scroll_content,
            handle: ent_scroll_handle,
        },
    ));

    ec.as_mut().set_components(ent_scroll_handle, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(100.0).into()),
            ..Default::default()
        },
        ChildComponent {
            parent: ent_scroll_handle_area,
        },
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#bebebeff").unwrap()),
            ..Default::default()
        },
    ));

    ec.as_mut().set_components(ent_scroll_view, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            parent_padding_max: Vec2::new(UiVal::px(20.0), UiVal::px(0.0)),
            ..Default::default()
        },
        ChildComponent { parent: ent_scroll },
        ChildrenRectMaskComponent,
        BatchedMeshComponent::default(),
        StandardMaterialComponent {
            material: material_panel.clone(),
        },
        ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#575757ff").unwrap()),
            ..Default::default()
        },
    ));

    ec.as_mut().set_components(ent_scroll_content, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            anchor: Vec2::new(0.0, 0.0),
            pivot: Vec2::new(0.0, 0.0),
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), None.into()),
            ..Default::default()
        },
        ChildComponent { parent: ent_scroll_view },
        RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        },
        ParentComponent { children: vec![].into() },
    ));

    ent_root
}