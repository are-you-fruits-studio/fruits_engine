use std::ffi::OsString;

use crate::*;

pub fn register_feature(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(SelectedFileResource::default())
        .ok()
        .unwrap();

    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(select_file_system)
        .insert_child_system(update_project_entry_selection_system);

    update
        .order_system(check_button_system)
        .before_system(select_file_system)
        .before_system(update_project_entry_selection_system);
}

#[derive(Resource, Clone, Default)]
pub struct SelectedFileResource {
    pub path: OsString,
    pub file_data: Vec<u8>,
}

#[derive(Event, Copy, Clone, Default)]
pub struct FileSelectedEvent;

pub fn select_file_system(
    button_click_evt: Evt<ButtonClickEvent>,
    entry_q: WorldQuery<&ProjectWindowEntryComponent>,
    mut selected_file_res: ResMut<SelectedFileResource>,
    mut select_evt: EvtMut<FileSelectedEvent>,
) {
    let Some(button_click_evt) = button_click_evt.last() else {
        return;
    };

    let Some(entry_c) = entry_q.get(button_click_evt.entity) else {
        return;
    };

    selected_file_res.path = entry_c.path.clone();
    selected_file_res.file_data = std::fs::read(&selected_file_res.path).unwrap_or_else(|_| Vec::new());
    select_evt.push(FileSelectedEvent);
}

pub fn update_project_entry_selection_system(
    inspected_file: Res<SelectedFileResource>,
    mut entry_q: WorldQuery<(&ProjectWindowEntryComponent, &mut ImageComponent)>,
) {
    for (entry_c, image_c) in entry_q.iter_mut() {
        if entry_c.path == inspected_file.path {
            image_c.color = Vec4::from_array(const { parse_color_rgba_f32("#7d90e6ff").unwrap() });
        } else {
            image_c.color = Vec4::from_array(const { parse_color_rgba_f32("#00000000").unwrap() });
        }
    }
}
