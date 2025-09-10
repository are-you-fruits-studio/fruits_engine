use crate::*;

pub fn update_project_entry_selection_system(
    inspected_file: Res<InspectedFileResource>,
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