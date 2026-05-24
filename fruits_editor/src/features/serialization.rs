use fruits_engine::*;

pub fn register_feature(mut world: WorldBuilderMut) {
    let mut serializer = GlobalSerializer::new();

    serializer.register(StandardTransSerializer::<i8>::default());
    serializer.register(StandardTransSerializer::<u8>::default());
    serializer.register(StandardTransSerializer::<i16>::default());
    serializer.register(StandardTransSerializer::<u16>::default());
    serializer.register(StandardTransSerializer::<i32>::default());
    serializer.register(StandardTransSerializer::<u32>::default());
    serializer.register(StandardTransSerializer::<i64>::default());
    serializer.register(StandardTransSerializer::<u64>::default());
    serializer.register(StandardTransSerializer::<i128>::default());
    serializer.register(StandardTransSerializer::<u128>::default());
    serializer.register(StandardTransSerializer::<f32>::default());
    serializer.register(StandardTransSerializer::<f64>::default());
    serializer.register(StandardTransSerializer::<String>::default());
    serializer.register(StandardTransSerializer::<char>::default());
    serializer.register(StandardTransSerializer::<bool>::default());
    serializer.register(StandardTransSerializer::<StandardMaterial>::default());
    serializer.register(StandardTransSerializer::<RenderSpace>::default());
    serializer.register(StandardTransSerializer::<Vec4<f32>>::default());
    serializer.register(StandardTransSerializer::<Option<f32>>::default());
    serializer.register(StandardTransSerializer::<Option<AssetHandle<StandardTexture>>>::default());

    world
        .data_mut()
        .resources_mut()
        .insert(SerializerResource(serializer))
        .ok()
        .unwrap();
}

// todo: to global unified resource in the engin (not in the editor)
#[derive(Resource)]
pub struct SerializerResource(pub GlobalSerializer);
