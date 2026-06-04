//! # fruits_asset_storage
//!
//! In-memory storage for engine assets (meshes, textures, materials, audio clips,
//! prefabs, fonts), addressed by lightweight handles instead of by ownership or
//! pointers.
//!
//! # How to use
//!
//! #### Storing an asset and keeping a handle to it
//!
//! Insert a value into an [`AssetStorageResource<T>`] to take ownership of it and
//! receive an [`AssetHandle<T>`]. The handle is small, [`Copy`]-cheap to clone, and
//! safe to store in components; the storage keeps the asset itself.
//!
//! ```
//! use fruits_asset_storage::AssetStorageResource;
//!
//! let mut clips: AssetStorageResource<String> = AssetStorageResource::new();
//!
//! let handle = clips.insert("explosion.wav".to_string());
//! assert_eq!(clips.get(&handle).map(String::as_str), Some("explosion.wav"));
//!
//! // Removing returns the asset and invalidates the handle.
//! assert_eq!(clips.remove(&handle).as_deref(), Some("explosion.wav"));
//! assert!(clips.get(&handle).is_none());
//! ```
//!
//! #### Looking an asset up by a string key
//!
//! Register a handle under a string key (typically the asset's source path) so the
//! same asset can be found again without re-inserting it. Asset loaders use this to
//! deduplicate: check [`get_registered`](AssetStorageResource::get_registered)
//! first, and only insert when the key is absent.
//!
//! ```
//! use fruits_asset_storage::AssetStorageResource;
//! use fruits_ffi::FfiString;
//!
//! let mut clips: AssetStorageResource<String> = AssetStorageResource::new();
//!
//! let handle = clips.insert("explosion.wav".to_string());
//! clips.register(FfiString::from_string("sfx/explosion".to_string()), handle.clone());
//!
//! assert_eq!(clips.get_registered("sfx/explosion"), Some(&handle));
//! assert_eq!(clips.get_registered("sfx/missing"), None);
//! ```
//!
//! #### Reading assets from inside a system
//!
//! Each asset type lives as its own ECS resource, so a system pulls the storage in
//! with [`Res`](fruits_ecs::Res) / [`ResMut`](fruits_ecs::ResMut) and resolves the
//! handles held by its components.
//!
//! ```no_run
//! use fruits_asset_storage::{AssetHandle, AssetStorageResource};
//! use fruits_ecs::Res;
//!
//! struct AudioClip;
//!
//! fn play_system(clips: Res<AssetStorageResource<AudioClip>>, handle: AssetHandle<AudioClip>) {
//!     if let Some(_clip) = clips.get(&handle) {
//!         // use the resolved asset
//!     }
//! }
//! ```
//!
//! # How to maintain
//!
//! #### Handles and stale-handle safety
//!
//! An [`AssetHandle<T>`] is `#[repr(transparent)]` over a
//! [`VersionIndex`]: a slot
//! index plus a version counter. Assets are stored in a
//! [`VersionCollection<T>`](fruits_utils::index_version_collection::VersionCollection),
//! which reuses freed slots and bumps the slot's version on
//! [`remove`](AssetStorageResource::remove). A handle to a slot that was removed
//! (and possibly re-filled) carries the old version, so
//! [`get`](AssetStorageResource::get) sees the version mismatch and returns `None`
//! rather than a different asset. This is what makes handles safe to keep around
//! after the asset they pointed to is gone.
//!
//! The `PhantomData<Arc<T>>` field carries no runtime cost and performs no reference
//! counting; it only marks the handle as logically referring to a `T`. All of
//! `AssetHandle`'s trait impls ([`Hash`], [`Eq`], [`Ord`], [`Clone`], [`Debug`]) are
//! written by hand against the inner `VersionIndex` so they hold for every `T`,
//! regardless of which traits `T` itself implements.
//!
//! #### Per-type storage layout
//!
//! [`AssetStorageResource<T>`] holds two structures: the `VersionCollection<T>` of
//! the assets themselves, and an
//! [`FfiHashMap`]`<FfiString, AssetHandle<T>>` mapping string
//! keys to handles. The key map is independent of the asset store — unregistering a
//! key does not remove the asset, and removing an asset does not unregister its key
//! (a later [`get_registered`](AssetStorageResource::get_registered) then yields a
//! handle that no longer resolves). Both `#[repr(C)]` types and `FfiHashMap` exist
//! because storages cross the engine's FFI boundary. The `Send`/`Sync` impls for
//! `AssetStorageResource<T>` are written manually and conditioned on `T`, since
//! `FfiHashMap` does not derive them automatically.
//!
//! #### The type-erased container
//!
//! [`AssetsStorageResource`] is a container that holds one
//! `AssetStorageResource<T>` per asset type, keyed by [`TypeId`] inside a
//! `Box<dyn Any + Send + Sync>`, with [`get`](AssetsStorageResource::get) /
//! [`get_mut`](AssetsStorageResource::get_mut) /
//! [`get_or_create`](AssetsStorageResource::get_or_create) downcasting back to the
//! concrete type. In the current engine, subsystems instead insert each
//! `AssetStorageResource<T>` directly as its own ECS resource (see `fruits_render`
//! and `fruits_audio`), so the container is the alternative aggregate entry point
//! rather than the path the engine wires up by default.

