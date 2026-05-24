mod serialization_transitive;
mod serialization_registry;
mod serialization_impls;
mod serialization_model;

pub use fruits_serialization_macros::*;
pub use serialization_transitive::*;
pub use serialization_registry::*;
pub use serialization_impls::*;
pub use serialization_model::*;

// todo:
// + impls for standard types
// + macros
// - editor
// - ffi
// - names tuple/struct -> list/map
// - ecs resource (and other public APIs)
// - refactor?