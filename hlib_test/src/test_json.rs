use std::{hint::black_box, time::Instant};

use dto::Person;
use fruits_serialization::{JsonObject, JsonValue, GlobalSerializer};
use serializers::{PersonSerializer, ProfileSerializer};

pub fn benchmark_serialization() {
    let mut serializer = GlobalSerializer::new();

    fruits_serialization::register_default_terminals(&mut serializer);
    fruits_serialization::register_self_and_related_default_terminals(&mut serializer, PersonSerializer);
    fruits_serialization::register_self_and_related_default_terminals(&mut serializer, ProfileSerializer);

    let count = 1000;

    let timer = Instant::now();

    for _ in 0..count {
        let json_value = JsonValue::parse(&mut TYPICAL_JSON.chars());
        black_box(json_value);
    }

    let parsing_time = timer.elapsed();

    println!("deserialization parsing: {}ns", parsing_time.as_nanos() / count);
    
    let json_value = JsonValue::parse(&mut TYPICAL_JSON.chars()).unwrap();
    
    let timer = Instant::now();

    for _ in 0..count {
        let sample_struct = serializer.deserialize::<Person>(&json_value).unwrap();
        black_box(sample_struct);
    }

    let converting_time = timer.elapsed();

    println!("deserialization converting: {}ns", converting_time.as_nanos() / count);


    

}

pub fn test_serialization() {
    let mut serializer = GlobalSerializer::new();

    fruits_serialization::register_default_terminals(&mut serializer);
    
    serializer.register(PersonSerializer);
    serializer.register(ProfileSerializer);

    //

    let sample_struct = serializer.deserialize::<Person>(&JsonValue::parse(&mut TYPICAL_JSON.chars()).unwrap()).unwrap();

    let serialized = serializer.serialize(&sample_struct);

    println!("G: {}", serialized.unwrap());

    //

    let sample_struct = serializer.deserialize_virtual(&JsonValue::parse(&mut TYPICAL_JSON.chars()).unwrap()).unwrap();

    let serialized = serializer.serialize_virtual(&*sample_struct).unwrap();

    println!("H: {:.2}", serialized);
}

pub fn test_json() {
    let mut json_object = JsonObject::new();
    {
        json_object.push_field("null_field", JsonValue::Null).ok().unwrap();
        let mut child = Vec::<JsonValue>::new();
        {
            child.push(false.into());
            child.push(123.into());
        }
        json_object.push_field("array_field", child).ok().unwrap();
        json_object.push_field("int_field", 5).ok().unwrap();
        json_object.push_field("float_field", 8.956).ok().unwrap();
        json_object.push_field("bool_field", false).ok().unwrap();
        json_object.push_field("string_field", String::from("hello, world")).ok().unwrap();
        let mut child = JsonObject::new();
        {
            child.push_field("name", String::from("Serhii")).ok().unwrap();
            child.push_field("age", 5).ok().unwrap();
        }
        json_object.push_field("object_field", child).ok().unwrap();
        json_object.push_field("empty_object_field", JsonObject::new()).ok().unwrap();
        json_object.push_field("empty_array_field", Vec::<JsonValue>::new()).ok().unwrap();
    }

    println!("{:.2}", JsonValue::Object(json_object));
}

//

mod dto { 
    pub struct Person {
        pub name: String,
        pub age: Option<u8>,
        pub is_developer: bool,
        pub friends: Vec<String>,
        pub profile: Profile,
    }

    pub struct Profile {
        pub email: String,
        pub password: String,
    }
}

mod serializers {
    use fruits_serialization::*;

    use super::dto::{Person, Profile};

    pub struct PersonSerializer;
    impl Serializer for PersonSerializer {
        type Deserialized = Person;

        fn name_override(&self) -> Option<&'static str> { Some("Dto.Person") }

        fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
            Ok(JsonValue::Object(JsonObject::new()
                .with_field("name", ctx.serialize(&value.name)?).ok().unwrap()
                .with_field("age", ctx.serialize(&value.age)?).ok().unwrap()
                .with_field("is_developer", ctx.serialize(&value.is_developer)?).ok().unwrap()
                .with_field("friends", ctx.serialize(&value.friends)?).ok().unwrap()
                .with_field("profile", ctx.serialize(&value.profile)?).ok().unwrap()
            ))
        }
        
        fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
            let JsonValue::Object(value) = value else {
                return Err(DeserializationError::InvalidInput);
            };
    
            Ok(Self::Deserialized {
                name: ctx.deserialize(value.get_value("name").ok_or(DeserializationError::InvalidInput)?)?,
                age: ctx.deserialize(value.get_value("age").ok_or(DeserializationError::InvalidInput)?)?,
                is_developer: ctx.deserialize(value.get_value("is_developer").ok_or(DeserializationError::InvalidInput)?)?,
                friends: ctx.deserialize(value.get_value("friends").ok_or(DeserializationError::InvalidInput)?)?,
                profile: ctx.deserialize(value.get_value("profile").ok_or(DeserializationError::InvalidInput)?)?,
            })
        }
    }

    pub struct ProfileSerializer;
    impl Serializer for ProfileSerializer {
        type Deserialized = Profile;

        fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
            Ok(JsonValue::Object(JsonObject::new()
                .with_field("email", ctx.serialize(&value.email)?).ok().unwrap()
                .with_field("password", ctx.serialize(&value.password)?).ok().unwrap()
            ))
        }
        
        fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
            let JsonValue::Object(value) = value else {
                return Err(DeserializationError::InvalidInput);
            };
    
            Ok(Self::Deserialized {
                email: ctx.deserialize(value.get_value("email").ok_or(DeserializationError::InvalidInput)?)?,
                password: ctx.deserialize(value.get_value("password").ok_or(DeserializationError::InvalidInput)?)?,
            })
        }
    }
}

static TYPICAL_JSON: &str = r#"
{
    "$type": "Dto.Person",
    "name": "Serhii",
    "age": 22,
    "is_developer": true,
    "friends": [
        "Hlib",
        "Illia",
        "Daniel"
    ],
    "profile": {
        "$type": "hlib_test::test_json::dto::Profile",
        "email": "serhii@gmail.com",
        "password": "12345678"
    }
}
"#;

