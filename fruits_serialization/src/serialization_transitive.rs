use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display, Write},
};

use fruits_ffi::{FfiAny, FfiIndexMap, FfiOption, FfiString, FfiVec};

use crate::{SerializedComposite, SerializedCompositeValues, SerializedEnumMetadata, SerializedMap, SerializedValue, SerializerRegistry};

// todo: ffi

pub struct SerializationResult<T> {
    pub result: T,
    pub err: FfiVec<SerializationError>,
}

impl<T> SerializationResult<T> {
    pub fn unwrap(self) -> T {
        if let Some(err) = self.format_err() {
            panic!("{}", err);
        }

        self.result
    }

    pub fn format_err(&self) -> Option<String> {
        if self.err.is_empty() {
            return None;
        }

        let mut message = String::new();

        for err in &self.err {
            writeln!(&mut message, "{}", err).unwrap();
        }

        Some(message)
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> SerializationResult<U> {
        SerializationResult {
            result: f(self.result),
            err: self.err,
        }
    }

    pub fn unwrap_and_store(self, error: &mut FfiVec<SerializationError>) -> T {
        error.extend_from_slice(&self.err);
        self.result
    }
}

impl<T> SerializationResult<Option<T>> {
    pub fn into_result(self) -> Result<SerializationResult<T>, FfiVec<SerializationError>> {
        match self.result {
            Some(result) => Ok(SerializationResult {
                result,
                err: self.err,
            }),
            None => Err(self.err),
        }
    }

    pub fn from_result(result: Result<SerializationResult<T>, FfiVec<SerializationError>>) -> Self {
        match result {
            Ok(result) => SerializationResult {
                result: Some(result.result),
                err: result.err,
            },
            Err(error) => SerializationResult {
                result: None,
                err: error,
            },
        }
    }
}

impl<T: Debug> Debug for SerializationResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializationResult")
            .field("result", &self.result)
            .field("err", &self.err)
            .finish()
    }
}

pub trait TransSerializable: Sized + 'static {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue>;
    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>>;
}

// todo: ffi
#[derive(Clone)]
pub enum SerializationError {
    MissingSerializer { type_name: Cow<'static, str> },
    InvalidInput { message: String },
}
impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSerializer { type_name } => {
                write!(f, "failed to deserialize {} - serializer for the type is not registered", type_name)
            }
            Self::InvalidInput { message } => write!(f, "failed to deserialize - invalid deserialization input: {}", message),
        }
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

    fn clone(&self) -> Self {
        Self {
            registry_global: self.registry_global,
            registry_local: self.registry_local,
        }
    }

    pub fn deserialize_map<T: 'static>(
        &self,
        value: &SerializedValue,
        f: impl FnOnce(&mut MapDeserializerCtx) -> Option<T>,
    ) -> SerializationResult<Option<T>> {
        let mut ctx = MapDeserializerCtx::new(
            self.clone(),
            value,
        );

        SerializationResult {
            result: f(&mut ctx),
            err: ctx.error,
        }
    }
    pub fn deserialize_list<T: 'static>(
        &self,
        value: &SerializedValue,
        f: impl FnOnce(&mut ListDeserializerCtx) -> Option<T>,
    ) -> SerializationResult<Option<T>> {
        let mut ctx = ListDeserializerCtx::new(
            self.clone(),
            value,
        );

        SerializationResult {
            result: f(&mut ctx),
            err: ctx.error,
        }
    }
    pub fn deserialize_enum<T>(&self) -> EnumDeserializerCtx<'brw, 'local, T> {
        EnumDeserializerCtx::new(self.clone())
    }

    pub fn serialize_map(&self) -> MapSerializerCtx<'brw, 'local> {
        MapSerializerCtx {
            ctx: self.clone(),
            fields: FfiIndexMap::new(),
            error: FfiVec::new(),
        }
    }
    pub fn serialize_list(&self) -> ListSerializerCtx<'brw, 'local> {
        ListSerializerCtx {
            ctx: self.clone(),
            elements: FfiVec::new(),
            error: FfiVec::new(),
        }
    }

    pub fn get_field<'a, T: 'static>(&self, value: &'a FfiIndexMap<String, SerializedValue>, name: &str) -> SerializationResult<Option<T>> {
        let serialized_value = value.get(name).unwrap_or_else(|| &SerializedValue::Null);

        self.deserialize(serialized_value)
    }
    pub fn get_element<'a, T: 'static>(&self, value: &'a FfiVec<SerializedValue>, idx: usize) -> SerializationResult<Option<T>> {
        let serialized_value = value.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null);

        self.deserialize(serialized_value)
    }

    pub fn serialize<T: 'static>(&self, value: &T) -> SerializationResult<SerializedValue> {
        if let Some(registry_local) = &self.registry_local {
            if let Some(serializer) = registry_local.get() {
                return serializer.serialize(self, value);
            }
        }

        if let Some(serializer) = self.registry_global.get() {
            return serializer.serialize(self, value);
        }

        SerializationResult {
            result: SerializedValue::Null,
            err: vec![SerializationError::MissingSerializer {
                type_name: std::any::type_name::<T>().into(),
            }].into(),
        }
    }
    pub fn deserialize<T: 'static>(&self, data: &SerializedValue) -> SerializationResult<Option<T>> {
        if let Some(registry_local) = &self.registry_local {
            if let Some(serializer) = registry_local.get() {
                return serializer.deserialize(self, data);
            }
        }

        if let Some(serializer) = self.registry_global.get() {
            return serializer.deserialize(self, data);
        }

        SerializationResult {
            result: None,
            err: vec![SerializationError::MissingSerializer {
                type_name: std::any::type_name::<T>().into(),
            }].into(),
        }
    }
    pub fn deserialize_any(&self, id: &str, data: &SerializedValue) -> SerializationResult<Option<FfiAny>> {
        if let Some(registry_local) = self.registry_local {
            if let Some(serializer) = registry_local.get_virtual(id) {
                return serializer.deserialize_any(self, data);
            }
        }

        if let Some(serializer) = self.registry_global.get_virtual(id) {
            return serializer.deserialize_any(self, data);
        }

        SerializationResult {
            result: None,
            err: vec![SerializationError::MissingSerializer {
                type_name: id.to_string().into(),
            }].into(),
        }
    }
}

