use fruits_engine::{ecs::{EntitiesComponentsHolder, ExclusiveWorldAccess}, prelude::{self as ayf, Component, ParentComponent, Resource}};

fn main() {
    let mut app = ayf::App::new();

    ayf::add_defult_modules_to(app.ecs_mut());

    app.ecs_mut().behavior_mut().get_mut(ayf::Schedule::Start).add_system(init_system);
    app.ecs_mut().behavior_mut().get_mut(ayf::Schedule::Update).add_system(update_project_window_content_system);

    app.ecs_mut().behavior_mut().get_mut(ayf::Schedule::Update).order_system(update_project_window_content_system).before_group(ayf::SYSTEM_GROUP_TRANSFORM);

    app.run();
}

fn init_system(
    mut world: ayf::ExclusiveWorldAccess,
) {
    let (res, ec, evt) = world.as_tuple_mut();

    let standard_render_assets_res = res.get::<ayf::StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();
    let texture_text = standard_render_assets_res.texture_text_px_8_8.clone();

    let materials_res = res.get_mut::<ayf::AssetStorageResource<ayf::StandardMaterial>>().unwrap();

    let material_panel = materials_res.insert(ayf::StandardMaterial::Unlit(ayf::UnlitMaterial {
        space: ayf::RenderSpace::Window,
        color: ayf::Vec4::splat(1.0),
        color_tex: None,
        alpha_threshold: 0.5,
    }));

    let material_text = materials_res.insert(ayf::StandardMaterial::Unlit(ayf::UnlitMaterial {
        space: ayf::RenderSpace::Window,
        color: ayf::Vec4::splat(1.0),
        color_tex: Some(texture_text),
        alpha_threshold: 0.5,
    }));

    res.insert(StandardAssetsResource {
        material_panel: material_panel.clone(),
        material_text: material_text.clone(),
    }).ok().unwrap();

    let ent_project_window = ec.create_entity();
    let ent_project_window_header = ec.create_entity();
    let ent_project_window_header_text = ec.create_entity();
    let ent_project_window_scroll = ec.create_entity();
    let ent_project_window_scroll_view = ec.create_entity();
    let ent_project_window_scroll_handle = ec.create_entity();
    let ent_project_window_scroll_content = ec.create_entity();

    EntityComponentsBuilder::new(ec, ent_project_window)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            ..Default::default()
        })
        .add_component(ayf::BatchedMeshComponent::default())
        .add_component(ayf::StandardMaterialComponent { material: material_panel.clone() })
        .add_component(ayf::ImageComponent {
            color: ayf::Vec4::from_array(ayf::parse_color_rgba_f32("#adadadff").unwrap()),
            ..Default::default()
        });
        
    EntityComponentsBuilder::new(ec, ent_project_window_header)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            anchor: ayf::Vec2::new(0.5, 0.0),
            pivot: ayf::Vec2::new(0.5, 0.0),
            scale: ayf::Vec2::new(Some(ayf::UiVal::Pd(1.0)), Some(ayf::UiVal::Px(20.0))),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_project_window })
        .add_component(ayf::BatchedMeshComponent::default())
        .add_component(ayf::StandardMaterialComponent { material: material_panel.clone() })
        .add_component(ayf::ImageComponent {
            color: ayf::Vec4::from_array(ayf::parse_color_rgba_f32("#929292ff").unwrap()),
            ..Default::default()
        });
        
    EntityComponentsBuilder::new(ec, ent_project_window_header_text)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            parent_padding_min: ayf::Vec2::splat(ayf::UiVal::Px(1.0)),
            parent_padding_max: ayf::Vec2::splat(ayf::UiVal::Px(1.0)),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_project_window_header })
        .add_component(ayf::BatchedMeshComponent::default())
        .add_component(ayf::StandardMaterialComponent { material: material_text.clone() })
        .add_component(ayf::TextComponent {
            color: ayf::Vec4::from_array(ayf::parse_color_rgba_f32("#000000ff").unwrap()),
            font: font.clone(),
            font_size: ayf::UiVal::Px(18.0),
            is_y_inverted: true,
            text: String::from("Project"),
            horizontal_spacing: 0.0,
            vertical_align: ayf::VerticalAlign::Middle,
            horizontal_align: ayf::HorizontalAlign::Left,
        });
        
    EntityComponentsBuilder::new(ec, ent_project_window_scroll)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            parent_padding_min: ayf::Vec2::new(ayf::UiVal::Px(0.0), ayf::UiVal::Px(20.0)),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_project_window })
        .add_component(ayf::BatchedMeshComponent::default())
        .add_component(ayf::StandardMaterialComponent { material: material_panel.clone() })
        .add_component(ayf::ImageComponent {
            color: ayf::Vec4::from_array(ayf::parse_color_rgba_f32("#757575ff").unwrap()),
            ..Default::default()
        });
        
    EntityComponentsBuilder::new(ec, ent_project_window_scroll_view)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            parent_padding_max: ayf::Vec2::new(ayf::UiVal::Px(20.0), ayf::UiVal::Px(0.0)),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_project_window_scroll })
        .add_component(ayf::ChildrenRectMaskComponent)
        .add_component(ayf::BatchedMeshComponent::default())
        .add_component(ayf::StandardMaterialComponent { material: material_panel.clone() })
        .add_component(ayf::ImageComponent {
            color: ayf::Vec4::from_array(ayf::parse_color_rgba_f32("#575757ff").unwrap()),
            ..Default::default()
        });
        
    EntityComponentsBuilder::new(ec, ent_project_window_scroll_content)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            anchor: ayf::Vec2::new(0.0, 0.0),
            pivot: ayf::Vec2::new(0.0, 0.0),
            scale: ayf::Vec2::new(Some(ayf::UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_project_window_scroll_view })
        .add_component(ayf::RectChildAlignComponent {
            anchor: ayf::Vec2::new(0.0, 0.0),
            direction: ayf::UiDirection::Vertical,
            min_gap: ayf::UiVal::Px(0.0),
            spacing: ayf::UiSpacing::Chunk,
            ..Default::default()
        })
        .add_component(ayf::ParentComponent { children: vec![] })
        .add_component(ProjectWindowContentComponent);

}

