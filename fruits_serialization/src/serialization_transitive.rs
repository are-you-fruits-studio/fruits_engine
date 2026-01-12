use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display},
};

use fruits_ffi::FfiAny;
use serde::{Deserialize, Serialize};

use crate::SerializerRegistry;

pub trait TransSerializable: Sized + 'static {
    fn serialize(&self, ctx: &SerializerCtx) -> Result<serde_json::Value, SerializationError>;
    fn deserialize(ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self, DeserializationError>;
}

pub struct SerializationError {
    pub type_name: Cow<'static, str>,
}
impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to serialize {} - probably serializer for the type is not registered", self.type_name)
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
                write!(f, "failed to deserialize {} - serializer for the type is not registered", type_name)
            }
            Self::InvalidInput => write!(f, "failed to deserialize - invalid deserialization input"),
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

pub struct StructSerializerCtx<'brw, 'local: 'brw> {
    ctx: SerializerCtx<'brw, 'local>,
    fields: serde_json::Map<String, serde_json::Value>,
    error: Option<SerializationError>,
}

impl<'brw, 'local: 'brw> StructSerializerCtx<'brw, 'local> {
    pub fn serialize_field<T: 'static>(mut self, name: impl Into<String>, value: &T) -> Self {
        if self.error.is_some() {
            return self;
        }

        match self.ctx.serialize(value) {
            Ok(serialized_value) => _ = self.fields.insert(name.into(), serialized_value),
            Err(err) => self.error = Some(err),
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

pub struct TupleSerializerCtx<'brw, 'local: 'brw> {
    ctx: SerializerCtx<'brw, 'local>,
    elements: Vec<serde_json::Value>,
    error: Option<SerializationError>,
}

impl<'brw, 'local: 'brw> TupleSerializerCtx<'brw, 'local> {
    pub fn serialize_element<T: 'static>(mut self, value: &T) -> Self {
        if self.error.is_some() {
            return self;
        }

        match self.ctx.serialize(value) {
            Ok(serialized_value) => self.elements.push(serialized_value),
            Err(err) => self.error = Some(err),
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

pub struct SerializerCtx<'brw, 'local: 'brw> {
    registry_global: &'brw SerializerRegistry<'static>,
    registry_local: Option<&'brw SerializerRegistry<'local>>,
}
impl<'brw, 'local: 'brw> SerializerCtx<'brw, 'local> {
    pub fn new(
        registry_global: &'brw SerializerRegistry<'static>,
        registry_local: Option<&'brw SerializerRegistry<'local>>,
    ) -> Self {
        Self {
            registry_global,
            registry_local,
        }
    }

    pub fn serialize_struct(self) -> StructSerializerCtx<'brw, 'local> {
        StructSerializerCtx {
            ctx: self,
            fields: serde_json::Map::new(),
            error: None,
        }
    }
    pub fn serialize_tuple(self) -> TupleSerializerCtx<'brw, 'local> {
        TupleSerializerCtx {
            ctx: self,
            elements: Vec::new(),
            error: None,
        }
    }
    pub fn serialize_enum_variant(&self, variant_name: impl Into<String>, value: serde_json::Value) -> serde_json::Value {
        serde_json::Value::Object([(variant_name.into(), value)].into_iter().collect())
    }
    //
    pub fn serialize<T: 'static>(&self, value: &T) -> Result<serde_json::Value, SerializationError> {
        if let Some(registry_local) = &self.registry_local {
            if let Some(serializer) = registry_local.get() {
                return serializer.serialize(self, value);
            }
        }

        if let Some(serializer) = self.registry_global.get() {
            return serializer.serialize(self, value);
        }

        Err(SerializationError {
            type_name: std::any::type_name::<T>().into(),
        })
    }
    pub fn deserialize<T: 'static>(&self, data: &serde_json::Value) -> Result<T, DeserializationError> {
        if let Some(registry_local) = &self.registry_local {
            if let Some(serializer) = registry_local.get() {
                return serializer.deserialize(self, data);
            }
        }

        if let Some(serializer) = self.registry_global.get() {
            return serializer.deserialize(self, data);
        }

        Err(DeserializationError::NoSerializerRegistered {
            type_name: std::any::type_name::<T>().into(),
        })
    }
    pub fn deserialize_any(&self, id: &str, data: &serde_json::Value) -> Result<FfiAny, DeserializationError> {
        if let Some(registry_local) = self.registry_local {
            if let Some(serializer) = registry_local.get_virtual(id) {
                return serializer.deserialize_any(self, data);
            }
        }

        if let Some(serializer) = self.registry_global.get_virtual(id) {
            return serializer.deserialize_any(self, data);
        }

        Err(DeserializationError::NoSerializerRegistered {
            type_name: id.to_string().into(),
        })
    }
}
// impl<'brw, 'local: 'brw> Clone for SerializerCtx<'brw, 'local> {
//     fn clone(&self) -> Self {
//         Self {
//             registry_global: self.registry_global,
//             registry_local: self.registry_local,
//         }
//     }
// }
// impl<'brw, 'local: 'brw> Copy for SerializerCtx<'brw, 'local> { }

impl<T: 'static + Serialize + for<'de> Deserialize<'de>> TransSerializable for T {
    fn serialize(&self, _ctx: &SerializerCtx) -> Result<serde_json::Value, SerializationError> {
        serde_json::value::to_value(self).map_err(|_| SerializationError {
            type_name: std::any::type_name::<T>().into(),
        })
    }

    fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self, DeserializationError> {
        serde_json::value::from_value(value.clone()).map_err(|_| DeserializationError::InvalidInput)
    }
}
