use std::{
    any::Any, borrow::Cow, collections::HashMap, error::Error, fmt::{Debug, Display}
};

pub struct ExampleStruct<T> {
    pub data: Vec<T>,
    pub name: String,
    pub age: u32,
}

impl<T: 'static> TransSerializable for ExampleStruct<T> {
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        // serialize_tuple
        // serialize_struct
        // serialize_enum
        ctx.serialize_struct()
            .serialize_field("data", &self.data)
            .serialize_field("name", &self.name)
            .serialize_field("age", &self.age)
            .end()
    }

    fn deserialize(ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        let serde_json::Value::Object(map) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        Ok(Self {
            age: ctx.deserialize(map.get("age").ok_or_else(|| DeserializationError::InvalidInput)?)?,
            data: ctx.deserialize(map.get("data").ok_or_else(|| DeserializationError::InvalidInput)?)?,
            name: ctx.deserialize(map.get("name").ok_or_else(|| DeserializationError::InvalidInput)?)?,
        })
    }
}

pub struct ExampleTuple<T>(Vec<T>, String, u32);

impl<T: 'static> TransSerializable for ExampleTuple<T> {
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        // serialize_tuple
        // serialize_struct
        // serialize_enum
        ctx.serialize_tuple()
            .serialize_element(&self.data)
            .serialize_element(&self.name)
            .serialize_element(&self.age)
            .end()
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
    Struct { data: Vec<T>, name: String, age: u32 }
}

impl<T: 'static> TransSerializable for ExampleEnum<T> {
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        match self {
            ExampleEnum::Data(e0) => ctx.serialize_tuple().serialize_element(e0).end_enum("Data"),
            ExampleEnum::Name(e0) => ctx.serialize_tuple().serialize_element(e0).end_enum("Name"),
            ExampleEnum::Age(e0) => ctx.serialize_tuple().serialize_element(e0).end_enum("Age"),
            ExampleEnum::Struct { data, name, age } => ctx.serialize_struct()
                .serialize_field("data", data)
                .serialize_field("name", name)
                .serialize_field("age", age)
                .end_enum("Struct"),
        }
    }

    fn deserialize(ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        let serde_json::Value::Object(array) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        // todo
        todo!()
        // Ok(Self(
        //     ctx.deserialize(array.get(0).ok_or_else(|| DeserializationError::InvalidInput)?)?,
        //     ctx.deserialize(array.get(1).ok_or_else(|| DeserializationError::InvalidInput)?)?,
        //     ctx.deserialize(array.get(2).ok_or_else(|| DeserializationError::InvalidInput)?)?,
        // ))
    }
}

//

pub trait TransSerializable: Sized + 'static {
    fn serialize(&self, ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError>;
    fn deserialize(ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError>;
}

pub trait TransSerializer {
    type Deserialized: 'static;

    fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError>;
    fn deserialize(&self, ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError>;
}

pub struct AbstractSerializer<T: 'static> {
    serializer: Box<dyn TransSerializer<Deserialized = T>>,
}

impl<T: 'static> AbstractSerializer<T> {
    pub fn new(serializer: impl 'static + TransSerializer<Deserialized = T>) -> Self {
        Self {
            serializer: Box::new(serializer),
        }
    }
    pub fn serialize(&self, ctx: &SerializerContext, value: &T) -> Result<serde_json::Value, SerializationError> {
        self.serializer.serialize(ctx, value)
    }
    pub fn deserialize(&self, ctx: &DeserializerContext, value: &serde_json::Value) -> Result<T, DeserializationError> {
        self.serializer.deserialize(ctx, value)
    }
}

pub trait VirtualSerializer {
    fn serialize(&self, ctx: &SerializerContext, value: &dyn Any) -> Result<serde_json::Value, SerializationError>;
    fn deserialize(&self, ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Box<dyn Any>, DeserializationError>;
    fn self_as_any(&self) -> &dyn Any;
}

impl<T: 'static> VirtualSerializer for AbstractSerializer<T> {
    fn deserialize(&self, ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Box<dyn Any>, DeserializationError> {
        Ok(Box::new(self.deserialize(ctx, value)?))
    }

    fn serialize(&self, ctx: &SerializerContext, value: &dyn Any) -> Result<serde_json::Value, SerializationError> {
        let value = value
            .downcast_ref::<T>()
            .ok_or(SerializationError { type_name: "<missing>".into() })?;
        self.serialize(ctx, value)
    }

    fn self_as_any(&self) -> &dyn Any {
        self
    }
}

pub struct SerializerRegistry {
    serializers: HashMap<Cow<'static, str>, Box<dyn VirtualSerializer>>,
}

impl SerializerRegistry {
    pub fn new() -> Self {
        Self {
            serializers: HashMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, serializer: impl 'static + TransSerializer<Deserialized = T>) {
        let serializer = Box::new(AbstractSerializer::new(serializer));

        let type_name = serializer.name_override().unwrap_or(std::any::type_name::<T>().into());

        self.serializers.insert(type_name, serializer);
    }

    pub fn get<T: 'static>(&self) -> Option<(&AbstractSerializer<T>, &str)> {
        let type_name = std::any::type_name::<T>();
        let (type_name, serializer) = self.serializers.get_key_value(&type_name)?;
        let serializer = serializer.self_as_any();
        let serializer = serializer.downcast_ref::<AbstractSerializer<T>>().unwrap();

        Some((serializer, type_name))
    }

