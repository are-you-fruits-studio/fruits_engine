use fruits_ecs::Event;
use winit::keyboard::SmolStr;

#[derive(Event)]
pub struct TextInputEvent(pub SmolStr);