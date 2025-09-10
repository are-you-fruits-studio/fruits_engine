use crate::*;

pub fn debug_layout_system(
    mut world: ExclusiveWorldAccess,
) {
    let aligns = world.entities_components().query_filtered::<Entity, WithFilter<RectChildAlignComponent>>().iter().collect::<Vec<_>>();

    for align in aligns {
        let children = world.entities_components_mut().get_component_mut::<ParentComponent>(align).map(|p| p.children.as_slice()).unwrap_or(&[]).to_vec();

        let mut children_debug = Vec::new();

        for child in children {
            if let Some(name) = world.entities_components().get_component::<DebugNameComponent>(child).map(|c| c.0.as_str()) {
                children_debug.push(name);
            }
        }

        if children_debug.len() > 1 {
            println!("{:?}" , children_debug);
        }
    }
}
