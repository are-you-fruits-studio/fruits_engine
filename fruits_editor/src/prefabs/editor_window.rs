use fruits_engine::*;

pub fn editor_window(mut world: WorldDataMut) -> EntityId {
    let (res, mut ent, evt) = world.as_tuple_mut();

    const HEADER_HEIGHT: f32 = 20.0;

    let ent_root = ent.create_entity();
    let ent_header = ent.create_entity();
    let ent_dock = ent.create_entity();

    ent.set_components(ent_root, (
        GlobalRectComponent::default(),
        LocalRectComponent::default(),
        ParentComponent {
            children: vec![ent_header, ent_dock].into(),
        },
    ));

    ent.set_components(ent_header, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            anchor: Vec2::new(0.5, 0.0),
            pivot: Vec2::new(0.5, 0.0),
            scale: Vec2::new(UiVal::pd(1.0).into(), UiVal::px(HEADER_HEIGHT).into()),
            ..Default::default()
        },
        ChildComponent {
            parent: ent_root,
        },
        RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.5),
            direction: UiDirection::Horizontal,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
        },
    ));

    ent.set_components(ent_dock, (
        GlobalRectComponent::default(),
        LocalRectComponent {
            parent_padding_min: Vec2::new(UiVal::px(0.0), UiVal::px(HEADER_HEIGHT)),
            ..Default::default()
        },
        ChildComponent {
            parent: ent_root,
        },
    ));

    ent_root
}