use fruits_ecs::Event;
use winit::keyboard::SmolStr;

// todo: ffi
#[derive(Event)]
pub struct TextInputEvent(pub SmolStr);