use std::{collections::HashMap, path::Path, sync::Mutex};

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_audio::{AudioClip, AudioClipAssetMetadata, AudioStateResource};
use fruits_ecs::{ResourcesHolderMut, ResourcesHolderRef};
use fruits_ffi::FfiString;
use fruits_prefab::{Prefab, PrefabDependencies};
use fruits_render_core::{CoordinateSpaceType, RenderApiResource, StandardMaterial, StandardMesh, StandardMeshAssetMetadata, StandardTexture, StandardTextureAssetMetadata};
use fruits_serialization::*;

use crate::{AudioClipLoader, EntityTransSerializer, MaterialLoader, MeshLoader, PrefabLoader, TextureLoader, serialize_prefab_no_deps};

pub fn load_asset_transitively_from_world<R>(
    res: ResourcesHolderMut,
    assets_dir_path: impl AsRef<Path>,
    prefab_dependencies: Option<&mut PrefabDependencies>,
    f: impl FnOnce(TransSerializerRegistry, &SerializersResource) -> R,
) -> Option<R> {
    Some(unsafe {
        load_asset_transitively::<R>(
            &*res.get_ptr::<RenderApiResource>()?,
            &mut *res.get_ptr::<AudioStateResource>()?,
            &*res.get_ptr::<SerializersResource>()?,
            &mut *res.get_ptr::<AssetStorageResource<Prefab>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardTexture>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardMaterial>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardMesh>>()?,
            &mut *res.get_ptr::<AssetStorageResource<AudioClip>>()?,
            assets_dir_path,
            prefab_dependencies,
            f,
        )
    })
}

pub fn load_asset_transitively<R>(
    render_api: &RenderApiResource,
    audio_state: &mut AudioStateResource,
    serializers: &SerializersResource,
    prefabs: &mut AssetStorageResource<Prefab>,
    textures: &mut AssetStorageResource<StandardTexture>,
    materials: &mut AssetStorageResource<StandardMaterial>,
    meshes: &mut AssetStorageResource<StandardMesh>,
    audio_clips: &mut AssetStorageResource<AudioClip>,
    assets_dir_path: impl AsRef<Path>,
    prefab_dependencies: Option<&mut PrefabDependencies>,
    f: impl FnOnce(TransSerializerRegistry, &SerializersResource) -> R,
) -> R {
    let (
        audio_state,
        prefabs,
        textures,
        materials,
        meshes,
        audio_clips,
        deps,
    ) = (
        &Mutex::new(audio_state),
        &Mutex::new(prefabs),
        &Mutex::new(textures),
        &Mutex::new(materials),
        &Mutex::new(meshes),
        &Mutex::new(audio_clips),
        prefab_dependencies.map(Mutex::new),
    );

    let assets_dir_path = assets_dir_path.as_ref();

    let entities_deserialized = HashMap::new();
    let entities_serialized = HashMap::new();

    let mut serializer_local = TransSerializerRegistry::new();

    // todo: collect deps on serialization as well?
    let deps = deps.as_ref();

    serializer_local.register(EntityTransSerializer::new(&entities_deserialized, &entities_serialized));

    serializer_local.register(TransitiveLoadTransSerializer { 
        assets: textures,
        assets_dir_path,
        deps,
        loader: |mut ctx, value, assets_dir_path| {
            TextureLoader {
                render_api: render_api,
            // todo: use real path?
            }.load_from_deserialized(ctx.deserialize_default::<StandardTextureAssetMetadata>("", value), assets_dir_path)
        },
    });
    serializer_local.register(TransitiveLoadTransSerializer { 
        assets: materials,
        assets_dir_path,
        deps,
        loader: |mut ctx, value, _assets_dir_path| {
            // todo: use real path?
            let asset_metadata = ctx.deserialize_default("", value);
            MaterialLoader {
                render_api: render_api,
            }.load_from_deserialized(asset_metadata, &*textures.lock().unwrap())
        },
    });
    serializer_local.register(TransitiveLoadTransSerializer { 
        assets: meshes,
        assets_dir_path,
        deps,
        loader: |mut ctx, value, assets_dir_path| {
            MeshLoader {
                render_api: render_api,
            // todo: use real path?
            }.load_from_deserialized(ctx.deserialize_default("", value), assets_dir_path)
        },
    });
    serializer_local.register(TransitiveLoadTransSerializer { 
        assets: audio_clips,
        assets_dir_path,
        deps,
        loader: |mut ctx, value, assets_dir_path| {
            AudioClipLoader {
                audio_state: &mut *audio_state.lock().unwrap(),
            // todo: use real path?
            }.load_from_deserialized(ctx.deserialize_default("", value), assets_dir_path)
        },
    });
    // todo: fonts
    serializer_local.register(TransitiveLoadTransSerializer { 
        assets: prefabs,
        assets_dir_path,
        deps,
        loader: |ctx, value, assets_dir_path| {
            PrefabLoader.load_from_serialized(ctx, value)
        },
    });

    f(serializer_local, serializers)
}

