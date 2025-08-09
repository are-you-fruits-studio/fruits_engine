use std::{net::{SocketAddr, TcpStream}, time::{Duration, Instant}};

use fruits_debug::{msg_types, DebugConnectionResource};
use fruits_modules::{asset::{AssetHandle, AssetStorageResource}, render::*, transform::*};
use fruits_prelude::*;
use fruits_reflection::refl_repr::*;
use fruits_utils::index_version_collection::VersionIndex;

use crate::{events::HierarchyUpdateEvent, resources::{AssetsResource, RequestsResource}};

pub fn init(
    mut world: ExclusiveWorldAccess,
) {
    let texture_text = world.resources().get::<StandardRenderAssetsResource>().unwrap().texture_text_px_8_8.clone();

    let material_text = world.resources_mut().get_mut::<AssetStorageResource<StandardMaterial>>().unwrap().insert(StandardMaterial::Unlit(UnlitMaterial {
        space: RenderSpace::Window,
        color: Vec4::with_all(1.0),
        color_tex: Some(texture_text),
        alpha_threshold: 0.5,
    }));

    world.resources_mut().insert(AssetsResource { material_text }).ok().unwrap();
    world.resources_mut().insert(RequestsResource::default()).ok().unwrap();
}

pub fn connect_debug_system(
    mut connection_res: ResMut<DebugConnectionResource>,
) {
    if connection_res.active_stream.is_none() {
        match TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], 55643)), Duration::from_millis(100)) {
            Ok(stream) => {
                connection_res.reset();
                stream.set_nonblocking(true).unwrap();
                connection_res.active_stream = Some(stream);
                println!("connected");
            },
            Err(err) => {
                return;
            },
        };
    }
}

pub fn request_hierarchy_system(
    mut connection_res: ResMut<DebugConnectionResource>,
    mut req_res: ResMut<RequestsResource>,
) {
    if connection_res.active_stream.is_none() {
        return;
    }

    if let Some(last_req_time) = req_res.last_req_time && last_req_time.elapsed().as_secs_f32() < 1.0 {
        return;
    }

    req_res.last_req_time = Some(Instant::now());
    connection_res.send_msg_queue.push_back((msg_types::HIERARCHY, Vec::new()));
}

pub fn parse_debug_msg_system(
    mut connection_res: ResMut<DebugConnectionResource>,
    mut hierarchy_update_evt: EvtMut<HierarchyUpdateEvent>,
) {
    let Some(msg) = connection_res.recv_msg_queue.pop_back() else {
        return;
    };
    
    if msg.0 == msg_types::HIERARCHY {
        let mut entities = Vec::new();

        for chunk in msg.1.chunks_exact(8) {
            let index_bytes: &[u8; 4] = &chunk[..4].try_into().unwrap();
            let version_bytes: &[u8; 4] = &chunk[4..].try_into().unwrap();

            entities.push(Entity::from_version_index(VersionIndex {
                index: u32::from_le_bytes(*index_bytes) as usize,
                version: u32::from_le_bytes(*version_bytes) as usize,
            }));
        }

        hierarchy_update_evt.push(HierarchyUpdateEvent {
            entities,
        });
    }
}

pub fn update_hierarchy(
    mut world: ExclusiveWorldAccess,
) {
    let (res, ec, evt) = world.as_tuple_mut();

    let Some(hierarchy_evt) = evt.get::<HierarchyUpdateEvent>().last() else {
        return;
    };

    for e in ec.query::<Entity>().iter().collect::<Vec<_>>() {
        ec.destroy_entity(e);
    }

    let font = res.get::<StandardRenderAssetsResource>().unwrap().font_px_8_8.clone();
    let material = res.get::<AssetsResource>().unwrap().material_text.clone();

    let layout = LayoutGlobalData {
        font,
        font_size: UiVal::Px(15.0),
        indent_size: 15.0,
        line_height: 20.0,
        material,
    };

    for (i, ent) in hierarchy_evt.entities.iter().enumerate() {
        let vi = ent.version_index();
        spawn_text(ec, &layout, format!("entity::{{ i: {}, v: {} }}", vi.index, vi.version), i, 0);
    }
}