pub struct MapSerializerCtx<'brw, 'local: 'brw> {
    ctx: SerializerCtx<'brw, 'local>,
    fields: FfiIndexMap<FfiString, SerializedValue>,
    error: FfiVec<SerializationError>,
}

impl<'brw, 'local: 'brw> MapSerializerCtx<'brw, 'local> {
    pub fn with_field<T: 'static>(mut self, name: impl Into<FfiString>, value: &T) -> Self {
        let result = self.ctx.serialize(value);

        self.error.extend_from_slice(&result.err);
        self.fields.insert(name.into(), result.result);

        self
    }

    pub fn finish_as_map(self, is_rigid: bool) -> SerializationResult<SerializedValue> {
        SerializationResult {
            err: self.error,
            result: SerializedValue::Composite(SerializedComposite {
                values: SerializedCompositeValues::Map(SerializedMap {
                    values: self.fields,
                    enum_metadata: None.into(),
                }),
                is_rigid,
            })
        }
    }

    pub fn finish_as_enum(self, is_rigid: bool, variant: impl Into<FfiString>, variants: FfiVec<FfiString>) -> SerializationResult<SerializedValue> {
        SerializationResult {
            err: self.error,
            result: SerializedValue::Composite(SerializedComposite {
                values: SerializedCompositeValues::Map(SerializedMap {
                    values: self.fields,
                    enum_metadata: SerializedEnumMetadata {
                        variant: variant.into(),
                        variants,
                    }.into(),
                }),
                is_rigid,
            })
        }
    }
}

// todo: ffi
pub struct ListSerializerCtx<'brw, 'local: 'brw> {
    ctx: SerializerCtx<'brw, 'local>,
    elements: FfiVec<SerializedValue>,
    error: FfiVec<SerializationError>,
}

impl<'brw, 'local: 'brw> ListSerializerCtx<'brw, 'local> {
    pub fn with_element<T: 'static>(mut self, value: &T) -> Self {
        let result = self.ctx.serialize(value);

        self.error.extend_from_slice(&result.err);
        self.elements.push(result.result);

        self
    }

    pub fn finish_as_list(self, is_rigid: bool) -> SerializationResult<SerializedValue> {
        SerializationResult {
            err: self.error,
            result: SerializedValue::Composite(SerializedComposite {
                values: SerializedCompositeValues::List(self.elements),
                is_rigid,
            })
        }
    }
}

//