//

pub fn save_with_asset_serializers_from_world<R>(
    res: ResourcesHolderRef,
    prefab_dependencies: Option<&mut PrefabDependencies>,
    f: impl FnOnce(TransSerializerRegistry) -> R,
) -> Option<R> {
    Some(
        save_with_asset_serializers::<R>(
            &*res.get::<AssetStorageResource<Prefab>>()?,
            &*res.get::<AssetStorageResource<StandardTexture>>()?,
            &*res.get::<AssetStorageResource<StandardMaterial>>()?,
            &*res.get::<AssetStorageResource<StandardMesh>>()?,
            &*res.get::<AssetStorageResource<AudioClip>>()?,
            prefab_dependencies,
            f,
        )
    )
}

pub fn save_with_asset_serializers<R>(
    prefabs: &AssetStorageResource<Prefab>,
    textures: &AssetStorageResource<StandardTexture>,
    materials: &AssetStorageResource<StandardMaterial>,
    meshes: &AssetStorageResource<StandardMesh>,
    audio_clips: &AssetStorageResource<AudioClip>,
    prefab_dependencies: Option<&mut PrefabDependencies>,
    f: impl FnOnce(TransSerializerRegistry) -> R,
) -> R {
    let deps = prefab_dependencies.map(Mutex::new);

    let entities_deserialized = HashMap::new();
    let entities_serialized = HashMap::new();

    let mut serializer_local = TransSerializerRegistry::new();

    // todo: collect deps on serialization as well?
    let deps = deps.as_ref();

    serializer_local.register(EntityTransSerializer::new(&entities_deserialized, &entities_serialized));

    serializer_local.register(AssetHandleLinkTransSerializer { assets: textures });
    serializer_local.register(AssetHandleLinkTransSerializer { assets: materials });
    serializer_local.register(AssetHandleLinkTransSerializer { assets: meshes });
    serializer_local.register(AssetHandleLinkTransSerializer { assets: audio_clips });
    // todo: font
    serializer_local.register(AssetHandleLinkTransSerializer { assets: prefabs });

    serializer_local.register(DirectAssetSaveTransSerializer {
        assets: textures,
        extractor: |_, a| a.meta().cloned().unwrap_or_default(),
    });
    serializer_local.register(DirectAssetSaveTransSerializer {
        assets: materials,
        extractor: |_, a| a.meta().clone(),
    });
    serializer_local.register(DirectAssetSaveTransSerializer {
        assets: meshes,
        extractor: |_, a| a.meta().cloned().unwrap_or_else(|| StandardMeshAssetMetadata {
            raw_mesh: Default::default(),
            coordinate_space: CoordinateSpaceType::LeftHandZForward,
            has_clockwise_winding: Default::default(),
            has_inverted_u: Default::default(),
            has_inverted_v: Default::default(),
        }),
    });
    serializer_local.register(DirectAssetSaveTransSerializer {
        assets: audio_clips,
        extractor: |_, a| a.meta().cloned().unwrap_or_else(|| AudioClipAssetMetadata {
            raw_audio: Default::default(),
        }),
    });
    // todo: fonts
    serializer_local.register(DirectAssetSaveTransSerializer {
        assets: prefabs,
        extractor: |ctx, p| serialize_prefab_no_deps(ctx, &p.entities).clone(),
    });

    f(serializer_local)
}

