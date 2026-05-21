use std::{collections::HashMap, sync::{Arc, Mutex}};

use cpal::{OutputCallbackInfo, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use fruits_asset_storage::*;
use fruits_ecs::*;
use fruits_ffi::{FfiDroppable, FfiVec};

// todo:
// + read different bits_per_sample (and sample type - f32, i32, i16, i8) -> convert to f32
// + read different channels_count -> convert to 2 channels
// + read different sample_rate -> convert to 48kHz (use cubic interpolation)
// + playback different channels_count (1 or 2) -> average channels if needed
// + create convenient ecs API for playback
// + ffi
// + create convenient assets API
// - read different audio formats (wav, mp3, ogg, etc.)
// - playback different sample_rates -> resample to native
// - playback different bits_per_sample (and sample type - f32, i32, i16, i8) -> convert from f32
// - make audio spatial (3D) in virtual world

pub const AUDIO_SAMPLE_RATE: usize = 48000;
pub const AUDIO_CHANNELS_COUNT: usize = 2;

pub const SYSTEM_GROUP_AUDIO: &'static str = "fruits_audio";

pub fn add_audio_module_to(mut world: WorldBuilderMut) {
    let audio_state = Arc::new(Mutex::new(AudioState::new()));
    let stream = start_playback(Arc::clone(&audio_state));

    world
        .data_mut()
        .resources_mut()
        .insert(AudioStateResource {
            state: WrappedAudioStateHandle::new(Arc::clone(&audio_state)),
            _stream: FfiDroppable::new(stream),
            last_samples: FfiVec::new(),
            next_audio_clip_id: 1,
        })
        .ok()
        .unwrap();

    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<AudioClip>::new())
        .ok()
        .unwrap();

    world.behavior_mut()
        .get_mut(Schedule::Update)
        .group(SYSTEM_GROUP_AUDIO)
        .insert_child_system(audio_system);
}

#[repr(C)]
#[derive(Clone)]
pub struct AudioClip {
    // interleaved stereo float [-1.0; 1.0] 48kHz samples
    samples: FfiVec<f32>,
    id: u64,
}

