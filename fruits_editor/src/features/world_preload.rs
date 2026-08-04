use std::sync::{Arc, Mutex};

use fruits_engine::*;
use libloading::Library;

use crate::{PROJECT_ASSETS_SUBPATH, SYSTEM_GROUP, app_building, features::inspector_window::data::OpenProjectResource};

pub fn register_feature(mut world: WorldBuilderMut) {
    world.data_mut().resources_mut().insert(SimulatedWorldResource(None));

    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(preload_assets_into_simulated_world_system);
}

pub struct SimulatedWorld {
    pub world: World,
    pub library: Option<Library>,
}
impl SimulatedWorld {
    pub fn new(world: World, library: Option<Library>) -> Self {
        Self {
            world,
            library,
        }
    }
}

#[derive(Resource)]
pub struct SimulatedWorldResource(pub Option<SimulatedWorld>);

fn preload_assets_into_simulated_world_system(
    open_project: Res<OpenProjectResource>,
    mut simulated_world: ResMut<SimulatedWorldResource>,
    render_api: Res<RenderApiResource>,
) {
    if simulated_world.0.is_some() {
        return;
    }

    let mut world = WorldBuilder::new();
    let init_result = unsafe {
        init_app_dynamically(world.as_mut(), app_building::path_scripts_lib_src(&open_project.dir_path))
    };
    let lib = match init_result {
        Ok(lib) => Some(lib),
        Err(err) => {
            println!("failed to load scripts lib: {err}");
            None
        },
    };
    let mut world = world.build();

    let mut res = world.data_mut().resources_mut();

    if res.as_ref().get::<SerializersResource>().is_none() {
        res.insert(SerializersResource(GlobalSerializer::new()));
    }

    let serializer = res.as_mut().get_mut::<SerializersResource>().unwrap();
    let serializer = &mut serializer.0;

    serializer.register(StandardTransSerializer::<SerializedValue>::default());
    serializer.register(StandardTransSerializer::<RenderSpace>::default());
    serializer.register(StandardTransSerializer::<f32>::default());
    serializer.register(StandardTransSerializer::<bool>::default());
    serializer.register(StandardTransSerializer::<FfiString>::default());
    serializer.register(StandardTransSerializer::<FfiSmallString>::default());
    serializer.register(StandardTransSerializer::<CoordinateSpaceType>::default());
    serializer.register(StandardTransSerializer::<Vec4<f32>>::default());
    serializer.register(StandardTransSerializer::<FfiOption<f32>>::default());
    serializer.register(StandardTransSerializer::<Option<f32>>::default());
    serializer.register(StandardTransSerializer::<StandardTextureAssetMetadata>::default());
    serializer.register(StandardTransSerializer::<StandardMaterial>::default());
    serializer.register(StandardTransSerializer::<StandardMeshAssetMetadata>::default());
    serializer.register(StandardTransSerializer::<AudioClipAssetMetadata>::default());
    // todo
    // serializer.register(StandardTransSerializer::<Prefab>::default());
    serializer.register(StandardTransSerializer::<FfiOption<StandardTexture>>::default());
    serializer.register(StandardTransSerializer::<FfiOption<AssetHandle<StandardTexture>>>::default());
    serializer.register(StandardTransSerializer::<DebugNameComponent>::default());
    serializer.register(StandardTransSerializer::<ChildComponent>::default());
    serializer.register(StandardTransSerializer::<ParentComponent>::default());
    serializer.register(StandardTransSerializer::<FfiVec<EntityId>>::default());

    res.insert(render_api.clone());
    res.insert(AudioStateResource::new(FfiDroppable::new(()), WrappedAudioStateHandle::new(Arc::new(Mutex::new(AudioState::new())))));
    res.insert(AssetStorageResource::<StandardTexture>::new());
    res.insert(AssetStorageResource::<StandardMaterial>::new());
    res.insert(AssetStorageResource::<Font>::new());
    res.insert(AssetStorageResource::<StandardMesh>::new());
    res.insert(AssetStorageResource::<AudioClip>::new());
    res.insert(AssetStorageResource::<Prefab>::new());

    load_all_assets(res.as_mut(), &(open_project.dir_path.to_string() + PROJECT_ASSETS_SUBPATH));

    simulated_world.0 = Some(SimulatedWorld::new(world, lib));
}