//! # fruits_modules
//!
//! Bundles the engine's built-in subsystems into a single registration step, so an
//! app starts from a world that already has collision, transforms, rendering, and
//! prefab support wired together.
//!
//! # How to use
//!
//! #### Building a default world
//!
//! Register every default subsystem at once on a freshly created `App`
//! before populating the world. This is the call almost every app makes first:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let mut app = App::new();
//! add_defult_modules_to(app.ecs_mut().as_mut());
//! // ... register your own systems and entities ...
//! app.run();
//! ```
//!
//! #### Printing the frame rate
//!
//! The FPS counter is opt-in — it is *not* part of the default modules. Add it to
//! print the averaged frame rate and frame time to stdout once per second:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let mut app = App::new();
//! add_defult_modules_to(app.ecs_mut().as_mut());
//! fps_counter::add_module_to(app.ecs_mut().as_mut());
//! app.run();
//! ```
//!
//! # How to maintain
//!
//! #### Default module set and ordering
//!
//! [`add_defult_modules_to`] is the assembly point. It registers the collision,
//! transform, and render subsystems through their own `add_*_module_to` helpers,
//! inserts the resources the prefab pipeline depends on
//! ([`AssetStorageResource<Prefab>`](fruits_asset_storage::AssetStorageResource),
//! [`PrefabComponentsDeserializerResource`], and [`SerializersResource`]), then constrains the [`Update`](fruits_ecs::Schedule::Update)
//! schedule so the [`SYSTEM_GROUP_COLLISION`](fruits_collision::SYSTEM_GROUP_COLLISION)
//! group runs before [`SYSTEM_GROUP_TRANSFORM`](fruits_transform::SYSTEM_GROUP_TRANSFORM),
//! which runs before [`SYSTEM_GROUP_RENDER`](fruits_render::SYSTEM_GROUP_RENDER): colliders
//! settle, transforms propagate, the frame renders. The name carries an upstream
//! misspelling (`defult`); callers must match it exactly.
//!
//! #### Shared serializer registry
//!
//! [`SerializersResource`] is a newtype over [`GlobalSerializer`]
//! that derefs through to it, so a system holding the resource can call the serializer's
//! own methods directly. It is inserted here, with the world, rather than in
//! `fruits_serialization`, because it is the world-level home for the engine's component
//! serializers. The prefab loading path in `fruits_asset_loading` reads it and combines
//! its [`registry`](fruits_serialization::GlobalSerializer::registry) with per-load
//! transient serializers to deserialize prefab components.
//!
//! #### FPS counter
//!
//! `fps_counter` keeps an `FpsResource` holding a sliding window of up to 100 recent
//! frame durations. Its system records the time between successive runs, pushes each
//! duration into the window (dropping the oldest past 100), and once per second
//! averages the window to derive frame time and FPS, which it prints. The resource's
//! value accessors are currently commented out (`todo`), so the measured numbers are
//! only printed, not exposed to other systems.

use std::ops::{Deref, DerefMut};

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_ecs::{Resource, Schedule, WorldBuilderMut};
use fruits_ffi::{FfiOption, FfiString, FfiVec};
use fruits_math::{Mat, Quat, Vec4};
use fruits_prefab::{Prefab, PrefabComponentsDeserializerResource};
use fruits_render::{Font, RenderSpace, StandardMaterial};
use fruits_render_core::{StandardMesh, StandardTexture};
use fruits_serialization::{GlobalSerializer, StandardTransSerializer, TransSerializable};

pub mod fps_counter;

pub fn add_defult_modules_to(mut world: WorldBuilderMut) {
    fruits_collision::add_collision_module_to(world.as_mut());
    fruits_transform::add_transform_module_to(world.as_mut());
    fruits_render::add_render_module_to(world.as_mut());
    fruits_audio::add_audio_module_to(world.as_mut());

    world.data_mut().resources_mut().insert(AssetStorageResource::<Prefab>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(PrefabComponentsDeserializerResource::default()).ok().unwrap();
    world.data_mut().resources_mut().insert({
        let mut serializers = SerializersResource::default();
        register_common_transserializers(&mut serializers.0);
        serializers
    }).ok().unwrap();

    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .order_group(fruits_collision::SYSTEM_GROUP_COLLISION)
        .before_group(fruits_transform::SYSTEM_GROUP_TRANSFORM)
        .before_group(fruits_render::SYSTEM_GROUP_RENDER);
}

pub fn register_common_transserializers(serializer: &mut GlobalSerializer) {
    register_self_and_related_common_transserializers::<i8>(serializer);
    register_self_and_related_common_transserializers::<u8>(serializer);
    register_self_and_related_common_transserializers::<i16>(serializer);
    register_self_and_related_common_transserializers::<u16>(serializer);
    register_self_and_related_common_transserializers::<i32>(serializer);
    register_self_and_related_common_transserializers::<u32>(serializer);
    register_self_and_related_common_transserializers::<i64>(serializer);
    register_self_and_related_common_transserializers::<u64>(serializer);
    register_self_and_related_common_transserializers::<i128>(serializer);
    register_self_and_related_common_transserializers::<u128>(serializer);
    register_self_and_related_common_transserializers::<f32>(serializer);
    register_self_and_related_common_transserializers::<f64>(serializer);
    register_self_and_related_common_transserializers::<String>(serializer);
    register_self_and_related_common_transserializers::<FfiString>(serializer);
    register_self_and_related_common_transserializers::<char>(serializer);
    register_self_and_related_common_transserializers::<bool>(serializer);
    register_self_and_related_common_transserializers::<StandardMaterial>(serializer);
    register_self_and_related_common_transserializers::<RenderSpace>(serializer);
    // todo?
    register_related_common_transserializers::<AssetHandle<StandardMaterial>>(serializer);
    register_related_common_transserializers::<AssetHandle<Font>>(serializer);
    register_related_common_transserializers::<AssetHandle<StandardTexture>>(serializer);
    register_related_common_transserializers::<AssetHandle<StandardMesh>>(serializer);
    register_related_common_transserializers::<AssetHandle<Prefab>>(serializer);
}

pub fn register_self_and_related_common_transserializers<T: 'static + TransSerializable>(serializer: &mut GlobalSerializer) {
    serializer.register(StandardTransSerializer::<T>::default());
    register_related_common_transserializers::<T>(serializer)
}
pub fn register_related_common_transserializers<T: 'static>(serializer: &mut GlobalSerializer) {
    serializer.register(StandardTransSerializer::<Vec4<T>>::default());
    serializer.register(StandardTransSerializer::<Quat<T>>::default());
    serializer.register(StandardTransSerializer::<Mat<0, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<1, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<2, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<3, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<4, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<5, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<6, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<7, T>>::default());
    serializer.register(StandardTransSerializer::<Mat<8, T>>::default());
    serializer.register(StandardTransSerializer::<Vec<T>>::default());
    serializer.register(StandardTransSerializer::<FfiVec<T>>::default());
    serializer.register(StandardTransSerializer::<Option<T>>::default());
    serializer.register(StandardTransSerializer::<FfiOption<T>>::default());
}

// todo: ffi?
#[derive(Default)]
pub struct SerializersResource(GlobalSerializer);

impl Resource for SerializersResource { }

impl Deref for SerializersResource {
    type Target = GlobalSerializer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerializersResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}