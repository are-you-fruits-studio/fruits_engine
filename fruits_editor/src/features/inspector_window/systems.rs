use std::collections::HashMap;

use fruits_engine::tree::{TreeBuilder, TreeNode};

use crate::{
    features::{
        asset_serialization::{InspectedAsset, get_asset_type}, input_field::{InputFieldComponent, InputFieldSelectionChangedEvent, SelectedInputFieldResource}, inspector_window::{
            data::*, utils::{
                entries::{parse_serialized, spawn_default_layout_ent, spawn_hierarchy_window_entry, spawn_serialized, spawn_serialized_field, spawn_text_ent}, serialization::{are_components_slices_similar, enrich_serialized_with_asset_type, load_asset_to_world_res, save_asset_from_world_res}, subsequence_match_ignore_case,
            },
        }, project_window_selection::{FileSelectedEvent, SelectedFileResource}, world_preload::SimulatedWorldResource,
    }, *,
};

pub fn select_entity_system(
    mut inspected_entity: ResMut<InspectedEntityResource>,
    button_click_evt: Evt<ButtonClickEvent>,
    entry_q: WorldQuery<&HierarchyWindowEntryComponent>,
    mut prefab_entity_selected_evt: EvtMut<PrefabEntitySelectedEvent>,
) {
    return_if_not!(Some(button_click_evt) = button_click_evt.last());

    if let Some(entry_c) = entry_q.get(button_click_evt.entity) {
        inspected_entity.selected_entity = entry_c.simulated_entity;
        prefab_entity_selected_evt.push(PrefabEntitySelectedEvent);
    }
}

pub fn update_hierarchy_entries_selection(
    inspected_entity: Res<InspectedEntityResource>,
    mut entry_q: WorldQuery<(&HierarchyWindowEntryComponent, &mut ImageComponent)>,
) {
    for (entry_c, image_c) in entry_q.iter_mut() {
        if entry_c.simulated_entity == inspected_entity.selected_entity {
            image_c.color = Vec4::from_array(const { parse_color_rgba_f32("#7d90e6ff").unwrap() });
        } else {
            image_c.color = Vec4::from_array(const { parse_color_rgba_f32("#00000000").unwrap() });
        }
    }
}

pub fn adjust_non_rigid_composite_system(mut world: WorldDataMut) {
    let (res, mut ent, evt) = world.as_tuple_mut();

    let ent_selected_input = res.as_ref().get::<SelectedInputFieldResource>().unwrap().selected;
    let assets = res.as_ref().get::<StandardAssetsResource>().unwrap().clone();

    for click_evt in evt.get::<ButtonClickEvent>() {
        if let Some(btn_add_c) = ent.get_component::<SerializedCompositeRemoveButton>(click_evt.entity)
            && let Some(serialized_val_c) = ent.get_component::<SerializedValueComponent>(btn_add_c.composite).copied()
            && let SerializedValueComponent::Container {
                ty, container_fields, ..
            } = serialized_val_c
        {
            let parent_c = ent.get_component_mut::<ParentComponent>(container_fields).unwrap();
            if let Some(ent_child) = parent_c.children.pop() {
                destroy_entity_and_children(ent.as_mut(), ent_child);
            }
        }
        if let Some(btn_add_c) = ent.get_component::<SerializedCompositeAddButton>(click_evt.entity)
            && let Some(serialized_val_c) = ent.get_component::<SerializedValueComponent>(btn_add_c.composite).copied()
            && let SerializedValueComponent::Container {
                ty, container_fields, ..
            } = serialized_val_c
        {
            let i = ent.get_component::<ParentComponent>(container_fields).unwrap().children.len();
            let serialized_key = match ty {
                SerializedValueContainerType::List => FfiString::from(i.to_string()),
                SerializedValueContainerType::Map => FfiString::from(""),
            };
            spawn_serialized_field(
                ent.as_mut(),
                i,
                serialized_key,
                container_fields,
                &SerializedValue::Null,
                ent_selected_input,
                assets.material_panel.clone(),
                assets.material_text.clone(),
                assets.font.clone(),
            );
        }
    }
}

