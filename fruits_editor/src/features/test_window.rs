use fruits_engine::*;

use crate::{SYSTEM_GROUP, components::{HierarchyWindowComponent, ProjectWindowComponent}, features::{inspector_window::data::InspectorWindowComponent, ui_interaction::{ButtonComponent, check_ui_interaction_system}}, prefabs::{self, WindowComponent}};

pub fn register_feature(mut world: WorldBuilderMut) {
    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(open_windows_system);

    update
        .order_system(check_ui_interaction_system)
        .before_system(open_windows_system);
}

#[derive(Component, Default, Debug)]
pub struct TestWindowButtonComponent;

#[derive(Component, Default, Debug)]
pub struct ProjectWindowButtonComponent;

#[derive(Component, Default, Debug)]
pub struct HierarchyWindowButtonComponent;

#[derive(Component, Default, Debug)]
pub struct InspectorWindowButtonComponent;

#[derive(Component, Default, Debug)]
pub struct TestWindowComponent;

pub fn open_windows_system(mut world: WorldDataMut) {
    open_specific_window_system::<TestWindowButtonComponent, _>(world.as_mut(), "Test Window", TestWindowComponent, |_, _| {});
    open_specific_window_system::<ProjectWindowButtonComponent, _>(world.as_mut(), "Project Window", ProjectWindowComponent::default(), |_, _| {});
    open_specific_window_system::<HierarchyWindowButtonComponent, _>(world.as_mut(), "Hierarchy Window", HierarchyWindowComponent::default(), |_, _| {});
    open_specific_window_system::<InspectorWindowButtonComponent, _>(world.as_mut(), "Inspector Window", InspectorWindowComponent::default(), prefabs::init_window_as_inspector);
}

fn open_specific_window_system<
    SpecButtonComponent: 'static + Component,
    SpecWindowComponent: 'static + Component,
>(
    mut world: WorldDataMut,
    title: &str,
    component: SpecWindowComponent,
    init: impl FnOnce(WorldDataMut, EntityId),
) {
    let ent = world.entities();
    
    let did_click = ent.query_filtered::<&ButtonComponent, WithFilter<SpecButtonComponent>>().iter().any(|b| b.was_clicked_this_frame);

    if !did_click {
        return;
    }

    if !ent.query::<&SpecWindowComponent>().is_empty() {
        return;
    }

    let ent_window = prefabs::window(world.as_mut());

    init(world.as_mut(), ent_window);

    let mut ent = world.entities_mut();

    ent.set_component(ent_window, component);

    let window_c = *ent.get_component_mut::<WindowComponent>(ent_window).unwrap();

    let text_c = ent.get_component_mut::<TextComponent>(window_c.title).unwrap();
    text_c.text.clear();
    text_c.text.push_str(title);
}