//

pub fn load_asset_single_from_world<R>(
    res: ResourcesHolderMut,
    assets_dir_path: impl AsRef<Path>,
    asset_key: &str,
    prefab_dependencies: Option<&mut PrefabDependencies>,
    f: impl FnOnce(TransSerializerRegistry, &SerializersResource) -> R,
) -> Option<R> {
    Some(unsafe {
        load_asset_single::<R>(
            &*res.get_ptr::<RenderApiResource>()?,
            &mut *res.get_ptr::<AudioStateResource>()?,
            &*res.get_ptr::<SerializersResource>()?,
            &mut *res.get_ptr::<AssetStorageResource<Prefab>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardTexture>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardMaterial>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardMesh>>()?,
            &mut *res.get_ptr::<AssetStorageResource<AudioClip>>()?,
            assets_dir_path,
            asset_key,
            prefab_dependencies,
            f,
        )
    })
}

pub fn load_asset_single<R>(
    render_api: &RenderApiResource,
    audio_state: &mut AudioStateResource,
    serializers: &SerializersResource,
    prefabs: &mut AssetStorageResource<Prefab>,
    textures: &mut AssetStorageResource<StandardTexture>,
    materials: &mut AssetStorageResource<StandardMaterial>,
    meshes: &mut AssetStorageResource<StandardMesh>,
    audio_clips: &mut AssetStorageResource<AudioClip>,
    assets_dir_path: impl AsRef<Path>,
    asset_key: &str,
    prefab_dependencies: Option<&mut PrefabDependencies>,
    f: impl FnOnce(TransSerializerRegistry, &SerializersResource) -> R,
) -> R {
    let (
        audio_state,
        prefabs,
        textures,
        materials,
        meshes,
        audio_clips,
        deps,
    ) = (
        &Mutex::new(audio_state),
        &Mutex::new(prefabs),
        &Mutex::new(textures),
        &Mutex::new(materials),
        &Mutex::new(meshes),
        &Mutex::new(audio_clips),
        prefab_dependencies.map(Mutex::new),
    );

    let assets_dir_path = assets_dir_path.as_ref();

    let entities_deserialized = HashMap::new();
    let entities_serialized = HashMap::new();

    let mut serializer_local = TransSerializerRegistry::new();

    // todo: collect deps on serialization as well?
    let deps = deps.as_ref();

    serializer_local.register(EntityTransSerializer::new(&entities_deserialized, &entities_serialized));

    serializer_local.register(SingleLoadTransSerializer { assets: textures, deps });
    serializer_local.register(SingleLoadTransSerializer { assets: materials, deps });
    serializer_local.register(SingleLoadTransSerializer { assets: meshes, deps });
    serializer_local.register(SingleLoadTransSerializer { assets: audio_clips, deps });
    // todo: font
    serializer_local.register(SingleLoadTransSerializer { assets: prefabs, deps });

    serializer_local.register(SingleDirectLoadTransSerializer { 
        assets: textures,
        assets_dir_path,
        asset_key,
        deps,
        loader: |mut ctx, value, assets_dir_path| {
            TextureLoader {
                render_api: render_api,
            // todo: use real path?
            }.load_from_deserialized(ctx.deserialize_default("", value), assets_dir_path)
        },
    });
    serializer_local.register(SingleDirectLoadTransSerializer { 
        assets: materials,
        assets_dir_path,
        asset_key,
        deps,
        loader: |mut ctx, value, _assets_dir_path| {
            // todo: use real path?
            let asset_metadata = ctx.deserialize_default("", value);
            MaterialLoader {
                render_api: render_api,
            }.load_from_deserialized(asset_metadata, &*textures.lock().unwrap())
        },
    });
    serializer_local.register(SingleDirectLoadTransSerializer { 
        assets: meshes,
        assets_dir_path,
        asset_key,
        deps,
        loader: |mut ctx, value, assets_dir_path| {
            MeshLoader {
                render_api: render_api,
            // todo: use real path?
            }.load_from_deserialized(ctx.deserialize_default("", value), assets_dir_path)
        },
    });
    serializer_local.register(SingleDirectLoadTransSerializer { 
        assets: audio_clips,
        assets_dir_path,
        asset_key,
        deps,
        loader: |mut ctx, value, assets_dir_path| {
            AudioClipLoader {
                audio_state: &mut *audio_state.lock().unwrap(),
            // todo: use real path?
            }.load_from_deserialized(ctx.deserialize_default("", value), assets_dir_path)
        },
    });
    // todo: fonts
    serializer_local.register(SingleDirectLoadTransSerializer { 
        assets: prefabs,
        assets_dir_path,
        asset_key,
        deps,
        loader: |ctx, value, assets_dir_path| {
            PrefabLoader.load_from_serialized(ctx, value)
        },
    });

    f(serializer_local, serializers)
}