// todo: ffi
pub struct MapDeserializerCtx<'brw, 'local: 'brw> {
    ctx: SerializerCtx<'brw, 'local>,
    fields: FfiIndexMap<FfiString, SerializedValue>,
    error: FfiVec<SerializationError>,
}

impl<'brw, 'local: 'brw> MapDeserializerCtx<'brw, 'local> {
    pub fn new(ctx: SerializerCtx<'brw, 'local>, value: &SerializedValue) -> Self {
        let fields = match value {
            SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::Map(value), .. }) => value.values.clone(),
            _ => FfiIndexMap::new()
        };

        Self {
            ctx,
            fields,
            error: FfiVec::new(),
        }
    }

    pub fn get_field<'a, T: 'static>(&mut self, name: &str) -> Option<T> {
        let serialized_value = self.fields.get(name).unwrap_or_else(|| &SerializedValue::Null);

        let result = self.ctx.deserialize(serialized_value);
        self.error.extend_from_slice(&result.err);
        result.result
    }
}

// todo: ffi
pub struct ListDeserializerCtx<'brw, 'local: 'brw> {
    ctx: SerializerCtx<'brw, 'local>,
    elements: FfiVec<SerializedValue>,
    error: FfiVec<SerializationError>,
}

impl<'brw, 'local: 'brw> ListDeserializerCtx<'brw, 'local> {
    pub fn new(ctx: SerializerCtx<'brw, 'local>, value: &SerializedValue) -> Self {
        let elements = match value {
            SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::List(value), .. }) => value.clone(),
            _ => FfiVec::new()
        };

        Self {
            ctx,
            elements,
            error: FfiVec::new(),
        }
    }

    pub fn get_element<'a, T: 'static>(&mut self, idx: usize) -> Option<T> {
        let serialized_value = self.elements.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null);

        let result = self.ctx.deserialize(serialized_value);
        self.error.extend_from_slice(&result.err);
        result.result
    }
}

// todo: ffi
pub struct EnumDeserializerCtx<'brw, 'local: 'brw, T> {
    ctx: SerializerCtx<'brw, 'local>,
    variants: FfiVec<(FfiString, Box<dyn FnMut(SerializerCtx<'brw, 'local>, &SerializedValue) -> SerializationResult<Option<T>>>)>,
}

impl<'brw, 'local: 'brw, T> EnumDeserializerCtx<'brw, 'local, T> {
    pub fn new(ctx: SerializerCtx<'brw, 'local>) -> Self {
        Self {
            ctx,
            variants: FfiVec::new(),
        }
    }
    pub fn variant(mut self, variant: impl Into<FfiString>, deserializer: impl 'static + FnMut(SerializerCtx<'brw, 'local>, &SerializedValue) -> SerializationResult<Option<T>>) -> Self {
        self.variants.push((variant.into(), Box::new(deserializer)));
        self
    }

    pub fn finish(mut self, value: &SerializedValue) -> SerializationResult<Option<T>> {
        let mut err = FfiVec::new();

        if let SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::Map(SerializedMap { enum_metadata: FfiOption::Some(enum_metadata), .. }), .. }) = value {
            for (variant, deserializer) in &mut self.variants {
                if variant == &enum_metadata.variant {
                    let result = deserializer(self.ctx.clone(), value);
                    
                    err.extend_from_slice(&result.err);
                    if let Some(value) = result.result {
                        return SerializationResult {
                            result: Some(value),
                            err,
                        };
                    }
                    break;
                }
            }
        }

        err.push(SerializationError::InvalidInput { message: String::from("Deserializizing enum without a valid variant") });

        for (_variant, deserializer) in &mut self.variants {
            let result = deserializer(self.ctx.clone(), value);
            err.extend_from_slice(&result.err);

            if let Some(value) = result.result {
                return SerializationResult {
                    result: Some(value),
                    err,
                };
            }
        }

        SerializationResult {
            result: None,
            err,
        }
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

//

// todo
// impl<T: 'static + Serialize + for<'de> Deserialize<'de>> TransSerializable for T {
//     fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
//         serde_json::value::to_value(self).map_err(|_| SerializationError::NoSerializerRegistered {
//             type_name: std::any::type_name::<T>().into(),
//         })
//     }

//     fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> Result<SerializationResult<Self>, SerializationError> {
//         serde_json::value::from_value(value.clone()).map_err(|_| SerializationError::InvalidInput)
//     }
// }