use std::{
    any::{Any, TypeId}, collections::HashMap, fmt::Debug, hash::{Hash, Hasher}, marker::PhantomData, sync::Arc
};

use fruits_ecs::Resource;
use fruits_ffi::{FfiHashMap, FfiString};
use fruits_utils::index_version_collection::{VersionCollection, VersionIndex};

// todo: ffi
#[derive(Resource)]
pub struct AssetsStorageResource {
    storages: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AssetsStorageResource {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    pub fn get<T: Send + Sync>(&self) -> Option<&AssetStorageResource<T>> {
        Some(self.storages.get(&TypeId::of::<T>())?.downcast_ref().unwrap())
    }

    pub fn get_mut<T: Send + Sync>(&mut self) -> Option<&mut AssetStorageResource<T>> {
        Some(self.storages.get_mut(&TypeId::of::<T>())?.downcast_mut().unwrap())
    }

    pub fn get_or_create<T: Send + Sync>(&mut self) -> &mut AssetStorageResource<T> {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(AssetStorageResource::<T>::new()))
            .downcast_mut()
            .unwrap()
    }
}

// todo: ffi
pub struct AssetStorageResource<T: 'static> {
    assets: VersionCollection<(T, Option<FfiString>)>,
    assets_by_key: FfiHashMap<FfiString, AssetHandle<T>>,
}

// todo: support generics in Resource impl macro
impl<T: 'static + Send + Sync> Resource for AssetStorageResource<T> {}

impl<T: 'static> AssetStorageResource<T> {
    pub fn new() -> Self {
        Self {
            assets: VersionCollection::new(),
            assets_by_key: FfiHashMap::new(),
        }
    }

    pub fn insert(&mut self, asset: T) -> AssetHandle<T> {
        AssetHandle::<T>::new(self.assets.insert((asset, None)))
    }

    pub fn remove(&mut self, handle: &AssetHandle<T>) -> Option<T> {
        self.assets.remove(handle.index()).map(|a| a.0)
    }

    pub fn get(&self, handle: &AssetHandle<T>) -> Option<&T> {
        self.assets.get(handle.index()).map(|a| &a.0)
    }

    pub fn get_mut(&mut self, handle: &AssetHandle<T>) -> Option<&mut T> {
        self.assets.get_mut(handle.index()).map(|a| &mut a.0)
    }

    pub fn register(&mut self, key: FfiString, handle: AssetHandle<T>) {
        // todo: remove panic
        self.assets.get_mut(handle.index()).unwrap().1 = Some(key.clone());
        if let Some(removed_handle) = self.assets_by_key.insert(key, handle) {
            self.assets.get_mut(removed_handle.index()).unwrap().1 = None;
        };
    }

    pub fn unregister(&mut self, key: &str) {
        // todo: remove panic
        if let Some(removed_handle) = self.assets_by_key.remove_by_str(key) {
            self.assets.get_mut(removed_handle.index()).unwrap().1 = None;
        };
    }

    pub fn get_registered(&self, key: &str) -> Option<&AssetHandle<T>> {
        self.assets_by_key.get_by_str(key)
    }

    pub fn get_registration(&self, handle: &AssetHandle<T>) -> Option<&str> {
        self.assets.get(handle.index()).map(|a| a.1.as_ref()).flatten().map(|a| a.as_str())
    }
}

unsafe impl<T: Send> Send for AssetStorageResource<T> {}
unsafe impl<T: Sync> Sync for AssetStorageResource<T> {}

//

#[repr(transparent)]
pub struct AssetHandle<T> {
    index: VersionIndex,
    _phantom: PhantomData<Arc<T>>,
}

impl<T> Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle").field("index", &self.index).finish()
    }
}

impl<T> Hash for AssetHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for AssetHandle<T> {}

impl<T> PartialOrd for AssetHandle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl<T> Ord for AssetHandle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            _phantom: Default::default(),
        }
    }
}

impl<T> AssetHandle<T> {
    pub fn new(index: VersionIndex) -> Self {
        Self {
            index,
            _phantom: Default::default(),
        }
    }

    pub fn index(&self) -> VersionIndex {
        self.index
    }
}
