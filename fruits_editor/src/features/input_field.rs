use crate::*;

pub fn register_feature(mut world: WorldBuilderMut) {
    world.data_mut().resources_mut().insert(SelectedInputFieldResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(select_input_field_system);
    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(highlight_selected_input_field_system);
    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).insert_child_system(update_selected_input_field_text_system);

    world.behavior_mut().get_mut(Schedule::Update)
        .order_system(check_button_system)
        .before_system(select_input_field_system)
        .before_system(highlight_selected_input_field_system)
        .before_system(update_selected_input_field_text_system);
}

#[derive(Component)]
pub struct InputFieldComponent {
    pub text: Entity,
    pub selection_border: Entity,
}

#[derive(Resource, Default)]
pub struct SelectedInputFieldResource {
    pub selected: Entity,
}

pub fn select_input_field_system(
    input_res: Res<InputResource>,
    button_click_evt: Evt<ButtonClickEvent>,
    mut selected_input_res: ResMut<SelectedInputFieldResource>,
    input_field_q: WorldQuery<&InputFieldComponent>,
) {
    if input_res.mouse.is_just_pressed(MouseButton::Left) {
        selected_input_res.selected = Entity::EMPTY;
    }

    let Some(button_click_evt) = button_click_evt.last() else {
        return;
    };

    let Some(input_field_c) = input_field_q.get(button_click_evt.entity) else {
        return;
    };
    
    selected_input_res.selected = button_click_evt.entity;
}

pub fn highlight_selected_input_field_system(
    selected_input_res: Res<SelectedInputFieldResource>,
    input_field_q: WorldQuery<(&InputFieldComponent, Entity)>,
    mut disableable_q: WorldQuery<&mut LocalDisableableComponent>,
) {
    for (input_field_c, ent) in input_field_q.iter() {
        let Some(disableable_c) = disableable_q.get_mut(input_field_c.selection_border) else {
            continue;
        };
    
        disableable_c.is_disabled = selected_input_res.selected != ent;
    }
}

pub fn update_selected_input_field_text_system(
    text_input_evt: Evt<TextInputEvent>,
    selected_input_res: Res<SelectedInputFieldResource>,
    input_field_q: WorldQuery<&InputFieldComponent>,
    mut text_q: WorldQuery<&mut TextComponent>,
) {
    let Some(input_field_c) = input_field_q.get(selected_input_res.selected) else {
        return;
    };
    
    let Some(text_c) = text_q.get_mut(input_field_c.text) else {
        return;
    };

    for evt in text_input_evt.iter() {
        if evt.0 == "\u{8}" {
            let mut text = text_c.text.to_string();
            text.pop();
            text_c.text = text.into();
        } else {
            let mut text = text_c.text.to_string();
            text.push_str(&evt.0);
            text_c.text = text.into();
        }
    }
}