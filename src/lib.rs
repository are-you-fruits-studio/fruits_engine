
pub mod prelude {
    pub use fruits_app::*;
    pub use fruits_ecs::*;
    pub use fruits_modules::*;
    pub use fruits_utils::*;
    pub use fruits_math::*;
    pub use fruits_math_macros::*;
    pub use fruits_serialization::*;
    pub use fruits_reflection::*;
    pub use fruits_entry_point::*;
    pub use fruits_ffi::*;
    pub use fruits_control_flow_macros::*;
}

pub mod app {
    pub use fruits_app::*;
    pub use fruits_entry_point::*;
    pub use fruits_app_launcher::*;
}

pub mod ecs {
    pub use fruits_ecs::*;
}

pub mod modules {
    pub use fruits_modules::*;
}

pub mod utils {
    pub use fruits_utils::*;
}

pub mod math {
    pub use fruits_math::*;
    pub use fruits_math_macros::*;
}

pub mod serialization {
    pub use fruits_serialization::*;
}

pub mod reflection {
    pub use fruits_reflection::*;
}

pub mod asset {
    pub use fruits_asset::*;
}

pub mod ffi {
    pub use fruits_ffi::*;
}

pub mod control_flow {
    pub use fruits_control_flow_macros::*;
}