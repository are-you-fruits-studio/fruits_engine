use crate::{features::{inspector_window::InspectorWindowComponent, scroll::ScrollHandleAreaComponent}, *};

pub fn inspector_window(mut world: WorldDataMut) -> EntityId {
    let (res, mut ec, evt) = world.as_tuple_mut();

    let assets_res = res.get::<StandardAssetsResource>().unwrap();

    let font = assets_res.font.clone();

    let standard_assets_res = res.get::<StandardAssetsResource>().unwrap();

    let material_panel = standard_assets_res.material_panel.clone();
    let material_text = standard_assets_res.material_text.clone();

    let ent_root = ec.create_entity();
    let ent_bordered_root = ec.create_entity();
    let ent_header = ec.create_entity();
    let ent_header_text = ec.create_entity();
    let ent_scroll = ec.create_entity();
    let ent_scroll_view = ec.create_entity();
    let ent_scroll_handle_area = ec.create_entity();
    let ent_scroll_handle = ec.create_entity();
    let ent_scroll_content = ec.create_entity();
    let ent_scroll_content_asset_type = ec.create_entity();
    let ent_scroll_content_asset_container = ec.create_entity();

    EntityComponentsBuilder::new(ec.as_mut(), ent_root)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            anchor: Vec2::new(1.0, 0.5),
            pivot: Vec2::new(1.0, 0.5),
            scale: Vec2::new(Some(UiVal::pd(0.333)).into(), Some(UiVal::pd(1.0)).into()),
            ..Default::default()
        })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            ..Default::default()
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_bordered_root)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            parent_padding_min: Vec2::splat(UiVal::px(1.0)),
            parent_padding_max: Vec2::splat(UiVal::px(1.0)),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_root })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#adadadff").unwrap()),
            ..Default::default()
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_header)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            anchor: Vec2::new(0.5, 0.0),
            pivot: Vec2::new(0.5, 0.0),
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), Some(UiVal::px(20.0)).into()),
            ..Default::default()
        })
        .add_component(ChildComponent {
            parent: ent_bordered_root,
        })
        .add_component(ChildrenRectMaskComponent)
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#929292ff").unwrap()),
            ..Default::default()
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_header_text)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            parent_padding_min: Vec2::splat(UiVal::px(1.0)),
            parent_padding_max: Vec2::splat(UiVal::px(1.0)),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_header })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_text.clone(),
        })
        .add_component(TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            font: font.clone(),
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text: String::from("Inspector").into(),
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            parent_padding_min: Vec2::new(UiVal::px(0.0), UiVal::px(20.0)),
            ..Default::default()
        })
        .add_component(ChildComponent {
            parent: ent_bordered_root,
        })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#757575ff").unwrap()),
            ..Default::default()
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll_handle_area)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            anchor: Vec2::new(1.0, 0.5),
            pivot: Vec2::new(1.0, 0.5),
            scale: Vec2::new(UiVal::px(20.0).into(), UiVal::pd(1.0).into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_scroll })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#686868ff").unwrap()),
            ..Default::default()
        })
        .add_component(ButtonComponent)
        .add_component(ScrollHandleAreaComponent {
            content: ent_scroll_content,
            handle: ent_scroll_handle,
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll_handle)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(100.0).into()),
            ..Default::default()
        })
        .add_component(ChildComponent {
            parent: ent_scroll_handle_area,
        })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#bebebeff").unwrap()),
            ..Default::default()
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll_view)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            parent_padding_max: Vec2::new(UiVal::px(20.0), UiVal::px(0.0)),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_scroll })
        .add_component(ChildrenRectMaskComponent)
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#575757ff").unwrap()),
            ..Default::default()
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll_content)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            anchor: Vec2::new(0.0, 0.0),
            pivot: Vec2::new(0.0, 0.0),
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), None.into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_scroll_view })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        })
        .add_component(ParentComponent { children: vec![ent_scroll_content_asset_type, ent_scroll_content_asset_container].into() })
        .add_component(InspectorWindowComponent {
            asset_type_text: ent_scroll_content_asset_type,
            content_container: ent_scroll_content_asset_container,
        });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll_content_asset_type)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(20.0).into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_scroll_content })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent { material: material_text })
        .add_component(TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            font,
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text: String::new().into(),
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        })
        .add_component(ParentComponent { children: vec![].into() });

    EntityComponentsBuilder::new(ec.as_mut(), ent_scroll_content_asset_container)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            anchor: Vec2::new(0.0, 0.0),
            pivot: Vec2::new(0.0, 0.0),
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), None.into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_scroll_content })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        })
        .add_component(ParentComponent { children: vec![].into() });

    ent_root
}
