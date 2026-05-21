use std::path::PathBuf;

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_audio::{AudioClip, AudioStateResource};
use fruits_ecs::ResourcesHolderMut;
use fruits_ffi::FfiString;
use fruits_json::JsonValue;

pub fn get_or_load_audio_clip_from_world(res: ResourcesHolderMut, key: &str) -> Option<AssetHandle<AudioClip>> {
    let (audio_clips, audio_state) = unsafe {
        (
            &mut *res.get_ptr::<AssetStorageResource<AudioClip>>()?,
            &mut *res.get_ptr::<AudioStateResource>()?,
        )
    };

    get_or_load_audio_clip(audio_clips, audio_state, key)
}

pub fn get_or_load_audio_clip(
    audio_clips: &mut AssetStorageResource<AudioClip>,
    audio_state: &mut AudioStateResource,
    key: &str,
) -> Option<AssetHandle<AudioClip>> {
    if let Some(stored_audio_clip) = audio_clips.get_registered(key) {
        if audio_clips.get(stored_audio_clip).is_some() {
            return Some(stored_audio_clip.clone());
        }

        audio_clips.unregister(key);
    }

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(key);

    let audio_clip_json = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_err) => return None,
    };

    let Some(audio_clip) = deserialize_audio_clip(&audio_clip_json, audio_state) else {
        return None;
    };

    let audio_clip_handle = audio_clips.insert(audio_clip);

    audio_clips.register(FfiString::from_string(key.to_string()), audio_clip_handle.clone());

    Some(audio_clip_handle)
}

fn deserialize_audio_clip(data: &str, audio_state: &mut AudioStateResource) -> Option<AudioClip> {
    let Some(audio_clip_json) = JsonValue::parse(&mut data.chars()) else {
        return None;
    };

    let JsonValue::Object(audio_clip_asset) = audio_clip_json else {
        return None;
    };

    let Some(JsonValue::String(asset_type)) = audio_clip_asset.get_value("asset_type") else {
        return None;
    };

    if asset_type != "audio_clip" {
        return None;
    }

    let Some(JsonValue::String(raw_audio)) = audio_clip_asset.get_value("raw_audio") else {
        return None;
    };

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(raw_audio);

    read_audio_clip(path, audio_state)
}

fn read_audio_clip<P: AsRef<std::path::Path>>(path: P, audio_state: &mut AudioStateResource) -> Option<AudioClip> {
    let mut reader = match hound::WavReader::open(path) {
        Err(err) => {
            eprintln!("{}", err);
            return None;
        },
        Ok(r) => r,
    };

    let audio_spec = reader.spec();

    let channels = audio_spec.channels as usize;
    let sample_rate = audio_spec.sample_rate as usize;

    let max_value = (1 << (audio_spec.bits_per_sample - 1)) as f64;

    let raw_samples = match audio_spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>(),
        hound::SampleFormat::Int => reader.samples::<i32>().map(|s| s.map(|s| (s as f64 / max_value) as f32)).collect::<Result<Vec<_>, _>>()
    };

    let raw_samples = match raw_samples {
        Err(err) => {
            eprintln!("{}", err);
            return None;
        },
        Ok(s) => s,
    };

    let mut result = Vec::new();

    for chunk in raw_samples.chunks(channels) {
        let multi_sample = if channels == 1 {
            [chunk[0], chunk[0]]
        } else {
            [chunk[0], chunk[1]]
        };

        result.push(multi_sample[0]);
        result.push(multi_sample[1]);
    }

    if sample_rate != fruits_audio::AUDIO_SAMPLE_RATE {
        result = fruits_audio::resample_audio(&result, fruits_audio::AUDIO_CHANNELS_COUNT, sample_rate, fruits_audio::AUDIO_SAMPLE_RATE);
    }

    AudioClip::new(result.into(), audio_state)
}