use std::{panic::catch_unwind, thread, time::{Duration, Instant}};

use fruits_engine::*;

fn main() {
    let result = catch_unwind(|| {
        run_samples_visualization_app();
    });

    if let Err(err) = result {
        if let Some(err) = err.downcast_ref::<&'static str>() {
            eprintln!("{}", err);
        } else if let Some(err) = err.downcast_ref::<String>() {
            eprintln!("{}", err);
        }
        thread::park();
    }
}

fn run_samples_visualization_app() {
    let mut app = App::new();

    let ecs = app.ecs_mut();

    add_render_module_to(ecs.as_mut());
    add_audio_module_to(ecs.as_mut());

    ecs.data_mut().resources_mut().insert(SamplesResource(Vec::new())).ok().unwrap();

    let clip = get_or_load_audio_clip_from_world(ecs.data_mut().resources_mut().as_mut(), "Through space.asset").unwrap();
    
    let mut data = ecs.data_mut();
    let mut ent = data.entities_mut();
    let audio_ent = ent.create_entity();
    ent.add_component(audio_ent, AudioSource {
        clip: clip.clone(),
        playback_time: 0.0,
        is_playing: true,
        should_force_playback_time: true,
        is_looped: true,
        volume: 1.0,
    }).ok().unwrap();

    let mut beh = ecs.behavior_mut();

    beh.get_mut(fruits_engine::Schedule::Start).insert_system(generate_samples);
    beh.get_mut(fruits_engine::Schedule::Update).insert_system(visualize_samples);
    beh.get_mut(fruits_engine::Schedule::Update).insert_system(try_restart_audio_sources);
    beh.get_mut(fruits_engine::Schedule::Update).order_system(visualize_samples).before_group(SYSTEM_GROUP_RENDER);

    app.run();
}

fn try_restart_audio_sources(
    mut last_restart: Local<LastAudioRestartResource>,
    mut sources_q: WorldQuery<&mut AudioSource>,
) {
    if last_restart.0.is_none() {
        last_restart.0 = Some(Instant::now());
        return;
    }

    let last_restart_instant = last_restart.0.unwrap();

    if last_restart_instant.elapsed() < Duration::from_secs_f32(0.1) {
        return;
    }

    last_restart.0 = Some(Instant::now());
    for source in sources_q.iter_mut() {
        // source.is_playing ^= true;
        // source.playback_time = 1.0;
        // source.should_force_playback_time = true;
    }
}

#[derive(SystemResource, Default)]
struct LastAudioRestartResource(Option<Instant>);

#[derive(Resource)]
struct SamplesResource(Vec<(Vec<f32>, Vec4<f32>)>);

fn generate_samples(mut samples: ResMut<SamplesResource>) {
    let sample_rate_original = 30;
    let sample_rate_resampled = 40;
    let sample_rate_detailed = 500;

    let samples_original = (0..sample_rate_original).into_iter().map(|_| rand::random::<f32>() * 2.0 - 1.0).collect::<Vec<_>>();
    let samples_resampled1 = resample(&samples_original, sample_rate_resampled, interpolate_linear);
    let samples_resampled2 = resample(&samples_original, sample_rate_resampled, interpolate_cubic);
    let samples_detailed1 = resample(&samples_original, sample_rate_detailed, interpolate_cubic);
    let samples_detailed2 = resample(&samples_resampled2, sample_rate_detailed, interpolate_cubic);

    samples.0.push((samples_original, Vec4::new(1.0, 1.0, 1.0, 1.0)));
    samples.0.push((samples_resampled1, Vec4::new(1.0, 1.0, 0.0, 1.0)));
    samples.0.push((samples_resampled2, Vec4::new(0.0, 1.0, 1.0, 1.0)));
    samples.0.push((samples_detailed1, Vec4::new(0.15, 0.0, 0.15, 1.0)));
    samples.0.push((samples_detailed2, Vec4::new(0.0, 0.15, 0.15, 1.0)));
}

fn interpolate_linear(pn1: f32, p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    lerp(p0, p1, t)
}

fn interpolate_cubic(pn1: f32, p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    let a = -pn1 * 0.5 + p0 * 1.5 - p1 * 1.5 + p2 * 0.5;
    let b = pn1 - p0 * 2.5 + p1 * 2.0 - p2 * 0.5;
    let c = -pn1 * 0.5 + p1 * 0.5;
    let d = p0;

    let tt = t * t;
    let ttt = tt * t;

    a * ttt + b * tt + c * t + d
}

fn resample<F: FnMut(f32, f32, f32, f32, f32) -> f32>(samples: &[f32], new_sample_rate: usize, mut resampler: F) -> Vec<f32> {
    let original_sample_rate = samples.len();
    let mut resampled_samples = Vec::with_capacity(new_sample_rate);

    for i in 0..new_sample_rate {
        let progress = i as f32 / (new_sample_rate - 1) as f32;

        let scaled_original_progress = progress * (original_sample_rate - 1) as f32;
        let j0 = (scaled_original_progress.floor() as usize).clamp(0, original_sample_rate - 1);
        let jt = scaled_original_progress - j0 as f32;
        let j1 = (j0 + 1).clamp(0, original_sample_rate - 1);
        let j2 = (j1 + 1).clamp(0, original_sample_rate - 1);
        let jn1 = (j0.max(1) - 1).clamp(0, original_sample_rate - 1);

        let resampled_sample = resampler(samples[jn1], samples[j0], samples[j1], samples[j2], jt);
        resampled_samples.push(resampled_sample);
    }

    resampled_samples
}

fn visualize_samples(
    samples: Res<SamplesResource>,
    audio: Res<AudioStateResource>,
    mut gizmo: ResMut<GizmosResource>,
) {
    let samples = audio.last_samples();

    if samples.len() <= 1 {
        return;
    }

    let scale_factor = 1.0_f32 / (samples.len().max(1) - 1) as f32;

    for s in 0..(samples.len() - 1) {
        let y0 = samples[s];
        let y1 = samples[s + 1];

        let x0 = -1.0 + s as f32 * scale_factor * 2.0;
        let x1 = -1.0 + (s + 1) as f32 * scale_factor * 2.0;

        gizmo.space(RenderSpace::Clip).push(GizmoLine {
            start: Vec3::new(x0, y0, 0.5),
            end: Vec3::new(x1, y1, 0.5),
            color: Vec4::new(0.5, 0.0, 0.5, 1.0),
        });
    }

    let multisamples_count = samples.len() / AUDIO_CHANNELS_COUNT;
    let scale_factor = 1.0_f32 / (multisamples_count.max(1) - 1) as f32;

    for s in 0..(multisamples_count - 1) {
        let mut y0 = 0.0;
        let mut y1 = 0.0;

        for c in 0..AUDIO_CHANNELS_COUNT {
            y0 += samples[s * AUDIO_CHANNELS_COUNT + c];
            y1 += samples[(s + 1) * AUDIO_CHANNELS_COUNT + c];
        }

        y0 /= AUDIO_CHANNELS_COUNT as f32;
        y1 /= AUDIO_CHANNELS_COUNT as f32;

        let x0 = -1.0 + s as f32 * scale_factor * 2.0;
        let x1 = -1.0 + (s + 1) as f32 * scale_factor * 2.0;

        gizmo.space(RenderSpace::Clip).push(GizmoLine {
            start: Vec3::new(x0, y0, 0.4),
            end: Vec3::new(x1, y1, 0.4),
            color: Vec4::new(1.0, 0.5, 1.0, 1.0),
        });
    }
}