fn spawn_repr_texts(
    ec: &mut EntitiesComponentsHolder,
    layout: &LayoutGlobalData,
    repr: &ReflRepr,
    line: &mut usize,
    indent: &mut usize,
) {
    match &repr {
        ReflRepr::Struct(repr) => {
            spawn_text(ec, layout, repr.name.clone(), *line, *indent);
            *line += 1;
            *indent += 1;

            match &repr.fields {
                ReflReprFields::Unit => (),
                ReflReprFields::Named(fields_named) => {
                    for (field_name, field_repr) in fields_named {
                        spawn_text(ec, layout, field_name.clone(), *line, *indent);
                        *line += 1;
                        
                        *indent += 1;
                        spawn_repr_texts(ec, layout, field_repr, line, indent);
                        *indent -= 1;
                    }
                },
                ReflReprFields::Tuple(fields_tuple) => {
                    for (i, field_repr) in fields_tuple.iter().enumerate() {
                        spawn_text(ec, layout, i.to_string(), *line, *indent);
                        *line += 1;

                        *indent += 1;
                        spawn_repr_texts(ec, layout, field_repr, line, indent);
                        *indent -= 1;
                    }
                },
            }

            *indent -= 1;
        },
        ReflRepr::Enum(repr) => todo!(),
        ReflRepr::Primitive(refl_repr_primitive) => {
            match refl_repr_primitive {
                ReflReprPrimitive::Int(repr) => {
                    spawn_text(ec, layout, repr.to_string(), *line, *indent);
                    *line += 1;
                },
                ReflReprPrimitive::Float(repr) => {
                    spawn_text(ec, layout, repr.to_string(), *line, *indent);
                    *line += 1;
                },
                ReflReprPrimitive::Char(repr) => {
                    spawn_text(ec, layout, repr.to_string(), *line, *indent);
                    *line += 1;
                },
                ReflReprPrimitive::Str(repr) => {
                    spawn_text(ec, layout, repr.to_string(), *line, *indent);
                    *line += 1;
                },
                ReflReprPrimitive::Bool(repr) => {
                    spawn_text(ec, layout, repr.to_string(), *line, *indent);
                    *line += 1;
                },
                ReflReprPrimitive::Unit => {
                    spawn_text(ec, layout, String::from("()"), *line, *indent);
                    *line += 1;
                },
            }
        },
    }
}

pub struct LayoutGlobalData {
    material: AssetHandle<StandardMaterial>,
    font: AssetHandle<Font>,
    font_size: UiVal,
    line_height: f32,
    indent_size: f32,
}

fn spawn_text(
    ec: &mut EntitiesComponentsHolder,
    layout: &LayoutGlobalData,
    text: String,
    line: usize,
    indent: usize,
) {
    let ent_type_name = ec.create_entity();
    ec.add_component(ent_type_name, GlobalTransform::IDENTITY).ok().unwrap();
    ec.add_component(ent_type_name, LocalTransform {
        position: Vec3::new(indent as f32 * layout.indent_size, (line as f32 + 0.5) * layout.line_height, 0.0),
        ..Default::default()
    }).ok().unwrap();
    ec.add_component(ent_type_name, BatchedMeshComponent::default()).ok().unwrap();
    ec.add_component(ent_type_name, StandardMaterialComponent { material: layout.material.clone() }).ok().unwrap();
    ec.add_component(ent_type_name, TextComponent {
        font: layout.font.clone(),
        font_size: layout.font_size,
        horizontal_align: HorizontalAlign::Left,
        vertical_align: VerticalAlign::Middle,
        horizontal_spacing: 0.0,
        is_y_inverted: true,
        text,
        color: Vec4::with_all(1.0),
    }).ok().unwrap();
}