impl AudioClip {
    pub fn new(samples: FfiVec<f32>, audio_state: &mut AudioStateResource) -> Option<Self> {
        if samples.len() % AUDIO_CHANNELS_COUNT as u64 != 0 {
            return None;
        }

        let id = {
            let id = audio_state.next_audio_clip_id;
            audio_state.next_audio_clip_id += 1;
            id
        };

        Some(Self {
            samples,
            id,
        })
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

#[repr(C)]
#[derive(Component)]
pub struct AudioSource {
    pub clip: AssetHandle<AudioClip>,
    pub playback_time: f64,
    pub should_force_playback_time: bool,
    pub is_playing: bool,
    pub is_looped: bool,
}

#[repr(C)]
#[derive(Resource)]
pub struct AudioStateResource {
    _stream: FfiDroppable,
    state: WrappedAudioStateHandle,
    last_samples: FfiVec<f32>,
    next_audio_clip_id: u64,
}

unsafe impl Send for AudioStateResource {}
unsafe impl Sync for AudioStateResource {}

impl AudioStateResource {
    pub fn last_samples(&self) -> &[f32] {
        &self.last_samples
    }
}

pub struct WrappedAudioStateHandle {
    state: FfiDroppable,
}

impl WrappedAudioStateHandle {
    fn new(state: Arc<Mutex<AudioState>>) -> Self {
        Self {
            state: FfiDroppable::new(state),
        }
    }

    unsafe fn as_raw(&self) -> &Arc<Mutex<AudioState>> {
        unsafe {
            &*(self.state.get() as *mut Arc<Mutex<AudioState>>)
        }
    }
}

struct AudioActivePlayback {
    clip: AudioClip,
    sample_index: usize,
    is_playing: bool,
    is_looped: bool,
}

struct AudioState {
    active_playbacks: HashMap<Entity, AudioActivePlayback>,
    last_played_samples: Vec<f32>,
}

impl AudioState {
    fn new() -> Self {
        Self {
            active_playbacks: HashMap::new(),
            last_played_samples: Vec::new(),
        }
    }
}

fn audio_system(
    mut state: ResMut<AudioStateResource>,
    mut source_q: WorldQuery<(Entity, &mut AudioSource)>,
    clips: Res<AssetStorageResource<AudioClip>>,
) {
    let mut last_samples = std::mem::replace(&mut state.last_samples, FfiVec::new());

    {
        let state = unsafe { state.state.as_raw() };
        let audio_state = &mut *state.lock().unwrap();
        
        last_samples.resize(audio_state.last_played_samples.len() as u64, 0.0);
        last_samples.copy_from_slice(&audio_state.last_played_samples);

        // remove redundant playbacks
        let mut playbacks_to_remove = Vec::new();
        for (entity, _) in &audio_state.active_playbacks {
            if source_q.get(*entity).is_none() {
                playbacks_to_remove.push(*entity);
            }
        }
        for playback_to_remove in playbacks_to_remove {
            audio_state.active_playbacks.remove(&playback_to_remove);
        }

        // add missing playbacks
        // sync all playbacks and components
        for (entity, source) in source_q.iter_mut() {
            let clip = clips.get(&source.clip);

            let playback = audio_state.active_playbacks
                .entry(entity)
                .or_insert_with(|| AudioActivePlayback {
                    clip: AudioClip {
                        samples: FfiVec::new(),
                        id: 0,
                    },
                    sample_index: 0,
                    is_playing: false,
                    is_looped: false,
                });

            let Some(clip) = clip else {
                source.should_force_playback_time = false;
                source.is_playing = false;
                source.playback_time = 0.0;
                continue;
            };

            if clip.id != playback.clip.id {
                playback.clip = clip.clone();
            }

            playback.is_playing = source.is_playing;
            playback.is_looped = source.is_looped;

            if source.should_force_playback_time {
                playback.sample_index = ((source.playback_time * AUDIO_SAMPLE_RATE as f64) as usize).clamp(0, playback.clip.samples.len() as usize - 1);
                source.should_force_playback_time = false;
            } else {
                source.playback_time = (playback.sample_index as f64 / AUDIO_SAMPLE_RATE as f64).clamp(0.0, 1.0);
            }

            if playback.sample_index >= playback.clip.samples.len() as usize / AUDIO_CHANNELS_COUNT {
                source.is_playing = false;
            }
        }
    }

    _ = std::mem::replace(&mut state.last_samples, last_samples);
}

fn start_playback(state: Arc<Mutex<AudioState>>) -> cpal::Stream {
    let host = cpal::default_host();

    // let input_device = host.default_input_device().unwrap();
    let output_device = host.default_output_device().unwrap();

    dbg!(output_device.description().unwrap().name());

    let output_config = StreamConfig {
        buffer_size: cpal::BufferSize::Default,
        channels: 2,
        sample_rate: 48000,
    };

    println!("Audio output config: channels: {}, sample_rate: {}, buffer_size: {:?}", output_config.channels, output_config.sample_rate, output_config.buffer_size);
    let channels_count = output_config.channels as usize;

    let stream = output_device.build_output_stream(
        &output_config,
        move |output_samples: &mut [f32], _: &OutputCallbackInfo| {
            for sample in output_samples.iter_mut() {
                *sample = 0.0;
            }
            let output_miltisamples_count = output_samples.len() / channels_count;

            {
                let state = &mut *state.lock().unwrap();

                for (_, playback) in &mut state.active_playbacks {
                    if !playback.is_playing {
                        continue;
                    }

                    // todo: reuse the same buffer from audio asset, not clone it (probably needs FfiArc to support ffi)
                    let playback_multisamples_count = playback.clip.samples.len() as usize / AUDIO_CHANNELS_COUNT;

                    for s in 0..output_miltisamples_count {
                        let handle_over_sample = |playback: &mut AudioActivePlayback| {
                            if playback.sample_index >= playback_multisamples_count {
                                if playback.is_looped {
                                    playback.sample_index %= playback_multisamples_count;
                                } else {
                                    playback.sample_index = playback_multisamples_count;
                                    playback.is_playing = false;
                                }
                            }
                        };

                        handle_over_sample(playback);

                        if playback.sample_index >= playback_multisamples_count {
                            break;
                        }

                        let playback_multisample = &(&*playback.clip.samples)[((playback.sample_index) * AUDIO_CHANNELS_COUNT)..((playback.sample_index + 1) * AUDIO_CHANNELS_COUNT)];
                        if channels_count == 1 {
                            output_samples[s] += playback_multisample.iter().sum::<f32>() / playback_multisample.len() as f32;
                        } else {
                            for c in 0..channels_count.min(AUDIO_CHANNELS_COUNT) {
                                output_samples[s * channels_count + c] += playback_multisample[c];
                            }
                        }

                        playback.sample_index += 1;

                        handle_over_sample(playback);
                    }
                }

                state.last_played_samples.resize(output_samples.len(), 0.0);
                state.last_played_samples.copy_from_slice(output_samples);
            }
        },
        move |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    ).unwrap();

    stream.play().unwrap();

    return stream;
}

pub fn resample_audio(samples: &[f32], channels: usize, old_sample_rate: usize, new_sample_rate: usize) -> Vec<f32> {
    let original_multisamples_count = samples.len() / channels;
    let new_multisamples_count = original_multisamples_count / old_sample_rate * new_sample_rate;
    let mut resampled_samples = Vec::with_capacity(new_sample_rate);

    for i in 0..new_multisamples_count {
        let progress = i as f64 / (new_multisamples_count - 1) as f64;

        let scaled_original_progress = progress * (original_multisamples_count - 1) as f64;
        let j0 = (scaled_original_progress.floor() as usize).clamp(0, original_multisamples_count - 1);
        let jt = (scaled_original_progress - j0 as f64) as f32;
        let j1 = (j0 + 1).clamp(0, original_multisamples_count - 1);
        let j2 = (j1 + 1).clamp(0, original_multisamples_count - 1);
        let jn1 = (j0.max(1) - 1).clamp(0, original_multisamples_count - 1);

        for c in 0..channels {
            let resampled_sample = interpolate_cubic(
                samples[jn1 * channels + c],
                samples[j0 * channels + c],
                samples[j1 * channels + c],
                samples[j2 * channels + c],
                jt,
            );

            resampled_samples.push(resampled_sample);
        }
    }

    resampled_samples
}

// todo: to math crate
fn interpolate_cubic(pn1: f32, p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    let a = -pn1 * 0.5 + p0 * 1.5 - p1 * 1.5 + p2 * 0.5;
    let b = pn1 - p0 * 2.5 + p1 * 2.0 - p2 * 0.5;
    let c = -pn1 * 0.5 + p1 * 0.5;
    let d = p0;

    let tt = t * t;
    let ttt = tt * t;

    a * ttt + b * tt + c * t + d
}