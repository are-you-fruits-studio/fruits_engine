use std::{
    error::Error,
    fmt::{Debug, Display},
};

use fruits_ffi::{FfiAny, FfiAnyRef, FfiFnMutMut, FfiIndexMap, FfiOption, FfiString, FfiVec};

use crate::{PureSerializerCtx, SerializedComposite, SerializedCompositeValues, SerializedEnumMetadata, SerializedMap, SerializedValue, SerializerRegistry};

// todo: ffi

pub trait TransSerializable: Sized + 'static {
    fn serialize(&self, ctx: SerializerCtx) -> SerializedValue;
    fn deserialize(ctx: SerializerCtx, value: &SerializedValue) -> Option<Self>;
}

#[repr(C)]
#[derive(Clone)]
pub enum SerializationError {
    MissingSerializer { type_name: FfiString },
    InvalidInput { message: FfiString },
}
impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSerializer { type_name } => {
                write!(f, "missing serializer for type {}", type_name)
            }
            Self::InvalidInput { message } => write!(f, "invalid serialization input: {}", message),
        }
    }
}
impl Debug for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <SerializationError as Display>::fmt(&self, f)
    }
}
impl Error for SerializationError {
}

// todo: ctx wraps wider ctx for scoping
// mod sealed {
//     pub trait Sealed {}
// }
// pub trait SerializerCtxTrait : sealed::Sealed {
//     type WiderCtx;
// }
// impl<'a, 'l: 'a> sealed::Sealed for SerializerCtx<'a, 'l> {
// }

