use fruits_engine::{EntityId, FfiIndexMap, FfiString};
use fruits_serialization::*;

fn main() {
    check_index_map();
}

fn check_index_map() {
    let mut map = FfiIndexMap::new();

    map.insert(String::from("two"), 2);
    map.insert(String::from("one"), 1);
    map.insert(String::from("two"), 3);

    *map.get_mut("two").unwrap() = 22;

    map.insert(String::from("zero"), 0);

    map.remove_swap("two");
    map.remove_swap("zero");
    map.remove_swap("one");
    map.remove_swap("minus1");

    dbg!(map);
}

fn check_serialization() {
    let mut global_serializer = GlobalSerializer::new();

    global_serializer.register(StandardTransSerializer::<String>::default());
    global_serializer.register(StandardTransSerializer::<u32>::default());
    global_serializer.register(StandardTransSerializer::<bool>::default());
    global_serializer.register(StandardTransSerializer::<Vec<ExampleUser>>::default());
    global_serializer.register(StandardTransSerializer::<ExampleStruct>::default());
    global_serializer.register(StandardTransSerializer::<ExampleUser>::default());

    //

    let component = ExampleStruct {
        is_verified: true,
        user: ExampleUser::Default {
            name: String::from("Sebastian"),
            age: 44,
        },
        friends: vec![
            ExampleUser::Default {
                name: String::from("Daniel"),
                age: 25,
            },
            ExampleUser::Token(String::from("token_123_abc")),
        ],
    };

    //

    let serialized_component = global_serializer.serialize(&component, None).unwrap();

    dbg!(&serialized_component);
    println!("{}", serde_json::to_string_pretty(&serialized_component.to_json()).unwrap());

    let deserialized_component = global_serializer.deserialize::<ExampleStruct>(&serialized_component, None).unwrap();

    dbg!(&deserialized_component);
    dbg!(deserialized_component == Some(component));
}

// fn check_macros() {
//     let mut global_serializer = GlobalSerializer::new();

//     global_serializer.register(StandardTransSerializer::<UserInfo>::default());
//     global_serializer.register(StandardTransSerializer::<String>::default());
//     // global_serializer.register(StandardTransSerializer::<u32>::default());
//     global_serializer.register(StandardTransSerializer::<SomeComponent>::default());

//     //

//     let component = SomeComponent {
//         entity: Entity::from_version_index(VersionIndex { index: 1, version: 5 }),
//         user_info: UserInfo {
//             name: String::from("winner1337"),
//             age: 10,
//         },
//     };

//     //

//     let entities_deserialized = [
//         (25, Entity::from_version_index(VersionIndex { index: 1, version: 5 })),
//     ].into_iter().collect();
//     let entities_serialized = [
//         (Entity::from_version_index(VersionIndex { index: 1, version: 5 }), 25),
//     ].into_iter().collect();
    
//     let mut local_serializer = SerializerRegistry::new();

//     local_serializer.register(EntityTransSerializer::new(
//         &entities_deserialized,
//         &entities_serialized,
//     ));

//     let serialized_component = global_serializer.serialize(&component, Some(&local_serializer)).unwrap();

//     println!("{}", serde_json::to_string_pretty(&serialized_component).unwrap());

//     let deserialized_component = global_serializer.deserialize::<SomeComponent>(&serialized_component, Some(&local_serializer)).unwrap();

//     dbg!(deserialized_component);
// }

#[derive(Debug, PartialEq)]
pub struct ExampleStruct {
    user: ExampleUser,
    friends: Vec<ExampleUser>,
    is_verified: bool,
}

#[derive(Debug, PartialEq)]
pub enum ExampleUser {
    Default { name: String, age: u32 },
    Token(String),
}

//

impl TransSerializable for ExampleStruct {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        ctx.serialize_map()
            .with_field("user", &self.user)
            .with_field("friends", &self.friends)
            .with_field("is_verified", &self.is_verified)
            .finish_as_map(true)
    }

    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        ctx.deserialize_map(value, |ctx| { Some(Self {
            user: ctx.get_field("user")?,
            friends: ctx.get_field("friends")?,
            is_verified: ctx.get_field("is_verified")?,
        })})
    }
}

impl TransSerializable for ExampleUser {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        let variants = ["Default", "Token"].into_iter().map(FfiString::from).collect();

        match self {
            Self::Default { name, age } => ctx.serialize_map()
                .with_field("name", name)
                .with_field("age", age)
                .finish_as_enum(true, "Default", variants),
            Self::Token(arg0) => ctx.serialize_map()
                .with_field("0", arg0)
                .finish_as_enum(true, "Token", variants),
        }
    }

    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        ctx.deserialize_enum()
            .variant("Default", |ctx, value| {
                ctx.deserialize_map(value, |ctx| {
                    Some(Self::Default {
                        name: ctx.get_field("name")?,
                        age: ctx.get_field("age")?,
                    })
                })
            })
            .variant("Token", |ctx, value| {
                ctx.deserialize_map(value, |ctx| {
                    Some(Self::Token(
                        ctx.get_field("0")?,
                    ))
                })
            })
            .finish(value)
    }
}


#[derive(TransSerializable, Debug)]
pub struct SomeComponent {
    pub entity: EntityId,
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
