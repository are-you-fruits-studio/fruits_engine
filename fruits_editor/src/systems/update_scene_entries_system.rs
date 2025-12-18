use crate::{features::project_window_selection::{FileSelectedEvent, SelectedFileResource}, *};

pub fn update_scene_entries_system(
    mut world: ExclusiveWorldAccess,
) {
    let selected_file_res = world.resources().get::<SelectedFileResource>().unwrap();

    if world.events().get::<FileSelectedEvent>().is_empty() {
        return;
    }

    let prefab = std::fs::read_to_string(&selected_file_res.path).ok().map(|t| InspectorPrefab::deserialize(&t)).flatten();
    
    let contents = world.entities().query_filtered::<Entity, WithFilter<SceneWindowContentComponent>>().iter().collect::<Vec<_>>();

    let assets = world.resources().get::<StandardAssetsResource>().unwrap().clone();

    let standard_render_assets_res = world.resources().get::<StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();

    let mut ec = world.entities_mut();

    for content in contents {
        destroy_entity_children(ec.as_mut(), content);

        let Some(prefab) = &prefab else {
            continue;
        };

        for prefab_entity in &prefab.entities {
            spawn_scene_window_entry(
                ec.as_mut(),
                &assets.material_text,
                &assets.material_panel,
                &font,
                content,
                prefab_entity
            )
        }
    }
}

fn spawn_scene_window_entry(
    mut ec: EntitiesHolderMut,
    material_text: &AssetHandle<StandardMaterial>,
    material_panel: &AssetHandle<StandardMaterial>,
    font: &AssetHandle<Font>,
    parent: Entity,
    src: &InspectorPrefabEntity,
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
        .add_component(ChildComponent { parent: parent })
        .add_component(ParentComponent { children: vec![].into() })
        .add_component(RectChildAlignComponent {
            anchor: Vec2::new(0.0, 0.0),
            direction: UiDirection::Vertical,
            min_gap: UiVal::px(0.0),
            spacing: UiSpacing::Chunk,
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(parent).unwrap().children.push(ent_entry);

    EntityComponentsBuilder::new(ec.as_mut(), ent_name_container)
        .add_component(GlobalRectComponent::default())
        .add_component(LocalRectComponent {
            scale: Vec2::new(Some(UiVal::pd(1.0)).into(), Some(UiVal::px(20.0)).into()),
            ..Default::default()
        })
        .add_component(ChildComponent { parent: ent_entry })
        .add_component(ParentComponent { children: vec![].into() })
        .add_component(BatchedMeshComponent::default())
        .add_component(ButtonComponent)
        .add_component(StandardMaterialComponent { material: material_panel.clone() })
        .add_component(ImageComponent {
            color: Vec4::splat(0.0),
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(ent_entry).unwrap().children.push(ent_name_container);

    EntityComponentsBuilder::new(ec.as_mut(), ent_name)
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
            font_size: UiVal::px(18.0),
            is_y_inverted: true,
            text: String::from("todo").into(),
            horizontal_spacing: UiVal::px(0.0),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Left,
        });
}