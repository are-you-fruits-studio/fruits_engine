use std::{hint::black_box, marker::PhantomData, sync::{LazyLock, Mutex}};

use fruits_prelude::{App, Component, ExclusiveWorldAccess, Schedule};

fn main() {
    let mut app = App::new();

    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(update_system);

    app.run();
}

fn update_system(
    mut w: ExclusiveWorldAccess
) {
    let ec = w.entities_components_mut();

    let e = ec.create_entity();

    ec.add_component(e, MaliciousComponent(Vec::with_capacity(8)));
    //ec.add_component(e, LoudComponent::new());

    // ec.remove_component::<MaliciousComponent>(e);

    ec.destroy_entity(e);
}

#[derive(Component)]
struct MaliciousComponent(Vec<u8>);

#[derive(Component)]
struct LoudComponent {
    id: usize,
}

static LOUD_COUNTER: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));

impl LoudComponent {
    pub fn new() -> Self {
        let mut counter = LOUD_COUNTER.lock().unwrap();
        let id = *counter;
        *counter += 1;

        println!("louddddd created {}", id);
        Self {
            id,
        }
    }
}

impl Drop for LoudComponent {
    fn drop(&mut self) {
        println!("louddddd destroyed {}", self.id);
    }
}


