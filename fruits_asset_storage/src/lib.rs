use std::{
    any::{Any, TypeId}, collections::HashMap, fmt::Debug, hash::{Hash, Hasher}, marker::PhantomData, sync::Arc
};

use fruits_ecs::Resource;
use fruits_ffi::{FfiHashMap, FfiString};
use fruits_utils::index_version_collection::{VersionCollection, VersionIndex};

#[repr(C)]
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

#[repr(C)]
pub struct AssetStorageResource<T: 'static> {
    assets: VersionCollection<T>,
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
        AssetHandle::<T>::new(self.assets.insert(asset))
    }

    pub fn remove(&mut self, handle: &AssetHandle<T>) -> Option<T> {
        self.assets.remove(handle.index())
    }

    pub fn get(&self, handle: &AssetHandle<T>) -> Option<&T> {
        self.assets.get(handle.index())
    }

    pub fn get_mut(&mut self, handle: &AssetHandle<T>) -> Option<&mut T> {
        self.assets.get_mut(handle.index())
    }

    pub fn register(&mut self, key: FfiString, handle: AssetHandle<T>) {
        self.assets_by_key.insert(key, handle);
    }

    pub fn unregister(&mut self, key: &str) {
        self.assets_by_key.remove_by_str(key);
    }

    pub fn get_registered(&self, key: &str) -> Option<&AssetHandle<T>> {
        self.assets_by_key.get_by_str(key)
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
