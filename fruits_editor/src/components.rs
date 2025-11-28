use std::ffi::OsString;

use fruits_engine::prelude::*;

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