#[repr(C)]
pub struct SerializerCtx<'a, 'l: 'a> {
    registry_global: &'a SerializerRegistry<'static>,
    registry_local: FfiOption<&'a SerializerRegistry<'l>>,
    err_handler: FfiFnMutMut<'a, SerializationError, ()>,
}
impl<'a, 'l: 'a> SerializerCtx<'a, 'l> {
    pub fn new(
        registry_global: &'a SerializerRegistry<'static>,
        registry_local: Option<&'a SerializerRegistry<'l>>,
        err_handler: FfiFnMutMut<'a, SerializationError, ()>,
    ) -> Self {
        Self {
            registry_global,
            registry_local: registry_local.into(),
            err_handler,
        }
    }

    pub fn as_mut<'r>(&'r mut self) -> SerializerCtx<'r, 'l>
        where 'a: 'r
    {
        SerializerCtx {
            registry_global: self.registry_global,
            registry_local: self.registry_local,
            err_handler: self.err_handler.as_mut(),
        }
    }

    pub fn as_pure<'r>(&'r mut self) -> PureSerializerCtx<'r>
        where 'a: 'r
    {
        PureSerializerCtx::new(self.err_handler.as_mut())
    }
    // todo
    // fn clone(&self) -> Self {
    //     Self {
    //         registry_global: self.registry_global,
    //         registry_local: self.registry_local,
    //     }
    // }

    pub fn report_err(&mut self, err: SerializationError) {
        self.err_handler.execute(err);
    }

    pub fn deserialize_map<T: 'static>(
        &mut self,
        value: &SerializedValue,
        f: impl FnOnce(MapDeserializerCtx) -> Option<T>,
    ) -> Option<T> {
        f(MapDeserializerCtx::new(
            self.as_mut(),
            value,
        ))
    }
    pub fn deserialize_list<T: 'static>(
        &mut self,
        value: &SerializedValue,
        f: impl FnOnce(ListDeserializerCtx) -> Option<T>,
    ) -> Option<T> {
        f(ListDeserializerCtx::new(
            self.as_mut(),
            value,
        ))
    }
    pub fn deserialize_enum<'r, T>(&'r mut self) -> EnumDeserializerCtx<'r, 'l, T>
        where 'a: 'r
    {
        EnumDeserializerCtx::<T>::new(self.as_mut())
    }

    pub fn serialize_map<'r>(&'r mut self) -> MapSerializerCtx<'r, 'l>
        where 'a: 'r
    {
        MapSerializerCtx {
            ctx: self.as_mut(),
            fields: FfiIndexMap::new(),
        }
    }
    pub fn serialize_list<'r>(&'r mut self) -> ListSerializerCtx<'r, 'l>
        where 'a: 'r
    {
        ListSerializerCtx {
            ctx: self.as_mut(),
            elements: FfiVec::new(),
        }
    }

    pub fn get_field<T: 'static>(&mut self, value: &FfiIndexMap<String, SerializedValue>, name: &str) -> Option<T> {
        let serialized_value = value.get(name).unwrap_or_else(|| &SerializedValue::Null);

        self.deserialize(serialized_value)
    }
    pub fn get_element<T: 'static>(&mut self, value: &FfiVec<SerializedValue>, idx: usize) -> Option<T> {
        let serialized_value = value.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null);

        self.deserialize(serialized_value)
    }

    pub fn serialize<T: 'static>(&mut self, value: &T) -> SerializedValue {
        if let Some(serializer) = self.registry_local.as_option().map(|r| r.get()).flatten() {
            let ctx = SerializerCtx {
                registry_global: self.registry_global,
                registry_local: self.registry_local,
                err_handler: self.err_handler.as_mut(),
            };
            return serializer.serialize(ctx, value);
        }
        if let Some(serializer) = self.registry_global.get() {
            return serializer.serialize(self.as_mut(), value);
        }

        self.report_err(SerializationError::MissingSerializer { type_name: std::any::type_name::<T>().into() });
        SerializedValue::Null
    }

    pub fn serialize_any(&mut self, value: FfiAnyRef) -> SerializedValue {
        let type_name = value.type_info().short().name();
       
        unsafe {
            if let Some(serializer) = self.registry_local.as_option().map(|r| r.get_virtual(type_name)).flatten() {
                let ctx = SerializerCtx {
                    registry_global: self.registry_global,
                    registry_local: self.registry_local,
                    err_handler: self.err_handler.as_mut(),
                };
                return serializer.serialize_any(ctx, value);
            }
            if let Some(serializer) = self.registry_global.get_virtual(type_name) {
                return serializer.serialize_any(self.as_mut(), value);
            }
        }

        self.report_err(SerializationError::MissingSerializer { type_name: type_name.into() });
        SerializedValue::Null
    }
    pub fn deserialize<T: 'static>(&mut self, data: &SerializedValue) -> Option<T> {
        if let Some(serializer) = self.registry_local.as_option().map(|r| r.get()).flatten() {
            let ctx = SerializerCtx {
                registry_global: self.registry_global,
                registry_local: self.registry_local,
                err_handler: self.err_handler.as_mut(),
            };
            return serializer.deserialize(ctx, data);
        }
        if let Some(serializer) = self.registry_global.get() {
            return serializer.deserialize(self.as_mut(), data);
        }

        self.report_err(SerializationError::MissingSerializer { type_name: std::any::type_name::<T>().into() });
        None
    }
    pub fn deserialize_any(&mut self, id: &str, data: &SerializedValue) -> Option<FfiAny> {
        if let Some(serializer) = self.registry_local.as_option().map(|r| r.get_virtual(id)).flatten() {
            let ctx = SerializerCtx {
                registry_global: self.registry_global,
                registry_local: self.registry_local,
                err_handler: self.err_handler.as_mut(),
            };
            return serializer.deserialize_any(ctx, data);
        }
        if let Some(serializer) = self.registry_global.get_virtual(id) {
            return serializer.deserialize_any(self.as_mut(), data);
        }

        self.report_err(SerializationError::MissingSerializer { type_name: id.to_string().into() });
        None
    }
}

pub struct MapSerializerCtx<'a, 'l: 'a> {
    ctx: SerializerCtx<'a, 'l>,
    fields: FfiIndexMap<FfiString, SerializedValue>,
}

