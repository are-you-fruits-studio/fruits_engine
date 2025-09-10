use crate::*;

pub fn select_file_system(
    button_click_evt: Evt<ButtonClickEvent>,
    entry_q: WorldQuery<&ProjectWindowEntryComponent>,
    mut inspected_file_res: ResMut<SelectedFileResource>,
) {
    let Some(button_click_evt) = button_click_evt.last() else {
        return;
    };

    let Some(entry_c) = entry_q.get(button_click_evt.entity) else {
        return;
    };

    inspected_file_res.path = entry_c.path.clone();
}