    pub fn get_virtual_by_name(&self, type_name: &str) -> Option<&dyn VirtualSerializer> {
        let serializer = &**self.serializers.get(type_name)?;

        Some(serializer)
    }
}

//

pub struct SerializationError {
    type_name: Cow<'static, str>,
}
impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SerializationError {{ {} }}", self.type_name)
    }
}
impl Debug for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <SerializationError as Display>::fmt(&self, f)
    }
}
impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub enum DeserializationError {
    NoSerializerRegistered { type_name: Cow<'static, str> },
    InvalidInput,
}
impl Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSerializerRegistered { type_name } => {
                write!(f, "NoSerializerRegistered {{ {} }}", type_name)
            }
            Self::InvalidInput => write!(f, "InvalidInput"),
        }
    }
}
impl Debug for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <DeserializationError as Display>::fmt(&self, f)
    }
}
impl Error for DeserializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub struct GlobalSerializer {
    serializers: SerializerRegistry,
}

impl GlobalSerializer {
    pub fn new() -> Self {
        Self {
            serializers: SerializerRegistry::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, serializer: impl 'static + TransSerializer<Deserialized = T>) {
        self.serializers.register(serializer)
    }

    pub fn serialize<T: 'static>(&self, value: &T) -> Result<serde_json::Value, SerializationError> {
        SerializerContext {
            registry: &self.serializers,
        }
        .serialize(value)
    }

    pub fn serialize_virtual(&self, value: &dyn Any) -> Result<serde_json::Value, SerializationError> {
        SerializerContext {
            registry: &self.serializers,
        }
        .serialize_virtual(value)
    }

    pub fn deserialize<T: 'static>(&self, data: &serde_json::Value) -> Result<T, DeserializationError> {
        DeserializerContext {
            registry: &self.serializers,
        }
        .deserialize(data)
    }

    pub fn deserialize_virtual(&self, data: &serde_json::Value) -> Result<Box<dyn Any>, DeserializationError> {
        DeserializerContext {
            registry: &self.serializers,
        }
        .deserialize_virtual(data)
    }
}

pub struct StructSerializerCtx<'r> {
    registry: &'r SerializerRegistry,
    fields: serde_json::Map<String, serde_json::Value>,
    error: Option<SerializationError>,
}

impl<'r> StructSerializerCtx<'r> {
    pub fn serialize_field<T>(mut self, name: impl Into<String>, value: &T) -> Self {
        if self.error.is_some() {
            return self;
        }

        let Some((serializer, type_name)) = self.registry.get::<T>() else {
            self.error = Some(SerializationError { type_name: std::any::type_name::<T>().into() });
            return self;
        };

        match serializer.serialize(&SerializerContext { registry: self.registry }, value) {
            Ok(serialized_value) => _ = self.fields.insert(name.into(), serialized_value),
            Err(_) => self.error = Some(SerializationError { type_name: std::any::type_name::<T>().into() }),
        }

        self
    }

    pub fn end_struct(self) -> Result<serde_json::Value, SerializationError> {
        if let Some(serialization_error) = self.error {
            return Err(serialization_error);
        }

        Ok(serde_json::Value::Object(self.fields))
    }

    pub fn end_enum(self, variant_name: impl Into<String>) -> Result<serde_json::Value, SerializationError> {
        if let Some(serialization_error) = self.error {
            return Err(serialization_error);
        }

        if self.fields.len() == 0 {
            return Ok(serde_json::Value::String(variant_name.into()));
        }

        let mut variant_as_json = serde_json::Map::<String, serde_json::Value>::new();

        variant_as_json.insert(variant_name.into(), serde_json::Value::Object(self.fields));

        Ok(serde_json::Value::Object(variant_as_json))
    }
}