pub fn remove_component_system(
    mut ent: EntitiesHolderMut,
    button_click_evt: Evt<ButtonClickEvent>,
) {
    for button_click_evt in button_click_evt.iter() {
        
        if let Some(component_remove_button_c) = ent.get_component::<ComponentRemoveButton>(button_click_evt.entity).copied() {
            destroy_entity_and_children(ent.as_mut(), component_remove_button_c.component);
        }
    }
}

pub fn add_component_system(
    mut ent: EntitiesHolderMut,
    button_click_evt: Evt<ButtonClickEvent>,
    assets: Res<StandardAssetsResource>,
) {
    let Some(inspector_window_c) = ent.as_mut().query::<&InspectorWindowComponent>().iter().next().copied() else {
        return;
    };

    let container_ent = inspector_window_c.content_container;

    for button_click_evt in button_click_evt.iter() {
        let Some(add_component_variant_c) = ent.get_component::<AddComponentVariantComponent>(button_click_evt.entity) else {
            continue;
        };

        let component_id = add_component_variant_c.component_id.clone();

        update_prefab_component_ent(
            ent.as_mut(),
            EntityId::EMPTY,
            container_ent,
            EntityId::EMPTY,
            &PrefabComponent {
                component_id,
                data: SerializedValue::Null,
            },
            assets.material_panel.clone(),
            assets.material_text.clone(),
            assets.font.clone(),
        );        
    }
}

pub fn update_add_component_variants_system(
    mut ent: EntitiesHolderMut,
    selected_input: Res<SelectedInputFieldResource>,
    assets: Res<StandardAssetsResource>,
    simulated_world: Res<SimulatedWorldResource>,
) {
    for entity in ent.query::<&AddComponentInputComponent>().iter().map(|c| c.variants_container).collect::<Vec<_>>() {
        destroy_entity_children(ent.as_mut(), entity);
    }

    let Some(simulated_world) = &simulated_world.0 else {
        return;
    };

    let Some(serializers) = simulated_world.world.data().resources().get::<SerializersResource>() else {
        return;
    };

    let Some(add_component_input_c) = ent.get_component::<AddComponentInputComponent>(selected_input.selected).copied() else {
        return;
    };

    let Some(input_field_c) = ent.get_component::<InputFieldComponent>(selected_input.selected) else {
        return;
    };

    let Some(text_c) = ent.get_component::<TextComponent>(input_field_c.text) else {
        return;
    };

    let searched_text = text_c.text.clone();

    let mut keys = serializers.0.registry().keys().map(FfiString::from).collect::<Vec<_>>();
    keys.sort();
    
    for component_id in keys {
        if !subsequence_match_ignore_case(component_id.as_str(), searched_text.as_str()) {
            continue;
        }

        let entry_ent = spawn_default_layout_ent(
            ent.as_mut(),
            add_component_input_c.variants_container,
            false,
        );

        let (entry_ent, _) = spawn_text_ent(
            ent.as_mut(),
            entry_ent,
            component_id.clone(),
            assets.material_panel.clone(),
            assets.material_text.clone(),
            assets.font.clone(),
        );

        ent.add_component(entry_ent, AddComponentVariantComponent { component_id: component_id }).ok().unwrap();
        ent.add_component(entry_ent, ButtonComponent).ok().unwrap();
    }
}

