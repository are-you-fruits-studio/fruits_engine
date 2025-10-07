use std::collections::{HashMap, HashSet};
use fruits_ecs::Resource;
use gilrs::Axis;

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
pub use gilrs::Button as GamepadButton;
pub use gilrs::Axis as GamepadAxis;

// todo: support ffi
#[derive(Resource)]
pub struct InputResource {
    pub keyboard: KeyboardInputStorage,
    pub mouse: MouseInputStorage,
    pub gamepad: GamepadInputStorage,
}

impl InputResource {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardInputStorage::new(),
            mouse: MouseInputStorage::new(),
            gamepad: GamepadInputStorage::new(),
        }
    }
}

//

pub struct KeyboardInputStorage {
    pressed: HashSet<KeyCode>,
    frame_pressed: HashSet<KeyCode>,
    frame_released: HashSet<KeyCode>,
}

impl KeyboardInputStorage {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            frame_pressed: HashSet::new(),
            frame_released: HashSet::new(),
        }
    }
    
    pub fn is_pressed(&self, k: KeyCode) -> bool {
        self.pressed.contains(&k)
    }
    
    pub fn is_just_pressed(&self, k: KeyCode) -> bool {
        self.frame_pressed.contains(&k)
    }
    
    pub fn is_just_released(&self, k: KeyCode) -> bool {
        self.frame_released.contains(&k)
    }
    
    pub fn press(&mut self, k: KeyCode) {
        self.pressed.insert(k);
        self.frame_pressed.insert(k);
    }
    
    pub fn release(&mut self, k: KeyCode) {
        self.pressed.remove(&k);
        self.frame_released.insert(k);
    }
    
    pub fn clear(&mut self) {
        self.pressed.clear();
        self.frame_pressed.clear();
        self.frame_released.clear();
    }

    pub fn clear_frame(&mut self) {
        self.frame_pressed.clear();
        self.frame_released.clear();
    }
}

//

pub struct MouseInputStorage {
    pressed: HashSet<MouseButton>,
    frame_pressed: HashSet<MouseButton>,
    frame_released: HashSet<MouseButton>,
    pub position: [f64; 2],
}

impl MouseInputStorage {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            frame_pressed: HashSet::new(),
            frame_released: HashSet::new(),
            position: [0.0; 2],
        }
    }
    
    pub fn is_pressed(&self, k: MouseButton) -> bool {
        self.pressed.contains(&k)
    }
    
    pub fn is_just_pressed(&self, k: MouseButton) -> bool {
        self.frame_pressed.contains(&k)
    }
    
    pub fn is_just_released(&self, k: MouseButton) -> bool {
        self.frame_released.contains(&k)
    }

    pub fn press(&mut self, k: MouseButton) {
        self.pressed.insert(k);
        self.frame_pressed.insert(k);
    }
    
    pub fn release(&mut self, k: MouseButton) {
        self.pressed.remove(&k);
        self.frame_released.insert(k);
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
        self.frame_pressed.clear();
        self.frame_released.clear();
    }

    pub fn clear_frame(&mut self) {
        self.frame_pressed.clear();
        self.frame_released.clear();
    }
}

//

pub struct GamepadInputStorage {
    pressed: HashSet<GamepadButton>,
    frame_pressed: HashSet<GamepadButton>,
    frame_released: HashSet<GamepadButton>,
    axes: HashMap<Axis, f32>,
}

impl GamepadInputStorage {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            frame_pressed: HashSet::new(),
            frame_released: HashSet::new(),
            axes: HashMap::new(),
        }
    }
    
    pub fn is_pressed(&self, k: GamepadButton) -> bool {
        self.pressed.contains(&k)
    }
    
    pub fn is_just_pressed(&self, k: GamepadButton) -> bool {
        self.frame_pressed.contains(&k)
    }
    
    pub fn is_just_released(&self, k: GamepadButton) -> bool {
        self.frame_released.contains(&k)
    }

    pub fn read_axis(&self, a: Axis) -> f32 {
        self.axes.get(&a).copied().unwrap_or_default()
    }

    pub fn write_axis(&mut self, a: Axis, v: f32) {
        self.axes.insert(a, v);
    }
    
    pub fn press(&mut self, k: GamepadButton) {
        self.pressed.insert(k);
        self.frame_pressed.insert(k);
    }
    
    pub fn release(&mut self, k: GamepadButton) {
        self.pressed.remove(&k);
        self.frame_released.insert(k);
    }
    
    pub fn clear(&mut self) {
        self.pressed.clear();
        self.axes.clear();
        self.frame_pressed.clear();
        self.frame_released.clear();
    }

    pub fn clear_frame(&mut self) {
        self.frame_pressed.clear();
        self.frame_released.clear();
    }
}