fn update_project_window_content_system(
    mut world: ExclusiveWorldAccess,
) {
    let assets = world.resources().get::<StandardAssetsResource>().unwrap().clone();

    let standard_render_assets_res = world.resources().get::<ayf::StandardRenderAssetsResource>().unwrap();

    let font = standard_render_assets_res.font_px_8_8.clone();

    let contents = world.entities_components().query_filtered::<ayf::Entity, ayf::WithFilter<ProjectWindowContentComponent>>().iter().collect::<Vec<_>>();

    let ec = world.entities_components_mut();

    let Ok(current_dir) = std::env::current_dir() else {
        return;
    };

    let entry = ProjectWindowDataEntry::scan(&current_dir);

    for content in contents {
        ayf::utils::destroy_entity_children(ec, content);

        for entry in &entry.children {
            spawn_project_window_entries(
                ec,
                &assets.material_text,
                &font,
                content,
                entry,
            );
        }
    }
}

fn spawn_project_window_entries(
    ec: &mut EntitiesComponentsHolder,
    material_text: &ayf::AssetHandle<ayf::StandardMaterial>,
    font: &ayf::AssetHandle<ayf::Font>,
    parent: ayf::Entity,
    entry: &ProjectWindowDataEntry,
 ) {
    let ent_entry = ec.create_entity();
    let ent_name = ec.create_entity();

    EntityComponentsBuilder::new(ec, ent_entry)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            scale: ayf::Vec2::new(Some(ayf::UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: parent })
        .add_component(ayf::ParentComponent { children: vec![] })
        .add_component(ayf::RectChildAlignComponent {
            anchor: ayf::Vec2::new(0.0, 0.0),
            direction: ayf::UiDirection::Vertical,
            min_gap: ayf::UiVal::Px(0.0),
            spacing: ayf::UiSpacing::Chunk,
            ..Default::default()
        });
    ec.get_component_mut::<ParentComponent>(parent).unwrap().children.push(ent_entry);

    EntityComponentsBuilder::new(ec, ent_name)
        .add_component(DebugNameComponent(String::from("ent_name")))
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            scale: ayf::Vec2::new(Some(ayf::UiVal::Pd(1.0)), Some(ayf::UiVal::Px(20.0))),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_entry })
        .add_component(ayf::BatchedMeshComponent::default())
        .add_component(ayf::StandardMaterialComponent { material: material_text.clone() })
        .add_component(ayf::TextComponent {
            color: ayf::Vec4::from_array(ayf::parse_color_rgba_f32("#000000ff").unwrap()),
            font: font.clone(),
            font_size: ayf::UiVal::Px(18.0),
            is_y_inverted: true,
            text: entry.name.clone(),
            horizontal_spacing: 0.0,
            vertical_align: ayf::VerticalAlign::Middle,
            horizontal_align: ayf::HorizontalAlign::Left,
        });
    ec.get_component_mut::<ParentComponent>(ent_entry).unwrap().children.push(ent_name);

    if entry.children.is_empty() {
        return;
    }

    // todo: Order of the ent_entry children (the layout order) is unstable.

    let ent_children = ec.create_entity();
    let ent_children_container = ec.create_entity();

    EntityComponentsBuilder::new(ec, ent_children)
        .add_component(DebugNameComponent(String::from("ent_children")))
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            scale: ayf::Vec2::new(Some(ayf::UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_entry });
    ec.get_component_mut::<ParentComponent>(ent_entry).unwrap().children.push(ent_children);

    EntityComponentsBuilder::new(ec, ent_children_container)
        .add_component(ayf::GlobalRectComponent::default())
        .add_component(ayf::LocalRectComponent {
            parent_padding_min: ayf::Vec2::new(ayf::UiVal::Px(20.0), ayf::UiVal::Px(0.0)),
            scale: ayf::Vec2::new(Some(ayf::UiVal::Pd(1.0)), None),
            ..Default::default()
        })
        .add_component(ayf::ChildComponent { parent: ent_children })
        .add_component(ayf::ParentComponent { children: vec![] })
        .add_component(ayf::RectChildAlignComponent {
            anchor: ayf::Vec2::new(0.0, 0.0),
            direction: ayf::UiDirection::Vertical,
            min_gap: ayf::UiVal::Px(0.0),
            spacing: ayf::UiSpacing::Chunk,
            ..Default::default()
        });
        
    for entry in &entry.children {
        spawn_project_window_entries(
            ec,
            material_text,
            font,
            ent_children_container,
            entry,
        );
    }
}