pub fn adjust_hierarchy_entries_system(
    button_click_evt: Evt<ButtonClickEvent>,
    mut inspected_entity: ResMut<InspectedEntityResource>,
    mut simulated_world: ResMut<SimulatedWorldResource>,
    button_add_q: WorldQuery<&HierarchyButtonAddComponent>,
    button_remove_q: WorldQuery<&HierarchyButtonRemoveComponent>,
) {
    let Some(simulated_world) = &mut simulated_world.0 else {
        return;
    };

    for click_evt in button_click_evt.iter() {
        if button_remove_q.get(click_evt.entity).is_some() {
            let removed_entity = inspected_entity.selected_entity;
            simulated_world.world.data_mut().entities_mut().destroy_entity(removed_entity);
            
            if let Some(removed_id) = inspected_entity.ent_to_id.remove(&removed_entity) {
                inspected_entity.id_to_ent.remove(&removed_id);
                inspected_entity.selected_entity = EntityId::EMPTY;
            }
        }
        if button_add_q.get(click_evt.entity).is_some() {
            let entity = simulated_world.world.data_mut().entities_mut().create_entity();
            
            let new_id = (1..=(inspected_entity.id_to_ent.len() as u64 + 1)).filter(|i| !inspected_entity.id_to_ent.contains_key(i)).next().unwrap();

            inspected_entity.ent_to_id.insert(entity, new_id);
            inspected_entity.id_to_ent.insert(new_id, entity);

            inspected_entity.selected_entity = entity;
        }
    }
}

pub fn apply_inspector_to_simulated_world_system(
    ent: EntitiesHolderRef,
    inspector_window_q: WorldQuery<&InspectorWindowComponent>,
    parent_q: WorldQuery<&ParentComponent>,
    text_q: WorldQuery<&TextComponent>,
    serialized_component_q: WorldQuery<&SerializedComponentComponent>,
    selected_file: Res<SelectedFileResource>,
    inspected_entity: Res<InspectedEntityResource>,
    open_project: Res<OpenProjectResource>,
    mut simulated_world: ResMut<SimulatedWorldResource>,
    mut inspected_asset_edited_evt: EvtMut<InspectedAssetEditedEvent>,
) {
    return_if_not!(Some(simulated_world) = &mut simulated_world.0);
    return_if_not!(Some(window_c) = inspector_window_q.iter().next().copied());
    let content_container = window_c.content_container;
    return_if_not!(Some(parent_c) = parent_q.get(content_container));
    return_if_not!(Some(&content_ent) = parent_c.children.first());

    return_if_not!(Some(asset_type) = get_asset_type(simulated_world.world.data().resources(), selected_file.potential_asset_key.as_str()));

    if asset_type == AssetType::Prefab {
        let stored_components = record_into_prefab_components(
            simulated_world.world.data().resources(),
            simulated_world.world.data().entities(),
            inspected_entity.selected_entity,
            &inspected_entity.ent_to_id,
        );

        let mut parsed_components = Vec::new();

        for &child in &parent_c.children {
            continue_if_not!(Some(serialized_component_c) = serialized_component_q.get(child));
            continue_if_not!(Some(text_c) = text_q.get(serialized_component_c.component_id_text));
            continue_if_not!(Some(component_data_container_parent_c) = parent_q.get(serialized_component_c.component_data_container));
            continue_if_not!(Some(&serialized_ent) = component_data_container_parent_c.children.get(0));

            let serialized_component = parse_serialized(ent, serialized_ent);

            parsed_components.push(PrefabComponent {
                component_id: text_c.text.clone(),
                data: serialized_component,
            });
        }

        return_if!(are_components_slices_similar(&stored_components, &parsed_components));

        let (sim_res, sim_ent, sim_evt) = simulated_world.world.data_mut().as_tuple_mut();
        override_entity_components_from_prefab(
            sim_res.as_ref(),
            sim_ent,
            inspected_entity.selected_entity,
            &parsed_components,
            &inspected_entity.id_to_ent,
        );

        inspected_asset_edited_evt.push(InspectedAssetEditedEvent);

        return;
    }

    let serialized_existing = save_asset_from_world_res(
        simulated_world.world.data().resources(),
        simulated_world.world.data().resources().get::<SerializersResource>().unwrap(),
        selected_file.potential_asset_key.as_str(),
    );

    let mut serialized_parsed = parse_serialized(ent, content_ent);

    enrich_serialized_with_asset_type(&mut serialized_parsed, asset_type);

    return_if!(SerializedValue::similar_option(
        serialized_existing.as_ref(),
        Some(&serialized_parsed)
    ));

    // todo
    let did_asset_load = load_asset_to_world_res(
        simulated_world.world.data_mut().resources_mut(),
        selected_file.potential_asset_key.as_str(),
        &serialized_parsed,
        asset_type,
        &(open_project.dir_path.to_string() + PROJECT_ASSETS_SUBPATH),
    );

    if did_asset_load {
        inspected_asset_edited_evt.push(InspectedAssetEditedEvent);
    };
}

