use crate::*;

pub fn update_project_window_content_system(
    mut world: ExclusiveWorldAccess,
) {
    let assets = world.resources().get::<StandardAssetsResource>().unwrap().clone();

    let standard_render_assets_res = world.resources().get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();

    let contents = world.entities_components().query_filtered::<Entity, WithFilter<ProjectWindowContentComponent>>().iter().collect::<Vec<_>>();

    let ec = world.entities_components_mut();

    let Ok(current_dir) = std::env::current_dir() else {
        return;
    };

    let entry = ProjectWindowDataEntry::scan(&current_dir);

    for content in contents {
        fruits_engine::modules::utils::destroy_entity_children(ec, content);

        for entry in &entry.children {
            spawn_project_window_entries(
                ec,
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
    ec: &mut EntitiesComponentsHolder,
    material_text: &AssetHandle<StandardMaterial>,
    material_panel: &AssetHandle<StandardMaterial>,
    font: &AssetHandle<Font>,
    parent: Entity,
    entry: &ProjectWindowDataEntry,
 ) {
    let ent_entry = ec.create_entity();
    let ent_name_container = ec.create_entity();
    let ent_name = ec.create_entity();

    EntityComponentsBuilder::new(ec, ent_entry)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: parent })
        .add_component(ParentComponent { children: vec![] })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::Px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(parent).unwrap().children.push(ent_entry);

    EntityComponentsBuilder::new(ec, ent_name_container)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::Pd(1.0)), Some(UiVal::Px(20.0))),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_entry })
        .add_component(ParentComponent { children: vec![] })
        .add_component(ProjectWindowEntryComponent {
            path: entry.path.clone(),
        })
        .add_component(BatchedMeshComponent::default())
        .add_component(ButtonComponent)
        .add_component(StandardMaterialComponent { material: material_panel.clone() })
        .add_component(ImageComponent {
            color: Vec4::splat(0.0),
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(ent_entry).unwrap().children.push(ent_name_container);

    EntityComponentsBuilder::new(ec, ent_name)
        .add_component(DebugNameComponent(String::from("ent_name")))
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_name_container })
        .add_component(BatchedMeshComponent::default())
        .add_component(StandardMaterialComponent { material: material_text.clone() })
        .add_component(TextComponent {
            color: Vec4::from_array(parse_color_rgba_f32("#000000ff").unwrap()),
            font: font.clone(),
            font_size: UiVal::Px(18.0),
            is_y_inverted: true,
            text: entry.name.clone(),
            horizontal_spacing: 0.0,
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        });

    if entry.children.is_empty() {
        return;
    }

    let ent_children = ec.create_entity();
    let ent_children_container = ec.create_entity();

    EntityComponentsBuilder::new(ec, ent_children)
        .add_component(DebugNameComponent(String::from("ent_children")))
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_entry });
    ec.get_component_mut::<ParentComponent>(ent_entry).unwrap().children.push(ent_children);

    EntityComponentsBuilder::new(ec, ent_children_container)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            parent_padding_min: Vec2::new(UiVal::Px(20.0), UiVal::Px(0.0)),
            scale: Vec2::new(Some(UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_children })
        .add_component(ParentComponent { children: vec![] })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::Px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        });
        
    for entry in &entry.children {
        spawn_project_window_entries(
            ec,
            material_text,
            material_panel,
            font,
            ent_children_container,
            entry,
        );
    }
}