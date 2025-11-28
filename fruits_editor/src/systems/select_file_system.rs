use crate::{features::ui_interaction::ButtonClickEvent, *};

pub fn select_file_system(
    button_click_evt: Evt<ButtonClickEvent>,
    entry_q: WorldQuery<&ProjectWindowEntryComponent>,
    mut selected_file_res: ResMut<SelectedFileResource>,
) {
    let Some(button_click_evt) = button_click_evt.last() else {
        return;
    };

    let Some(entry_c) = entry_q.get(button_click_evt.entity) else {
        return;
    };

    selected_file_res.path = entry_c.path.clone();
}