pub fn save_simulated_entities_to_prefab_system(
    mut simulated_world: ResMut<SimulatedWorldResource>,
    inspected_asset: Res<InspectedAssetResource>,
) {
    let Some(simulated_world) = &mut simulated_world.0 else {
        return;
    };

    let Some(prefab) = record_into_prefab(
        simulated_world.world.data().resources(),
        simulated_world.world.data().entities(),
        inspected_asset.spawned_prefab
    ) else {
        return;
    };

    let Some(AssetType::Prefab) = get_asset_type(simulated_world.world.data().resources(), inspected_asset.asset_key.as_str()) else {
        return;
    };

    let prefabs = simulated_world.world.data_mut().resources_mut().get_mut::<AssetStorageResource<Prefab>>().unwrap();
    let prefab_handle = prefabs.get_registered(inspected_asset.asset_key.as_str()).unwrap().clone();

    *prefabs.get_mut(&prefab_handle).unwrap() = prefab;
}

pub fn save_inspected_asset_from_simulated_world_to_file_system(
    evt_inspected_asset_edited: Evt<InspectedAssetEditedEvent>,
    simulated_world: Res<SimulatedWorldResource>,
    selected_file: Res<SelectedFileResource>,
    inspected_asset: Res<InspectedAssetResource>,
) {
    return_if!(evt_inspected_asset_edited.is_empty() && inspected_asset.asset_key == selected_file.potential_asset_key);

    return_if_not!(Some(simulated_world) = &simulated_world.0);

    let serialized = save_asset_from_world_res(
        simulated_world.world.data().resources(),
        simulated_world.world.data().resources().get::<SerializersResource>().unwrap(),
        selected_file.potential_asset_key.as_str(),
    );

    return_if_not!(Some(serialized) = serialized);

    let json_str = serde_json::to_string_pretty(&serialized.to_json()).unwrap();

    let json_bytes = json_str.into_bytes();

    if let Err(err) = std::fs::write(&selected_file.path, &json_bytes) {
        eprintln!("failed to write to {:?}. {}", &selected_file.path, err);
        return;
    }
}

pub fn destroy_non_inspected_entity_system(
    mut simulated_world: ResMut<SimulatedWorldResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
    selected_file: Res<SelectedFileResource>,
) {
    return_if!(selected_file.potential_asset_key == inspected_asset.asset_key);

    return_if_not!(Some(world) = &mut simulated_world.0);

    for entity in world.world.data().entities().query::<EntityId>().iter().collect::<Vec<_>>() {
        world.world.data_mut().entities_mut().destroy_entity(entity);
    }

    inspected_asset.spawned_prefab = EntityId::EMPTY;
}