impl<'a, 'l: 'a> MapSerializerCtx<'a, 'l> {
    pub fn with_field<T: 'static>(mut self, name: impl Into<FfiString>, value: &T) -> Self {
        let result = self.ctx.serialize(value);

        self.fields.insert(name.into(), result);

        self
    }

    pub fn finish_as_map(self, is_rigid: bool) -> SerializedValue {
        SerializedValue::Composite(SerializedComposite {
            values: SerializedCompositeValues::Map(SerializedMap {
                values: self.fields,
                enum_metadata: None.into(),
            }),
            is_rigid,
        })
    }

    pub fn finish_as_enum(self, is_rigid: bool, variant: impl Into<FfiString>, variants: FfiVec<FfiString>) -> SerializedValue {
        SerializedValue::Composite(SerializedComposite {
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

// todo: ffi
pub struct ListSerializerCtx<'a, 'l: 'a> {
    ctx: SerializerCtx<'a, 'l>,
    elements: FfiVec<SerializedValue>,
}

impl<'a, 'l: 'a> ListSerializerCtx<'a, 'l> {
    pub fn with_element<T: 'static>(mut self, value: &T) -> Self {
        let result = self.ctx.serialize(value);

        self.with_serialized_element::<T>(result)
    }
    pub fn with_serialized_element<T: 'static>(mut self, result: SerializedValue) -> Self {
        self.elements.push(result);

        self
    }

    pub fn finish_as_list(self, is_rigid: bool) -> SerializedValue {
        SerializedValue::Composite(SerializedComposite {
            values: SerializedCompositeValues::List(self.elements),
            is_rigid,
        })
    }
}

//



// todo: ffi
pub struct MapDeserializerCtx<'a, 'l: 'a> {
    ctx: SerializerCtx<'a, 'l>,
    fields: FfiIndexMap<FfiString, SerializedValue>,
}

impl<'a, 'l: 'a> MapDeserializerCtx<'a, 'l> {
    pub fn new(ctx: SerializerCtx<'a, 'l>, value: &SerializedValue) -> Self {
        let fields = match value {
            SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::Map(value), .. }) => value.values.clone(),
            _ => FfiIndexMap::new()
        };

        Self {
            ctx,
            fields,
        }
    }

    pub fn get_field<T: 'static>(&mut self, name: &str) -> Option<T> {
        let serialized_value = self.fields.get(name).unwrap_or_else(|| &SerializedValue::Null);

        self.ctx.deserialize(serialized_value)
    }
}

// todo: ffi
pub struct ListDeserializerCtx<'a, 'l: 'a> {
    ctx: SerializerCtx<'a, 'l>,
    elements: FfiVec<SerializedValue>,
}

impl<'a, 'l: 'a> ListDeserializerCtx<'a, 'l> {
    pub fn new(ctx: SerializerCtx<'a, 'l>, value: &SerializedValue) -> Self {
        let elements = match value {
            SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::List(value), .. }) => value.clone(),
            _ => FfiVec::new()
        };

        Self {
            ctx,
            elements,
        }
    }

    pub fn get_element<T: 'static>(&mut self, idx: usize) -> Option<T> {
        let serialized_value = self.elements.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null);

        self.ctx.deserialize(serialized_value)
    }

    pub fn get_serialized_element<T: 'static>(&mut self, idx: usize) -> &SerializedValue {
        self.elements.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null)
    }
}

// todo: ffi
pub struct EnumDeserializerCtx<'a, 'l: 'a, T> {
    ctx: SerializerCtx<'a, 'l>,
    variants: FfiVec<(FfiString, Box<dyn for<'b> FnMut(SerializerCtx<'b, 'l>, &SerializedValue) -> Option<T>>)>,
}

impl<'a, 'l: 'a, T> EnumDeserializerCtx<'a, 'l, T> {
    pub fn new(ctx: SerializerCtx<'a, 'l>) -> Self {
        Self {
            ctx,
            variants: FfiVec::new(),
        }
    }
    pub fn variant(mut self, variant: impl Into<FfiString>, deserializer: impl 'static + for<'b> FnMut(SerializerCtx<'b, 'l>, &SerializedValue) -> Option<T>) -> Self {
        self.variants.push((variant.into(), Box::new(deserializer)));
        self
    }

    pub fn finish(mut self, value: &SerializedValue) -> Option<T> {
        if let SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::Map(SerializedMap { enum_metadata: FfiOption::Some(enum_metadata), .. }), .. }) = value {
            for (variant, deserializer) in &mut self.variants {
                if variant == &enum_metadata.variant {
                    if let Some(value) = deserializer(self.ctx.as_mut(), value) {
                        return Some(value);
                    }
                    break;
                }
            }
        }

        self.ctx.report_err(SerializationError::InvalidInput { message: format!("Deserializing enum {} without a valid variant", std::any::type_name::<T>()).into() });

        for (_variant, deserializer) in &mut self.variants {
            if let Some(value) = deserializer(self.ctx.as_mut(), value) {
                return Some(value);
            }
        }

        None
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
