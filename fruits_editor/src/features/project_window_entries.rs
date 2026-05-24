use crate::*;

pub fn register_feature(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(ProjectWindowCache::default())
        .ok()
        .unwrap();

    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(update_project_window_content_system);

    update
        .order_system(update_project_window_content_system)
        .before_system(prepare_ui_raycast_system);
}

#[derive(Resource, Debug, Default)]
pub struct ProjectWindowCache {
    pub dir_entry: ProjectWindowDataEntry,
}

pub fn update_project_window_content_system(mut world: ExclusiveWorldAccess) {
    let (mut res, mut ent, evt) = world.as_tuple_mut();

    let assets = res.get::<StandardAssetsResource>().unwrap().clone();

    let standard_render_assets_res = res.get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();

    let contents = ent
        .query_filtered::<Entity, WithFilter<ProjectWindowContentComponent>>()
        .iter()
        .collect::<Vec<_>>();

    let Ok(current_dir) = std::env::current_dir() else {
        return;
    };

    let cache = res.get_mut::<ProjectWindowCache>().unwrap();

    let entry = ProjectWindowDataEntry::scan(&current_dir);

    if cache.dir_entry == entry {
        return;
    }

    cache.dir_entry = entry.clone();

    for content in contents {
        destroy_entity_children(ent.as_mut(), content);

        for entry in &entry.children {
            spawn_project_window_entries(
                ent.as_mut(),
                &assets.material_text,
                &assets.material_panel,
                &font,
                content,
                entry,
            );
        }
    }
}

fn spawn_project_window_entries(
    mut ec: EntitiesHolderMut,
    material_text: &AssetHandle<StandardMaterial>,
    material_panel: &AssetHandle<StandardMaterial>,
    font: &AssetHandle<Font>,
    parent: Entity,
    entry: &ProjectWindowDataEntry,
) {
    let ent_entry = ec.create_entity();
    let ent_name_container = ec.create_entity();
    let ent_name = ec.create_entity();

    EntityComponentsBuilder::new(ec.as_mut(), ent_entry)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), None.into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent })
        .add_component(ParentComponent { children: vec![].into() })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(parent)
        .unwrap()
        .children
        .push(ent_entry);

    EntityComponentsBuilder::new(ec.as_mut(), ent_name_container)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), Some(UiVal::px(20.0)).into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_entry })
        .add_component(ParentComponent { children: vec![].into() })
        .add_component(ProjectWindowEntryComponent {
            path: entry.path.clone(),
        })
        .add_component(BatchedMeshComponent::default())
        .add_component(ButtonComponent)
        .add_component(StandardMaterialComponent {
            material: material_panel.clone(),
        })
        .add_component(ImageComponent {
            color: Vec4::splat(0.0),
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(ent_entry)
        .unwrap()
        .children
        .push(ent_name_container);

    EntityComponentsBuilder::new(ec.as_mut(), ent_name)
        .add_component(DebugNameComponent(String::from("ent_name")))
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent { ..Default::default() })
        .add_component(ChildComponent {
            parent: ent_name_container,
        })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent {
            material: material_text.clone(),
        })
        .add_component(TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            font: font.clone(),
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text: entry.name.clone().into(),
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        });

    if entry.children.is_empty() {
        return;
    }

    let ent_children = ec.create_entity();
    let ent_children_container = ec.create_entity();

    EntityComponentsBuilder::new(ec.as_mut(), ent_children)
        .add_component(DebugNameComponent(String::from("ent_children")))
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), None.into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_entry });
    ec.get_component_mut::<ParentComponent>(ent_entry)
        .unwrap()
        .children
        .push(ent_children);

    EntityComponentsBuilder::new(ec.as_mut(), ent_children_container)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            parent_padding_min: Vec2::new(UiVal::px(20.0), UiVal::px(0.0)),
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), None.into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_children })
        .add_component(ParentComponent { children: vec![].into() })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        });

    for entry in &entry.children {
        spawn_project_window_entries(
            ec.as_mut(),
            material_text,
            material_panel,
            font,
            ent_children_container,
            entry,
        );
    }
}
