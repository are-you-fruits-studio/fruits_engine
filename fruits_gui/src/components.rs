use fruits_engine::prelude::*;

#[derive(Component)]
pub struct ProjectWindowContentComponent;

#[derive(Component, Debug, Clone)]
pub struct ButtonComponent;

#[derive(Component, Debug, Clone)]
pub struct DebugNameComponent(pub String);