pub fn spawn_inspected_prefab_system(
    mut simulated_world: ResMut<SimulatedWorldResource>,
    mut inspected_asset: ResMut<InspectedAssetResource>,
    mut inspected_entity: ResMut<InspectedEntityResource>,
    selected_file: Res<SelectedFileResource>,
) {
    return_if!(selected_file.potential_asset_key == inspected_asset.asset_key);

    return_if_not!(Some(world) = &mut simulated_world.0);

    let (res, mut ent, _evt) = world.world.data_mut().as_tuple_mut();

    return_if!(get_asset_type(res.as_ref(), selected_file.potential_asset_key.as_str()) != Some(AssetType::Prefab));

    return_if_not!(
        Some(prefab_handle) = res
            .as_ref()
            .get::<AssetStorageResource<Prefab>>()
            .unwrap()
            .get_registered(selected_file.potential_asset_key.as_str())
            .cloned()
    );

    let instantiated = instantiate_prefab(res.as_ref(), ent.as_mut(), prefab_handle).unwrap_or(EntityId::EMPTY);

    if !ent.contains_entity(instantiated) {
        for entity in ent.query::<EntityId>().iter().collect::<Vec<_>>() {
            ent.destroy_entity(entity);
        }
        return;
    }

    inspected_entity.ent_to_id.clear();
    inspected_entity.id_to_ent.clear();

    for entity in ent.query::<EntityId>().iter() {
        let id = entity.version_index().index + 1;

        inspected_entity.ent_to_id.insert(entity, id);
        inspected_entity.id_to_ent.insert(id, entity);
    }

    inspected_asset.spawned_prefab = instantiated;
}

pub fn update_hierarchy_window_system(
    simulated_world: Res<SimulatedWorldResource>,
    inspected_entity: Res<InspectedEntityResource>,
    mut entities: EntitiesHolderMut,
    assets: Res<StandardAssetsResource>,
    file_selected_evt: Evt<FileSelectedEvent>,
) {
    return_if_not!(Some(simulated_world) = simulated_world.0.as_ref());

    // todo
    // if file_selected_evt.is_empty() {
    //     return;
    // }

    let contents = entities
        .as_ref()
        .query_filtered::<EntityId, WithFilter<HierarchyWindowContentComponent>>()
        .iter()
        .collect::<Vec<_>>();

    for content in contents {
        destroy_entity_children(entities.as_mut(), content);

        let mut hierarchy_tree = TreeBuilder::new();

        for (sim_ent, sim_child_c) in simulated_world.world.data().entities().query::<(EntityId, Option<&ChildComponent>)>().iter() {
            let sim_parent = sim_child_c.map(|c| c.parent).filter(|p| simulated_world.world.data().entities().contains_entity(*p));

            match sim_parent {
                Some(sim_parent) => hierarchy_tree.insert_pair(sim_parent, sim_ent),
                None => hierarchy_tree.insert_single(sim_ent),
            };
        }

        let mut hierarchy_tree = hierarchy_tree.build();

        hierarchy_tree.sort_by_key(|n| n.value.version_index().index);
        for node in &mut hierarchy_tree {
            node.iter_nodes_recursively(|n| n.children.sort_by_key(|n| n.value.version_index().index));
        }

        for node in &hierarchy_tree {
            spawn_hierarchy_window_entries_recursively(
                entities.as_mut(),
                simulated_world.world.data().entities(),
                &assets.material_text,
                &assets.material_panel,
                &assets.font,
                content,
                &inspected_entity.ent_to_id,
                node,
                0,
            )
        }
    }
}

fn spawn_hierarchy_window_entries_recursively(
    mut ent: EntitiesHolderMut,
    sim_ent: EntitiesHolderRef,
    material_text: &AssetHandle<StandardMaterial>,
    material_panel: &AssetHandle<StandardMaterial>,
    font: &AssetHandle<Font>,
    parent: EntityId,
    ent_to_id: &HashMap<EntityId, u64>,
    node: &TreeNode<EntityId>,
    depth: usize,
) {
    let simulated_ent = node.value;

    let entity_id = ent_to_id.get(&simulated_ent).map(|n| n.to_string()).unwrap_or_else(|| "_".into());

    let name = sim_ent.get_component::<DebugNameComponent>(simulated_ent)
        .map(|s| s.0.as_str().into())
        .unwrap_or_else(|| format!("e{{{}}}", simulated_ent.version_index().index));

    // todo: text indent to layout indent
    let indent = std::iter::repeat_n(' ', depth).collect::<String>();
    
    let name = format!("{}[{}] {}", indent, entity_id, name).into();

    spawn_hierarchy_window_entry(
        ent.as_mut(),
        &material_text,
        &material_panel,
        &font,
        name,
        parent,
        simulated_ent,
    );

    for child in &node.children {
        spawn_hierarchy_window_entries_recursively(
            ent.as_mut(),
            sim_ent,
            material_text,
            material_panel,
            font,
            parent,
            ent_to_id,
            child,
            depth + 1,
        );
    }
}

