use std::{
    any::Any, borrow::Cow, collections::HashMap, error::Error, fmt::{Debug, Display}
};

use serde::{Deserialize, Serialize};

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

        let type_name = std::any::type_name::<T>().into();

        self.serializers.insert(type_name, serializer);
    }

    pub fn get<T: 'static>(&self) -> Option<(&AbstractSerializer<T>, &str)> {
        let type_name = std::any::type_name::<T>();
        let (type_name, serializer) = self.serializers.get_key_value(type_name)?;
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
    pub type_name: Cow<'static, str>,
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

    pub fn deserialize<T: 'static>(&self, data: &serde_json::Value) -> Result<T, DeserializationError> {
        DeserializerContext {
            registry: &self.serializers,
        }
        .deserialize(data)
    }
}

pub struct StructSerializerCtx<'r> {
    registry: &'r SerializerRegistry,
    fields: serde_json::Map<String, serde_json::Value>,
    error: Option<SerializationError>,
}

impl<'r> StructSerializerCtx<'r> {
    pub fn serialize_field<T: 'static>(mut self, name: impl Into<String>, value: &T) -> Self {
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

    pub fn end(self) -> Result<serde_json::Value, SerializationError> {
        if let Some(serialization_error) = self.error {
            return Err(serialization_error);
        }

        Ok(serde_json::Value::Object(self.fields))
    }

    pub fn end_enum(self, variant_name: impl Into<String>) -> Result<serde_json::Value, SerializationError> {
        if let Some(serialization_error) = self.error {
            return Err(serialization_error);
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
    pub fn serialize_element<T: 'static>(mut self, value: &T) -> Self {
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

        Ok(serializer.serialize(self, value)?)
    }
}

pub struct DeserializerContext<'r> {
    registry: &'r SerializerRegistry,
}
impl<'r> DeserializerContext<'r> {
    pub fn deserialize<T: 'static>(&self, data: &serde_json::Value) -> Result<T, DeserializationError> {
        let Some((serializer, _)) = self.registry.get() else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: std::any::type_name::<T>().into(),
            });
        };
        serializer.deserialize(self, data)
    }
}

impl<T: 'static + Serialize + for<'de> Deserialize<'de>> TransSerializable for T {
    fn serialize(&self, _ctx: &SerializerContext) -> Result<serde_json::Value, SerializationError> {
        serde_json::value::to_value(self).map_err(|_| SerializationError { type_name: std::any::type_name::<T>().into() })
    }

    fn deserialize(_ctx: &DeserializerContext, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        serde_json::value::from_value(value.clone()).map_err(|_| DeserializationError::InvalidInput)
    }
}