//

// todo: use path where appropriate

#[repr(C)]
pub enum DirectSerializableAsset<T> {
    Handle(AssetHandle<T>),
    Key(FfiString),
}

pub struct AssetHandleLinkTransSerializer<'m, T: 'static> {
    assets: &'m AssetStorageResource<T>,
}
impl<'m, T: 'static, S: Copy> Serializer<S> for AssetHandleLinkTransSerializer<'m, T> {
    type Deserialized = AssetHandle<T>;

    fn serialize(&self, value: &Self::Deserialized, mut ctx: SerializerCtx<S>, path: &str) -> SerializedValue {
        let key = self.assets.get_registration(value);

        if key.is_none() {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle is not registered".into() });
        }

        SerializedValue::Primitive(SerializedPrimitive::String(key.unwrap_or("").into()))
    }

    fn deserialize(&self, value: &mut Self::Deserialized, mut ctx: SerializerCtx<S>, path: &str, serialized: &SerializedValue) {
        *value = Default::default();

        let SerializedValue::Primitive(SerializedPrimitive::String(key)) = serialized else {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle can only be deserialized from string".into() });
            return;
        };

        let Some(stored_asset) = self.assets.get_registered(key) else {
            ctx.report_err(SerializationError::InvalidInput { message: format!("AssetHandle with a key \"{key}\" not found").into() });
            return;
        };

        *value = stored_asset.clone()
    }
    
    fn serializable_default(&self) -> Self::Deserialized {
        Default::default()
    }
}

pub struct DirectAssetSaveTransSerializer<'m, T: 'static, S: 'static, F: Fn(SerializerCtx<TransSerializerCtxState>, &T) -> S> {
    assets: &'m AssetStorageResource<T>,
    extractor: F,
}
impl<'a, 'm, T: 'static, S: 'static, F: Fn(SerializerCtx<TransSerializerCtxState>, &T) -> S> Serializer<TransSerializerCtxState<'a>> for DirectAssetSaveTransSerializer<'m, T, S, F> {
    type Deserialized = DirectSerializableAsset<T>;

    fn serialize(&self, value: &Self::Deserialized, mut ctx: SerializerCtx<TransSerializerCtxState>, path: &str) -> SerializedValue {
        let serializable_part = {
            let assets = self.assets;

            let asset_handle = match value {
                DirectSerializableAsset::Handle(asset_handle) => asset_handle,
                DirectSerializableAsset::Key(key) => {
                    let Some(asset) = assets.get_registered(key.as_str()) else {
                        ctx.report_err(SerializationError::InvalidInput { message: format!("failed to serialize asset {}, Asset key is missing {}", std::any::type_name::<T>(), key.as_str()).into() });
                        return SerializedValue::Null;
                    };

                    asset
                },
            };

            let Some(asset) = assets.get(asset_handle) else {
                println!("failed to serialize asset {}, AssetHandle is invalid", std::any::type_name::<T>());
                return SerializedValue::Null;
            };

            (self.extractor)(ctx.as_mut(), asset)
        };

        ctx.serialize(&serializable_part, path)
    }

