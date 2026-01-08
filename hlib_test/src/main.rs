use fruits_engine::{Entity, EntityTransSerializer, index_version_collection::VersionIndex};
use fruits_serialization::*;

fn main() {
    let mut global_serializer = GlobalSerializer::new();

    global_serializer.register(StandardTransSerializer::<UserInfo>::default());
    global_serializer.register(StandardTransSerializer::<String>::default());
    // global_serializer.register(StandardTransSerializer::<u32>::default());
    global_serializer.register(StandardTransSerializer::<SomeComponent>::default());

    //

    let component = SomeComponent {
        entity: Entity::from_version_index(VersionIndex { index: 1, version: 5 }),
        user_info: UserInfo {
            name: String::from("winner1337"),
            age: 10,
        },
    };

    //

    let entities_deserialized = [
        (25, Entity::from_version_index(VersionIndex { index: 1, version: 5 })),
    ].into_iter().collect();
    let entities_serialized = [
        (Entity::from_version_index(VersionIndex { index: 1, version: 5 }), 25),
    ].into_iter().collect();
    
    let mut local_serializer = SerializerRegistry::new();

    local_serializer.register(EntityTransSerializer::new(
        &entities_deserialized,
        &entities_serialized,
    ));

    let serialized_component = global_serializer.serialize(&component, Some(&local_serializer)).unwrap();

    println!("{}", serde_json::to_string_pretty(&serialized_component).unwrap());

    let deserialized_component = global_serializer.deserialize::<SomeComponent>(&serialized_component, Some(&local_serializer)).unwrap();

    dbg!(deserialized_component);
}

#[derive(TransSerializable, Debug)]
pub struct SomeComponent {
    pub entity: Entity,
    pub user_info: UserInfo,
}

//

#[derive(TransSerializable, Debug)]
pub struct UserInfo {
    pub name: String,
    pub age: u32,
}

#[derive(TransSerializable)]
pub struct SomeStruct<'a, 'b, T>
    where T : Copy
{
    name: &'b String,
    age: &'a u32,
    data: T,
    unit: SomeUnit,
}

#[derive(TransSerializable)]
pub struct SomeTuple(String, u32);

#[derive(TransSerializable)]
pub struct SomeUnit;

#[derive(TransSerializable)]
pub enum SomeEnum {
    A,
    B(u32),
    C { name: String },
    D(String, String, u32, Option<u32>),
    E { name1: String, password: String, age: Option<u8> }
}