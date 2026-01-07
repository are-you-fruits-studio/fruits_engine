use crate::*;

pub struct ExampleStruct<T> {
    pub data: Vec<T>,
    pub name: String,
    pub age: u32,
}

impl<T> TransSerializable for ExampleStruct<T>
where
    Vec<T>: 'static,
    String: 'static,
    u32: 'static,
{
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        Ok(serde_json::Value::Object([
            (String::from("data"), ctx.serialize(&self.data)?),
            (String::from("name"), ctx.serialize(&self.name)?),
            (String::from("age"), ctx.serialize(&self.age)?),
        ].into_iter().collect()))
    }

    fn deserialize(ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        let serde_json::Value::Object(map) = value else { return Err(DeserializationError::InvalidInput); };

        Ok(Self {
            age: ctx.deserialize(map.get("age").ok_or_else(|| DeserializationError::InvalidInput)?)?,
            data: ctx.deserialize(map.get("data").ok_or_else(|| DeserializationError::InvalidInput)?)?,
            name: ctx.deserialize(map.get("name").ok_or_else(|| DeserializationError::InvalidInput)?)?,
        })
    }
}

pub struct ExampleTuple<T>(Vec<T>, String, u32);

impl<T> TransSerializable for ExampleTuple<T> 
where
    Vec<T>: 'static,
    String: 'static,
    u32: 'static,
{
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        Ok(serde_json::Value::Array(vec![
            ctx.serialize(&self.0)?,
            ctx.serialize(&self.1)?,
            ctx.serialize(&self.2)?,
        ]))
    }

    fn deserialize(ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        let serde_json::Value::Array(array) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        Ok(Self(
            ctx.deserialize(array.get(0).ok_or_else(|| DeserializationError::InvalidInput)?)?,
            ctx.deserialize(array.get(1).ok_or_else(|| DeserializationError::InvalidInput)?)?,
            ctx.deserialize(array.get(2).ok_or_else(|| DeserializationError::InvalidInput)?)?,
        ))
    }
}

pub enum ExampleEnum<T> {
    Data(Vec<T>),
    Name(String),
    Age(u32),
    Struct { data: Vec<T>, name: String, age: u32 },
    Empty,
}

impl<T> TransSerializable for ExampleEnum<T>
where
    Vec<T>: 'static,
    String: 'static,
    u32: 'static,
    Vec<T>: 'static,
    String: 'static,
    u32: 'static,
{
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        match self {
            Self::Data(e0) => ctx.serialize_tuple()
                .serialize_element(e0)
                .end_enum("Data"),
            Self::Name(e0) => ctx.serialize_tuple()
                .serialize_element(e0)
                .end_enum("Name"),
            Self::Age(e0) => ctx.serialize_tuple()
                .serialize_element(e0)
                .end_enum("Age"),
            Self::Struct { data, name, age } => ctx.serialize_struct()
                .serialize_field("data", data)
                .serialize_field("name", name)
                .serialize_field("age", age)
                .end_enum("Struct"),
            Self::Empty => ctx.serialize_struct()
                .end_enum("Empty"),
        }
    }

    fn deserialize(ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        let serde_json::Value::Object(object) = value else {
            return Err(DeserializationError::InvalidInput);
        };
        if object.len() != 1 {
            return Err(DeserializationError::InvalidInput);
        }
        let (variant, data) = object.iter().next().unwrap();
        match (variant.as_str(), data) {
            ("Data", serde_json::Value::Array(array)) => Ok(Self::Data(ctx.deserialize::<Vec<T>>(array.get(0).ok_or_else(|| DeserializationError::InvalidInput)?)?)),
            ("Name", serde_json::Value::Array(array)) => Ok(Self::Name(ctx.deserialize::<String>(array.get(0).ok_or_else(|| DeserializationError::InvalidInput)?)?)),
            ("Age", serde_json::Value::Array(array)) => Ok(Self::Age(ctx.deserialize::<u32>(array.get(0).ok_or_else(|| DeserializationError::InvalidInput)?)?)),
            ("Struct", serde_json::Value::Object(object)) => Ok(Self::Struct {
                data: ctx.deserialize::<Vec<T>>(object.get("data").ok_or_else(|| DeserializationError::InvalidInput)?)?,
                name: ctx.deserialize::<String>(object.get("name").ok_or_else(|| DeserializationError::InvalidInput)?)?,
                age: ctx.deserialize::<u32>(object.get("age").ok_or_else(|| DeserializationError::InvalidInput)?)?,
            }),
            ("Empty", _) => Ok(Self::Empty),
            _ => Err(DeserializationError::InvalidInput),
        }
    }
}