    fn deserialize(&self, value: &mut Self::Deserialized, mut ctx: SerializerCtx<TransSerializerCtxState>, path: &str, serialized: &SerializedValue) {
        todo!()
    }
    
    fn serializable_default(&self) -> Self::Deserialized {
        todo!()
    }
}

fn get_from_asset_storage_or_unregister_if_missing<T: 'static>(
    storage: &mut AssetStorageResource<T>,
    key: &str,
) -> Option<AssetHandle<T>> {
    let Some(stored_asset) = storage.get_registered(key) else {
        return None;
    };

    if storage.get(stored_asset).is_some() {
        return Some(stored_asset.clone());
    }

    storage.unregister(key);
    None
}

fn try_read_serialized_from_file(assets_dir_path: impl AsRef<Path>, key: &str) -> Option<SerializedValue> {
    let mut path = assets_dir_path.as_ref().to_path_buf();
    path.push(key);

    let raw_asset = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_err) => return None,
    };

    let value = SerializedValue::from_json(&serde_json::from_str::<serde_json::Value>(&raw_asset).ok()?);

    Some(value)
}

pub struct TransitiveLoadTransSerializer<'m, 'brw: 'm, T: 'static, F: 'm + Send + Sync + Fn(SerializerCtx<TransSerializerCtxState>, &SerializedValue, &Path) -> Option<T>> {
    pub assets: &'m Mutex<&'brw mut AssetStorageResource<T>>,
    pub assets_dir_path: &'m Path,
    pub deps: Option<&'m Mutex<&'brw mut PrefabDependencies>>,
    pub loader: F,
}
impl<'a, 'm, 'brw: 'm, T: 'static, F: 'm + Send + Sync + Fn(SerializerCtx<TransSerializerCtxState>, &SerializedValue, &Path) -> Option<T>> Serializer<TransSerializerCtxState<'a>> for TransitiveLoadTransSerializer<'m, 'brw, T, F> {
    type Deserialized = AssetHandle<T>;

    fn serialize(&self, value: &Self::Deserialized, mut ctx: SerializerCtx<TransSerializerCtxState>, path: &str) -> SerializedValue {
        let asset_storage = self.assets.lock().unwrap();
        let key = asset_storage.get_registration(value);

        if key.is_none() {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle is not registered".into() });
        }

        SerializedValue::Primitive(SerializedPrimitive::String(key.unwrap_or("").into()))
    }

    fn deserialize(&self, value: &mut Self::Deserialized, mut ctx: SerializerCtx<TransSerializerCtxState>, path: &str, serialized: &SerializedValue) {
        *value = Default::default();

        let SerializedValue::Primitive(SerializedPrimitive::String(key)) = serialized else {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle can only be deserialized from string".into() });
            return;
        };

        if key.as_str().trim().is_empty() {
            return;
        }

        if let Some(handle) = { get_from_asset_storage_or_unregister_if_missing(&mut self.assets.lock().unwrap(), key) } {
            *value = handle;
            return;
        }

        let Some(serialized) = try_read_serialized_from_file(self.assets_dir_path, key) else {
            ctx.report_err(SerializationError::InvalidInput { message: format!("AssetHandle failed to be deserialized from key {key}").into() });
            return;
        };

        let Some(asset) = (self.loader)(ctx.as_mut(), &serialized, self.assets_dir_path) else {
            return;
        };
        
        *value = self.assets.lock().unwrap().insert_and_register(asset, key.clone());
    }
    
    fn serializable_default(&self) -> Self::Deserialized {
        Default::default()
    }
}

