use std::path::Path;

use fruits_asset_storage::AssetStorageResource;
use fruits_ecs::ResourcesHolderMut;
use fruits_render::StandardMaterial;
use fruits_serialization::{SerializedValue, SerializerCtx, TransSerializable};

use crate::AssetLoader;

pub struct MaterialLoader<'a> {
    pub materials: &'a mut AssetStorageResource<StandardMaterial>,
}

impl<'a> MaterialLoader<'a> {
    pub fn from_world(res: ResourcesHolderMut<'a>) -> Option<Self> {
        Some(Self {
            materials: res.into_get_mut::<AssetStorageResource<StandardMaterial>>()?,
        })
    }
}
impl<'a> AssetLoader for MaterialLoader<'a> {
    type Asset = StandardMaterial;
    type SelfWithAnotherLifetime<'r> = MaterialLoader<'r>;

    fn create_loader<'r>(res: ResourcesHolderMut<'r>) -> Option<Self::SelfWithAnotherLifetime<'r>> {
        Self::SelfWithAnotherLifetime::from_world(res)
    }
    
    fn get_related_asset_storage(&mut self) -> &mut AssetStorageResource<Self::Asset> {
        self.materials
    }
    
    fn load_from_serialized(&mut self, ctx: SerializerCtx, value: &SerializedValue, assets_dir_path: impl AsRef<Path>) -> Option<Self::Asset> {
        StandardMaterial::deserialize(ctx, value)
    }
}