pub fn change_inspected_asset_system(selected_file: Res<SelectedFileResource>, mut inspected_asset: ResMut<InspectedAssetResource>) {
    if inspected_asset.asset_key.as_str() != selected_file.potential_asset_key.as_str() {
        inspected_asset.asset_key = selected_file.potential_asset_key.clone();
    }
}

pub fn update_inspector_window_system(
    mut ent: EntitiesHolderMut,
    selected_input_field: Res<SelectedInputFieldResource>,
    inspected_entity: Res<InspectedEntityResource>,
    selected_file: Res<SelectedFileResource>,
    simulated_world: Res<SimulatedWorldResource>,
    assets: Res<StandardAssetsResource>,
    file_selected_evt: Evt<FileSelectedEvent>,
    inspected_asset_edited_evt: Evt<InspectedAssetEditedEvent>,
    prefab_entity_selected_evt: Evt<PrefabEntitySelectedEvent>,
    input_field_selection_changed_evt: Evt<InputFieldSelectionChangedEvent>,
) {
    // todo
    return_if!(
        file_selected_evt.is_empty()
            && inspected_asset_edited_evt.is_empty()
            && input_field_selection_changed_evt.is_empty()
            && prefab_entity_selected_evt.is_empty()
    );

    let ent_selected_input = selected_input_field.selected;

    let container_q = ent.query::<&InspectorWindowComponent>().iter().copied().collect::<Vec<_>>();

    return_if_not!(Some(simulated_world) = &simulated_world.0);

    for window_c in container_q {
        let asset_type_text = ent.get_component_mut::<TextComponent>(window_c.asset_type_text).unwrap();
        asset_type_text.text.clear();

        let Some(asset_type) = get_asset_type(simulated_world.world.data().resources(), selected_file.potential_asset_key.as_str()) else {
            destroy_entity_children(ent.as_mut(), window_c.content_container);
            continue;
        };

        asset_type_text.text.push_str("asset type: ");
        asset_type_text.text.push_str(asset_type.serialized_str());

        let inspected_asset = InspectedAsset {
            asset_key: selected_file.potential_asset_key.clone().into(),
            asset_type,
        };

        // todo
        let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
        // let mut err_handler = |_| ();

        // todo
        if asset_type == AssetType::Prefab {
            let inspected_components = record_into_prefab_components(
                simulated_world.world.data().resources(),
                simulated_world.world.data().entities(),
                inspected_entity.selected_entity,
                &inspected_entity.ent_to_id,
            );

            update_prefab_components_ent(
                ent.as_mut(),
                window_c.content_container,
                ent_selected_input,
                &inspected_components,
                assets.material_panel.clone(),
                assets.material_text.clone(),
                assets.font.clone(),
            );

            continue;
        }

        let serialized = save_with_asset_serializers_from_world(simulated_world.world.data().resources(), None, |local_registry| {
            let serializers = simulated_world.world.data().resources().get::<SerializersResource>().unwrap();
            let serializer_ctx = serializers.0.to_ctx(Some(&local_registry), &mut err_handler);
            inspected_asset.to_serialized(serializer_ctx)
        })
        .unwrap();

        let ent_last = ent
            .get_component::<ParentComponent>(window_c.content_container)
            .map(|p| p.children.first())
            .flatten()
            .copied()
            .unwrap_or(EntityId::EMPTY);

        spawn_serialized(
            ent.as_mut(),
            ent_last,
            window_c.content_container,
            ent_selected_input,
            &serialized,
            assets.material_panel.clone(),
            assets.material_text.clone(),
            assets.font.clone(),
        );
    }
}

