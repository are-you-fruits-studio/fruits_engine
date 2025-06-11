use std::{collections::VecDeque, time::Instant};

use fruits_ecs::{ResMut, Resource, Schedule, WorldBuilder};


pub fn add_module_to(world: &mut WorldBuilder) {
    world.data_mut().resources_mut().insert(FpsResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).add_system(count_fps);
}

#[derive(Resource, Default)]
pub struct FpsResource {
    frame_times: VecDeque<f64>,
    last_frame_time: Option<Instant>,
    last_print_time: Option<Instant>,
}

pub fn count_fps(
    mut res: ResMut<FpsResource>,
) {
    const MAX_FRAMES: usize = 100;
    const PRINT_PERIOD_S: f64 = 1.0;

    if res.frame_times.len() >= MAX_FRAMES {
        res.frame_times.pop_front();
    }

    let last_frame_time = res.last_frame_time;
    
    res.last_frame_time = Some(Instant::now());

    let Some(last_frame_time) = last_frame_time else {
        return;
    };
    
    let duration = last_frame_time.elapsed();
    res.frame_times.push_back(duration.as_secs_f64());

    if res.frame_times.is_empty() {
        return;
    }

    let Some(last_print_time) = res.last_print_time else {
        res.last_print_time = Some(Instant::now());
        return;
    };

    if last_print_time.elapsed().as_secs_f64() < PRINT_PERIOD_S {
        return;
    }
    
    let avg_frame_time_s = res.frame_times.iter().sum::<f64>() / res.frame_times.len() as f64;
    let avg_frame_time_ms = avg_frame_time_s * 1000.0;
    let fps = (1.0 / avg_frame_time_s) as usize;

    println!("fps: {: >5} | frame_time: {: >10.2} ms", fps, avg_frame_time_ms);
    res.last_print_time = Some(Instant::now());
}