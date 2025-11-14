use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
};

use crate::*;

pub trait Serializer {
    type Deserialized: 'static;

    fn name_override(&self) -> Option<&'static str> {
        None
    }
    fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError>;
    fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Self::Deserialized, DeserializationError>;
}

pub struct AbstractSerializer<T: 'static> {
    serializer: Box<dyn Serializer<Deserialized = T>>,
}

impl<T: 'static> AbstractSerializer<T> {
    pub fn new(serializer: impl 'static + Serializer<Deserialized = T>) -> Self {
        Self {
            serializer: Box::new(serializer),
        }
    }
    pub fn serialize(&self, ctx: &SerializerContext, value: &T) -> Result<JsonValue, SerializationError> {
        self.serializer.serialize(ctx, value)
    }
    pub fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<T, DeserializationError> {
        self.serializer.deserialize(ctx, value)
    }
    pub fn name_override(&self) -> Option<&'static str> {
        self.serializer.name_override()
    }
}

pub trait VirtualSerializer {
    fn serialize(&self, ctx: &SerializerContext, value: &dyn Any) -> Result<JsonValue, SerializationError>;
    fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Box<dyn Any>, DeserializationError>;
    fn self_as_any(&self) -> &dyn Any;
}

impl<T: 'static> VirtualSerializer for AbstractSerializer<T> {
    fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Box<dyn Any>, DeserializationError> {
        Ok(Box::new(self.deserialize(ctx, value)?))
    }

    fn serialize(&self, ctx: &SerializerContext, value: &dyn Any) -> Result<JsonValue, SerializationError> {
        let value = value
            .downcast_ref::<T>()
            .ok_or(SerializationError { type_name: "<missing>" })?;
        self.serialize(ctx, value)
    }

    fn self_as_any(&self) -> &dyn Any {
        self
    }
}

pub struct SerializerRegistry {
    serializers: Vec<(Box<dyn VirtualSerializer>, &'static str, TypeId)>,
    type_id_mapping: HashMap<TypeId, usize>,
    type_name_mapping: HashMap<String, usize>,
}

impl SerializerRegistry {
    pub fn new() -> Self {
        Self {
            serializers: Vec::new(),
            type_id_mapping: HashMap::new(),
            type_name_mapping: HashMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, serializer: impl 'static + Serializer<Deserialized = T>) {
        let serializer = Box::new(AbstractSerializer::new(serializer));

        let index = self.serializers.len();

        let type_id = TypeId::of::<T>();
        let type_name = serializer.name_override().unwrap_or(std::any::type_name::<T>());

        self.serializers.push((serializer, type_name, type_id));
        self.type_id_mapping.insert(type_id, index);
        self.type_name_mapping.insert(String::from(type_name), index);
    }

    pub fn get<T: 'static>(&self) -> Option<(&AbstractSerializer<T>, &'static str)> {
        let index = *self.type_id_mapping.get(&TypeId::of::<T>())?;
        let (serializer, type_name, _) = &self.serializers[index];
        let serializer = serializer.self_as_any();
        let serializer = serializer.downcast_ref::<AbstractSerializer<T>>().unwrap();

        Some((serializer, type_name))
    }

    pub fn get_virtual_by_name(&self, type_name: &str) -> Option<&dyn VirtualSerializer> {
        let index = *self.type_name_mapping.get(type_name)?;
        let serializer = &*self.serializers[index].0;

        Some(serializer)
    }

    pub fn get_virtual_by_id(&self, type_id: &TypeId) -> Option<(&dyn VirtualSerializer, &'static str)> {
        let index = *self.type_id_mapping.get(type_id)?;
        let meta = &self.serializers[index];

        Some((&*meta.0, meta.1))
    }
}

pub struct SerializationError {
    type_name: &'static str,
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
    NoSerializerRegistered { type_name: String },
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

    pub fn register<T: 'static>(&mut self, serializer: impl 'static + Serializer<Deserialized = T>) {
        self.serializers.register(serializer)
    }

    pub fn serialize<T: 'static>(&self, value: &T) -> Result<JsonValue, SerializationError> {
        SerializerContext {
            registry: &self.serializers,
        }
        .serialize(value)
    }

    pub fn serialize_virtual(&self, value: &dyn Any) -> Result<JsonValue, SerializationError> {
        SerializerContext {
            registry: &self.serializers,
        }
        .serialize_virtual(value)
    }

    pub fn deserialize<T: 'static>(&self, data: &JsonValue) -> Result<T, DeserializationError> {
        DeserializerContext {
            registry: &self.serializers,
        }
        .deserialize(data)
    }

    pub fn deserialize_virtual(&self, data: &JsonValue) -> Result<Box<dyn Any>, DeserializationError> {
        DeserializerContext {
            registry: &self.serializers,
        }
        .deserialize_virtual(data)
    }
}

pub struct SerializerContext<'r> {
    registry: &'r SerializerRegistry,
}
impl<'r> SerializerContext<'r> {
    pub fn serialize<T: 'static>(&self, value: &T) -> Result<JsonValue, SerializationError> {
        let Some((serializer, type_name)) = self.registry.get() else {
            return Err(SerializationError {
                type_name: std::any::type_name::<T>(),
            });
        };

        let mut json_value = serializer.serialize(self, value)?;

        Self::try_annotate_type(&mut json_value, type_name)?;

        Ok(json_value)
    }

    pub fn serialize_virtual(&self, value: &dyn Any) -> Result<JsonValue, SerializationError> {
        let Some((serializer, type_name)) = self.registry.get_virtual_by_id(&Any::type_id(value)) else {
            return Err(SerializationError { type_name: "<missing>" });
        };

        let mut json_value = serializer.serialize(&self, value)?;

        Self::try_annotate_type(&mut json_value, type_name)?;

        Ok(json_value)
    }

    fn try_annotate_type(json_value: &mut JsonValue, type_name: &'static str) -> Result<(), SerializationError> {
        if let JsonValue::Object(json_object) = json_value {
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
    pub fn deserialize<T: 'static>(&self, data: &JsonValue) -> Result<T, DeserializationError> {
        let Some((serializer, _)) = self.registry.get() else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: String::from(std::any::type_name::<T>()),
            });
        };
        serializer.deserialize(self, data)
    }

    pub fn deserialize_virtual(&self, data: &JsonValue) -> Result<Box<dyn Any>, DeserializationError> {
        let JsonValue::Object(json_object) = data else {
            return Err(DeserializationError::InvalidInput);
        };

        let Some(type_name) = json_object.get_value("$type") else {
            return Err(DeserializationError::NoSerializerRegistered {
                type_name: String::from("<missing>"),
            });
        };

        let JsonValue::String(type_name) = type_name else {
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