fn update_prefab_components_ent(
    mut ent: EntitiesHolderMut,
    ent_parent: EntityId,
    ent_selected_input: EntityId,
    components: &[PrefabComponent],
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) {
    if let Some(parent_c) = ent.get_component_mut::<ParentComponent>(ent_parent) {
        while ent.get_component_mut::<ParentComponent>(ent_parent).unwrap().children.len() > components.len() as u64 {
            let popped_ent = ent.get_component_mut::<ParentComponent>(ent_parent).unwrap().children.pop().unwrap();
            destroy_entity_and_children(ent.as_mut(), popped_ent);
        }

        for i in 0..components.len() {
            let ent_last = ent.get_component_mut::<ParentComponent>(ent_parent).unwrap().children.get(i as u64).copied().unwrap_or(EntityId::EMPTY);

            update_prefab_component_ent(
                ent.as_mut(),
                ent_last,
                ent_parent,
                ent_selected_input,
                &components[i],
                material_panel.clone(),
                material_text.clone(),
                font.clone(),
            )
        }
    }
}

fn update_prefab_component_ent(
    mut ent: EntitiesHolderMut,
    ent_last: EntityId,
    ent_parent: EntityId,
    ent_selected_input: EntityId,
    component: &PrefabComponent,
    material_panel: AssetHandle<StandardMaterial>,
    material_text: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
) {
    if let Some(&serialized_component_c) = ent.get_component::<SerializedComponentComponent>(ent_last)
    && let Some(container_parent_c) = ent.get_component::<ParentComponent>(serialized_component_c.component_data_container)
    && container_parent_c.children.len() == 1
    && let data_ent = container_parent_c.children[0]
    && let Some(text_c) = ent.get_component_mut::<TextComponent>(serialized_component_c.component_id_text) {
        text_c.text = component.component_id.clone();
        
        spawn_serialized(
            ent.as_mut(),
            data_ent,
            serialized_component_c.component_data_container,
            ent_selected_input,
            &component.data,
            material_panel.clone(),
            material_text.clone(),
            font.clone(),
        );

        return;
    }

    let comp_ent = spawn_default_layout_ent(ent.as_mut(), ent_parent, false);
    let (_, componet_id_text_ent) = spawn_text_ent(
        ent.as_mut(),
        comp_ent,
        component.component_id.clone(),
        material_panel.clone(),
        material_text.clone(),
        font.clone(),
    );
    let (btn_remove, _) = spawn_text_ent(
        ent.as_mut(),
        comp_ent,
        "X".into(),
        material_panel.clone(),
        material_text.clone(),
        font.clone(),
    );
    ent.add_component(btn_remove, ComponentRemoveButton { component: comp_ent }).ok().unwrap();
    ent.add_component(btn_remove, ButtonComponent).ok().unwrap();
    let comp_data_container = spawn_default_layout_ent(ent.as_mut(), comp_ent, false);

    ent.add_component(comp_ent, SerializedComponentComponent {
        component_id_text: componet_id_text_ent,
        component_data_container: comp_data_container,
    }).ok().unwrap();
    
    spawn_serialized(
        ent.as_mut(),
        EntityId::EMPTY,
        comp_data_container,
        ent_selected_input,
        &component.data,
        material_panel.clone(),
        material_text.clone(),
        font.clone(),
    );
}

// todo
fn tree_to_debug_str(tree: &[TreeNode<EntityId>]) -> String {
    fn log_tree_internal(s: &mut String, tree: &[TreeNode<EntityId>]) {
        if tree.is_empty() {
            return;
        }

        s.push_str(" { ");

        for node in tree {
            s.push_str(&node.value.version_index().index.to_string());

            log_tree_internal(s, &node.children);
            
            s.push_str(", ");
        }

        s.push_str("}");
    }

    let mut s = String::new();
    log_tree_internal(&mut s, tree);

    s
}