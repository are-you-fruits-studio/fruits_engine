use crate::*;

pub fn scene_window(mut world: WorldDataMut) -> Entity {
    let (res, mut ec, evt) = world.as_tuple_mut();

    let standard_render_assets_res = res.get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();
    let texture_text = standard_render_assets_res.texture_text_px_8_8.clone();

    let standard_assets_res = res.get::<StandardAssetsResource>().unwrap();

    let material_panel = standard_assets_res.material_panel.clone();
    let material_text = standard_assets_res.material_text.clone();

    let ent_root = ec.create_entity();
    let ent_bordered_root = ec.create_entity();
    let ent_header = ec.create_entity();
    let ent_header_text = ec.create_entity();
    let ent_scroll = ec.create_entity();
    let ent_scroll_view = ec.create_entity();
    let ent_scroll_handle = ec.create_entity();
    let ent_scroll_content = ec.create_entity();

    EntityComponentsBuilder::new(ec.as_mut(), ent_root)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            anchor: Vec2::new(0.5, 0.5),
            pivot: Vec2::new(0.5, 0.5),
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
            text: String::from("Scene").into(),
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
        .add_component(ParentComponent { children: vec![].into() })
        .add_component(SceneWindowContentComponent);

    ent_root
}
