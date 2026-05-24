use std::ffi::OsString;

use fruits_engine::*;

#[derive(Component)]
pub struct ProjectWindowContentComponent;

#[derive(Component, Debug, Clone)]
pub struct DebugNameComponent(pub String);

#[derive(Component, Debug, Clone)]
pub struct ProjectWindowEntryComponent {
    pub path: OsString,
}

#[derive(Component)]
pub struct SceneWindowContentComponent;

#[derive(Component)]
pub enum SerializedValueComponent {
    Container { container: Entity, ty: SerializedValueContainerType, enum_metadata: Entity },
    Primitive { text: Entity, ty: SerializedValuePrimitiveType },
}

pub struct SerializedFieldComponent {
    pub key_text: Entity,
    pub value_container: Entity
}

pub struct SerializedEnumMetadataComponent {
    pub value_text: Entity,
    pub variants: Vec<String>,
}

pub enum SerializedValueContainerType {
    List,
    Map,
}

pub enum SerializedValuePrimitiveType {
    Null,
    Bool,
    Int,
    Float,
    String,
}