pub struct TupleSerializerCtx<'r> {
    registry: &'r SerializerRegistry,
    elements: Vec<serde_json::Value>,
    error: Option<SerializationError>,
}

impl<'r> TupleSerializerCtx<'r> {
    pub fn serialize_element<T>(mut self, value: &T) -> Self {
        if self.error.is_some() {
            return self;
        }

        let Some((serializer, type_name)) = self.registry.get::<T>() else {
            self.error = Some(SerializationError { type_name: std::any::type_name::<T>().into() });
            return self;
        };

        match serializer.serialize(&SerializerContext { registry: self.registry }, value) {
            Ok(serialized_value) => _ = self.elements.push(serialized_value),
            Err(_) => self.error = Some(SerializationError { type_name: std::any::type_name::<T>().into() }),
        }

        self
    }

    pub fn end(self) -> Result<serde_json::Value, SerializationError> {
        if let Some(serialization_error) = self.error {
            return Err(serialization_error);
        }

        Ok(serde_json::Value::Array(self.elements))
    }

    pub fn end_enum(self, variant_name: impl Into<String>) -> Result<serde_json::Value, SerializationError> {
        if let Some(serialization_error) = self.error {
            return Err(serialization_error);
        }

        if self.fields.len() == 0 {
            return Ok(serde_json::Value::String(variant_name.into()));
        }

        let mut variant_as_json = serde_json::Map::<String, serde_json::Value>::new();

        variant_as_json.insert(variant_name.into(), serde_json::Value::Array(self.elements));

        Ok(serde_json::Value::Object(variant_as_json))
    }
}

pub struct SerializerContext<'r> {
    registry: &'r SerializerRegistry,
}
impl<'r> SerializerContext<'r> {
    pub fn serialize_struct(&self) -> StructSerializerCtx<'r> {
        StructSerializerCtx { registry: self.registry, fields: serde_json::Map::new(), error: None }
    }
    pub fn serialize_tuple(&self) -> TupleSerializerCtx<'r> {
        TupleSerializerCtx { registry: self.registry, elements: Vec::new(), error: None }
    }
    pub fn serialize_enum(&self) -> TupleSerializerCtx<'r> {
        TupleSerializerCtx { registry: self.registry, elements: Vec::new(), error: None }
    }
    //
    pub fn serialize<T: 'static>(&self, value: &T) -> Result<serde_json::Value, SerializationError> {
        let Some((serializer, type_name)) = self.registry.get() else {
            return Err(SerializationError {
                type_name: std::any::type_name::<T>().into(),
            });
        };

        let mut json_value = serializer.serialize(self, value)?;

        Self::try_annotate_type(&mut json_value, type_name)?;

        Ok(json_value)
    }

    pub fn serialize_virtual(&self, value: &dyn Any) -> Result<serde_json::Value, SerializationError> {
        let Some((serializer, type_name)) = self.registry.get_virtual_by_id(&Any::type_id(value)) else {
            return Err(SerializationError { type_name: "<missing>".into() });
        };

        let mut json_value = serializer.serialize(&self, value)?;

        Self::try_annotate_type(&mut json_value, type_name)?;

        Ok(json_value)
    }

    fn try_annotate_type(json_value: &mut serde_json::Value, type_name: Cow<'static, str>) -> Result<(), SerializationError> {
        if let serde_json::Value::Object(json_object) = json_value {
            if let Err(_) = json_object.push_field("$type", String::from(type_name)) {
                return Err(SerializationError { type_name });
            }
        }

        Ok(())
    }
}

pub struct DeserializerContext<'r> {
    registry: &'r SerializerRegistry,
}
impl<'r> DeserializerContext<'r> {
    pub fn deserialize<T: 'static>(&self, data: &serde_json::Value) -> Result<T, DeserializationError> {
        let Some((serializer, _)) = self.registry.get() else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: String::from(std::any::type_name::<T>()),
            });
        };
        serializer.deserialize(self, data)
    }

    pub fn deserialize_virtual(&self, data: &serde_json::Value) -> Result<Box<dyn Any>, DeserializationError> {
        let serde_json::Value::Object(json_object) = data else {
            return Err(DeserializationError::InvalidInput);
        };

        let Some(type_name) = json_object.get_value("$type") else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: String::from("<missing>"),
            });
        };

        let serde_json::Value::String(type_name) = type_name else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: type_name.to_string(),
            });
        };

        let Some(serializer) = self.registry.get_virtual_by_name(type_name) else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: String::from(type_name),
            });
        };

        serializer.deserialize(self, data)
    }
}