fn debug_layout_system(
    mut world: ExclusiveWorldAccess,
) {
    let aligns = world.entities_components().query_filtered::<ayf::Entity, ayf::WithFilter<ayf::RectChildAlignComponent>>().iter().collect::<Vec<_>>();

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

//

#[derive(Component, Debug, Clone)]
struct DebugNameComponent(pub String);

#[derive(Debug, Clone)]
struct ProjectWindowDataEntry {
    pub name: String,
    pub children: Vec<ProjectWindowDataEntry>,
}

impl ProjectWindowDataEntry {
    pub fn scan(src: &std::path::PathBuf) -> Self {
        Self {
            name: src.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default(),
            children: Self::get_children(src),
        }
    }

    fn from_dir_entry(src: &std::fs::DirEntry) -> Self {
        Self {
            name: src.file_name().to_string_lossy().into_owned(),
            children: Self::get_children(&src.path()),
        }
    }

    fn get_children(src: &std::path::PathBuf) -> Vec<Self> {
        let mut result = Vec::new();
        
        let Ok(read_dir) = std::fs::read_dir(src) else {
            return result;
        };

        for dir_entry in read_dir {
            if let Ok(dir_entry) = dir_entry {
                result.push(Self::from_dir_entry(&dir_entry));
            }
        }

        result
    }
}

struct EntityComponentsBuilder<'ec> {
    ec: &'ec mut ayf::EntitiesComponentsHolder,
    ent: ayf::Entity,
}

impl<'ec> EntityComponentsBuilder<'ec> {
    pub fn new(ec: &'ec mut ayf::EntitiesComponentsHolder, ent: ayf::Entity) -> Self {
        Self {
            ec,
            ent,
        }
    }

    pub fn add_component<C: ayf::Component>(&mut self, component: C) -> &mut Self {
        self.ec.add_component(self.ent, component).ok().unwrap();
        self
    }
}

//

#[derive(Resource, Clone)]
pub struct StandardAssetsResource {
    pub material_panel: ayf::AssetHandle<ayf::StandardMaterial>,
    pub material_text: ayf::AssetHandle<ayf::StandardMaterial>,
}

//

#[derive(Component)]
pub struct ProjectWindowContentComponent;

//