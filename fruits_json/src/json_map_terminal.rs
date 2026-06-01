use std::marker::PhantomData;

use crate::{json_map::*, json_repr::JsonValue};

// todo: overflow check
// todo: ffi for all?

#[derive(Copy, Clone, Default)]
pub struct StringSerializer;
impl Serializer for StringSerializer {
    type Deserialized = String;

    fn serialize(&self, _ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
        Ok(JsonValue::String(value.clone()))
    }

    fn deserialize(&self, _ctx: &DeserializerContext, value: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
        if let JsonValue::String(value) = value {
            Ok(value.clone())
        } else {
            Err(DeserializationError::InvalidInput)
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct StrSerializer;
impl Serializer for StrSerializer {
    type Deserialized = &'static str;

    fn serialize(&self, _ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
        Ok(JsonValue::String(String::from(*value)))
    }

    fn deserialize(&self, _ctx: &DeserializerContext, _value: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
        // todo: unsupported.
        Err(DeserializationError::InvalidInput)
    }
}

macro_rules! primitive_terminal_serializer {
    ($name: ident, $deserialized: ident, $json_type: ident, $convert_type: ident) => {
        #[derive(Copy, Clone, Default)]
        pub struct $name;
        impl Serializer for $name {
            type Deserialized = $deserialized;

            fn serialize(&self, _ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
                Ok(JsonValue::$json_type((*value).into()))
            }

            fn deserialize(
                &self,
                _ctx: &DeserializerContext,
                value: &JsonValue,
            ) -> Result<Self::Deserialized, DeserializationError> {
                if let JsonValue::$json_type(value) = value {
                    Ok((*value).into())
                } else {
                    Err(DeserializationError::InvalidInput)
                }
            }
        }
    };
}

primitive_terminal_serializer!(BoolSerializer, bool, Bool, bool);
primitive_terminal_serializer!(USizeSerializer, usize, Number, i128);
primitive_terminal_serializer!(ISizeSerializer, isize, Number, i128);
primitive_terminal_serializer!(U8Serializer, u8, Number, i128);
primitive_terminal_serializer!(I8Serializer, i8, Number, i128);
primitive_terminal_serializer!(U16Serializer, u16, Number, i128);
primitive_terminal_serializer!(I16Serializer, i16, Number, i128);
primitive_terminal_serializer!(U32Serializer, u32, Number, i128);
primitive_terminal_serializer!(I32Serializer, i32, Number, i128);
primitive_terminal_serializer!(U64Serializer, u64, Number, i128);
primitive_terminal_serializer!(I64Serializer, i64, Number, i128);
primitive_terminal_serializer!(I128Serializer, i128, Number, i128);
primitive_terminal_serializer!(F32Serializer, f32, Number, f64);
primitive_terminal_serializer!(F64Serializer, f64, Number, f64);

#[derive(Copy, Clone)]
pub struct VecSerializer<T>(PhantomData<T>);
impl<T> Default for VecSerializer<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T: 'static> Serializer for VecSerializer<T> {
    type Deserialized = Vec<T>;

    fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
        let mut results = Vec::new();

        for item in value {
            results.push(ctx.serialize(item)?);
        }

        Ok(JsonValue::Array(results))
    }

    fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
        let JsonValue::Array(json_array) = value else {
            return Err(DeserializationError::InvalidInput);
        };

        let mut results = Vec::new();

        for item in json_array {
            results.push(ctx.deserialize(item)?);
        }

        Ok(results)
    }
}

#[derive(Copy, Clone)]
pub struct BoxedSliceSerializer<T>(PhantomData<Option<T>>);
impl<T> Default for BoxedSliceSerializer<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T: 'static> Serializer for BoxedSliceSerializer<T> {
    type Deserialized = Box<[T]>;

    fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
        let mut results = Vec::new();

        for item in value {
            results.push(ctx.serialize(item)?);
        }

        Ok(JsonValue::Array(results))
    }

    fn deserialize(&self, ctx: &DeserializerContext, data: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
        let JsonValue::Array(json_array) = data else {
            return Err(DeserializationError::InvalidInput);
        };

        let mut results = Vec::new();

        for item in json_array {
            results.push(ctx.deserialize(item)?);
        }

        Ok(results.into_boxed_slice())
    }
}

#[derive(Copy, Clone)]
pub struct OptionSerializer<T>(PhantomData<T>);
impl<T> Default for OptionSerializer<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T: 'static> Serializer for OptionSerializer<T> {
    type Deserialized = Option<T>;

    fn serialize(&self, ctx: &SerializerContext, value: &Self::Deserialized) -> Result<JsonValue, SerializationError> {
        let Some(value) = value else {
            return Ok(JsonValue::Null);
        };
        ctx.serialize(value)
    }

    fn deserialize(&self, ctx: &DeserializerContext, value: &JsonValue) -> Result<Self::Deserialized, DeserializationError> {
        if let JsonValue::Null = value {
            return Ok(None);
        }
        Ok(Some(ctx.deserialize(value)?))
    }
}

pub fn register_default_terminals(global: &mut GlobalSerializer) {
    register_self_and_related_default_terminals(global, StringSerializer);
    register_self_and_related_default_terminals(global, StrSerializer);
    register_self_and_related_default_terminals(global, BoolSerializer);
    register_self_and_related_default_terminals(global, USizeSerializer);
    register_self_and_related_default_terminals(global, ISizeSerializer);
    register_self_and_related_default_terminals(global, U8Serializer);
    register_self_and_related_default_terminals(global, I8Serializer);
    register_self_and_related_default_terminals(global, U16Serializer);
    register_self_and_related_default_terminals(global, I16Serializer);
    register_self_and_related_default_terminals(global, U32Serializer);
    register_self_and_related_default_terminals(global, I32Serializer);
    register_self_and_related_default_terminals(global, U64Serializer);
    register_self_and_related_default_terminals(global, I64Serializer);
    register_self_and_related_default_terminals(global, I128Serializer);
    register_self_and_related_default_terminals(global, F32Serializer);
    register_self_and_related_default_terminals(global, F64Serializer);
}

pub fn register_self_and_related_default_terminals<S: 'static + Serializer>(global: &mut GlobalSerializer, serializer: S) {
    global.register(serializer);
    global.register(VecSerializer::<S::Deserialized>::default());
    global.register(BoxedSliceSerializer::<S::Deserialized>::default());
    global.register(OptionSerializer::<S::Deserialized>::default());
}
