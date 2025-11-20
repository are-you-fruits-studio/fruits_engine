use fruits_ecs::Resource;
use winit::window::{Fullscreen, Window};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FullscreenState {
    Windowed,
    Borderless,
    Exclusive,
}

#[derive(Copy, Clone)]
pub struct WindowState {
    pub fullscreen: FullscreenState,
}

impl WindowState {
    pub(crate) fn from_window(window: &Window) -> Self {
        Self {
            fullscreen: match window.fullscreen() {
                Some(Fullscreen::Borderless { .. }) => FullscreenState::Borderless,
                Some(Fullscreen::Exclusive { .. }) => FullscreenState::Exclusive,
                None => FullscreenState::Windowed,
            },
        }
    }

    pub(crate) fn apply_difference(prev: &WindowState, next: &WindowState, window: &Window) {
        if prev.fullscreen != next.fullscreen {
            if let Some(monitor) = window.current_monitor() && let Some(video_mode) = monitor.video_modes().next() {
                window.set_fullscreen(match next.fullscreen {
                    FullscreenState::Windowed => None,
                    FullscreenState::Borderless => Some(Fullscreen::Borderless(None)),
                    FullscreenState::Exclusive => Some(Fullscreen::Exclusive(video_mode)),
                });
            }
        }
    }
}

#[derive(Resource)]
pub struct WindowResource {
    last_state: WindowState,
    next_state: WindowState,
}

impl WindowResource {
    pub(crate) fn new(state: WindowState) -> Self {
        Self {
            last_state: state.clone(),
            next_state: state,
        }
    }

    pub fn prev_state(&self) -> &WindowState {
        &self.last_state
    }
    pub(crate) fn prev_state_mut(&mut self) -> &mut WindowState {
        &mut self.last_state
    }
    pub fn next_state(&self) -> &WindowState {
        &self.next_state
    }
    pub fn next_state_mut(&mut self) -> &mut WindowState {
        &mut self.next_state
    }
}