pub struct DirectDeserializedAsset<T> {
    handle: AssetHandle<T>,
    key: FfiString,
}

impl<T> Default for DirectDeserializedAsset<T> {
    fn default() -> Self {
        Self {
            handle: Default::default(),
            key: Default::default(),
        }
    }
}

pub struct SingleDirectLoadTransSerializer<'m, 'brw: 'm, T: 'static, F: 'm + Send + Sync + Fn(SerializerCtx<TransSerializerCtxState>, &SerializedValue, &Path) -> Option<T>> {
    pub assets: &'m Mutex<&'brw mut AssetStorageResource<T>>,
    pub assets_dir_path: &'m Path,
    pub asset_key: &'m str,
    pub deps: Option<&'m Mutex<&'brw mut PrefabDependencies>>,
    pub loader: F,
}
impl<'a, 'm, 'brw: 'm, T: 'static, F: 'm + Send + Sync + Fn(SerializerCtx<TransSerializerCtxState>, &SerializedValue, &Path) -> Option<T>> Serializer<TransSerializerCtxState<'a>> for SingleDirectLoadTransSerializer<'m, 'brw, T, F> {
    type Deserialized = DirectDeserializedAsset<T>;

    fn serialize(&self, value: &Self::Deserialized, mut ctx: SerializerCtx<TransSerializerCtxState>, path: &str) -> SerializedValue {
        let asset_storage = self.assets.lock().unwrap();
        let key = asset_storage.get_registration(&value.handle);

        if key.is_none() {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle is not registered".into() });
        }

        SerializedValue::Primitive(SerializedPrimitive::String(key.unwrap_or("").into()))
    }

    fn deserialize(&self, value: &mut Self::Deserialized, mut ctx: SerializerCtx<TransSerializerCtxState>, path: &str, serialized: &SerializedValue) {
        let Some(asset) = (self.loader)(ctx.as_mut(), &serialized, self.assets_dir_path) else {
            *value = self.serializable_default();
            return;
        };

        let asset_key: FfiString = self.asset_key.into();
        let asset_handle = self.assets.lock().unwrap().insert_and_register(asset, asset_key.clone());

        *value = DirectDeserializedAsset {
            handle: asset_handle,
            key: asset_key,
        };
    }
    
    fn serializable_default(&self) -> Self::Deserialized {
        DirectDeserializedAsset {
            handle: AssetHandle::EMPTY,
            key: Default::default(),
        }
    }
}
pub struct SingleLoadTransSerializer<'m, 'brw: 'm, T: 'static> {
    pub assets: &'m Mutex<&'brw mut AssetStorageResource<T>>,
    pub deps: Option<&'m Mutex<&'brw mut PrefabDependencies>>,
}
impl<'m, 'brw: 'm, T: 'static, S: Copy> Serializer<S> for SingleLoadTransSerializer<'m, 'brw, T> {
    type Deserialized = AssetHandle<T>;

    fn serialize(&self, value: &Self::Deserialized, mut ctx: SerializerCtx<S>, path: &str) -> SerializedValue {
        let asset_storage = self.assets.lock().unwrap();
        let key = asset_storage.get_registration(&value);

        if key.is_none() {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle is not registered".into() });
        }

        SerializedValue::Primitive(SerializedPrimitive::String(key.unwrap_or("").into()))
    }

    fn deserialize(&self, value: &mut Self::Deserialized, mut ctx: SerializerCtx<S>, path: &str, serialized: &SerializedValue) {
        *value = Default::default();

        let SerializedValue::Primitive(SerializedPrimitive::String(key)) = serialized else {
            ctx.report_err(SerializationError::InvalidInput { message: "AssetHandle can only be deserialized from string".into() });
            return;
        };

        *value = self.assets.lock().unwrap().get_registered(key.as_str()).cloned().unwrap_or_default()
    }
    
    fn serializable_default(&self) -> Self::Deserialized {
        Default::default()
    }
}
