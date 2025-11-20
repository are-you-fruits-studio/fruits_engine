use std::sync::Arc;

use fruits_ecs::{Schedule, World, WorldBuilder};
use fruits_modules::RenderApiResource;
use gilrs::Gilrs;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{InputResource, WindowResource, WindowState};

enum EventLoopHandlerState {
    Created(WorldBuilder),
    Starting,
    Polling {
        world: World,
        window: Arc<Window>,
        gamepad_host: Option<Gilrs>,
    },
}

impl EventLoopHandlerState {
    fn try_start_creating(&mut self) -> Option<WorldBuilder> {
        let EventLoopHandlerState::Created { .. } = self else {
            return None;
        };

        let EventLoopHandlerState::Created(world) = std::mem::replace(self, EventLoopHandlerState::Starting) else {
            return None;
        };

        Some(world)
    }
}

pub struct EventLoopHandler(EventLoopHandlerState);

impl EventLoopHandler {
    pub fn new(world: WorldBuilder) -> Self {
        Self(EventLoopHandlerState::Created(world))
    }
}

impl ApplicationHandler for EventLoopHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(mut world) = self.0.try_start_creating() else {
            return;
        };

        let window = Arc::new(event_loop.create_window(WindowAttributes::default()).unwrap());

        let state = RenderApiResource::new(Arc::clone(&window));

        let window_state = WindowState::from_window(&window);

        world.data_mut().resources_mut().insert(state).ok().unwrap();
        world.data_mut().resources_mut().insert(InputResource::new()).ok().unwrap();
        world.data_mut().resources_mut().insert(WindowResource::new(window_state)).ok().unwrap();
        let mut world = world.build();
        world.execute_iteration(Schedule::Start);

        let gamepad_host = Gilrs::new();

        if let Err(err) = &gamepad_host {
            eprintln!("ayf: Failed to init Gilrs(gamepad). {}", err);
        }

        self.0 = EventLoopHandlerState::Polling {
            world,
            window,
            gamepad_host: gamepad_host.ok(),
        };
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        let EventLoopHandlerState::Polling { .. } = &mut self.0 else {
            return;
        };

        match event {
            // todo
            // DeviceEvent::MouseMotion { delta } => {
            //     let mut input = world.data().resources().get_mut::<InputResource>().unwrap();

            //     input.mouse.position[0] += delta.0;
            //     input.mouse.position[1] += delta.1;
            // },
            _ => (),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let EventLoopHandlerState::Polling {
            world,
            window,
            gamepad_host,
        } = &mut self.0
        else {
            return;
        };

        let mut world_data = world.data_mut();
        let mut res = world_data.resources_mut();

        let render_state = res.get_mut::<RenderApiResource>().unwrap();

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                let input = res.get_mut::<InputResource>().unwrap();

                input.mouse.position = [position.x, position.y];
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let input = res.get_mut::<InputResource>().unwrap();

                match state {
                    ElementState::Pressed => input.mouse.press(button),
                    ElementState::Released => input.mouse.release(button),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat {
                    return;
                }

                let PhysicalKey::Code(key_code) = event.physical_key else {
                    return;
                };

                let input = res.get_mut::<InputResource>().unwrap();

                match event.state {
                    ElementState::Pressed => input.keyboard.press(key_code),
                    ElementState::Released => input.keyboard.release(key_code),
                }
            }
            WindowEvent::Resized(new_size) => {
                render_state.resize([new_size.width, new_size.height]);
            }
            WindowEvent::RedrawRequested => {
                let input = res.get_mut::<InputResource>().unwrap();

                if let Some(gamepad_host) = gamepad_host {
                    while let Some(gamepad_evt) = gamepad_host.next_event() {
                        match gamepad_evt.event {
                            gilrs::EventType::ButtonPressed(button, _) => input.gamepad.press(button),
                            gilrs::EventType::ButtonReleased(button, _) => input.gamepad.release(button),
                            gilrs::EventType::AxisChanged(axis, axis_value, _) => input.gamepad.write_axis(axis, axis_value),
                            // todo: handle other gamepad events
                            _ => (),
                        }
                    }
                }

                // println!("ayf: redraw start");
                world.execute_iteration(Schedule::Update);

                world.data_mut().events_mut().clear();

                let mut world_data = world.data_mut();
                let mut res = world_data.resources_mut();

                let input = res.get_mut::<InputResource>().unwrap();
                input.keyboard.clear_frame();
                input.mouse.clear_frame();
                input.gamepad.clear_frame();

                let window_res = res.get_mut::<WindowResource>().unwrap();

                let window_state_next = window_res.next_state().clone();
                let window_state_prev = std::mem::replace(window_res.prev_state_mut(), window_state_next.clone());

                WindowState::apply_difference(&window_state_prev, &window_state_next, &window);

                window.request_redraw();
                // println!("ayf: redraw end");
            }
            WindowEvent::Destroyed => {
                // todo
            }
            _ => {}
        }
    }
}
