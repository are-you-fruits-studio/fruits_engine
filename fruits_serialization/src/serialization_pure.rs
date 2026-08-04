use fruits_ffi::{FfiFnMutMut, FfiIndexMap, FfiOption, FfiString, FfiVec};

use crate::{SerializationError, SerializedComposite, SerializedCompositeValues, SerializedEnumMetadata, SerializedMap, SerializedValue};

// todo: ffi

pub trait Serializable: Sized + 'static {
    fn serialize(&self, ctx: PureSerializerCtx) -> SerializedValue;
    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self>;
}

#[repr(C)]
pub struct PureSerializerCtx<'a> {
    err_handler: FfiFnMutMut<'a, SerializationError, ()>,
}
impl<'a> PureSerializerCtx<'a> {
    pub fn new(err_handler: FfiFnMutMut<'a, SerializationError, ()>) -> Self {
        Self {
            err_handler,
        }
    }

    pub fn as_mut<'r>(&'r mut self) -> PureSerializerCtx<'r>
        where 'a: 'r
    {
        PureSerializerCtx {
            err_handler: self.err_handler.as_mut(),
        }
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
        f: impl FnOnce(PureMapDeserializerCtx) -> Option<T>,
    ) -> Option<T> {
        f(PureMapDeserializerCtx::new(
            self.as_mut(),
            value,
        ))
    }
    pub fn deserialize_list<T: 'static>(
        &mut self,
        value: &SerializedValue,
        f: impl FnOnce(PureListDeserializerCtx) -> Option<T>,
    ) -> Option<T> {
        f(PureListDeserializerCtx::new(
            self.as_mut(),
            value,
        ))
    }
    pub fn deserialize_enum<'r, T>(&'r mut self) -> PureEnumDeserializerCtx<'r, T>
        where 'a: 'r
    {
        PureEnumDeserializerCtx::<T>::new(self.as_mut())
    }

    pub fn serialize_map<'r>(&'r mut self) -> PureMapSerializerCtx<'r>
        where 'a: 'r
    {
        PureMapSerializerCtx {
            ctx: self.as_mut(),
            fields: FfiIndexMap::new(),
        }
    }
    pub fn serialize_list<'r>(&'r mut self) -> PureListSerializerCtx<'r>
        where 'a: 'r
    {
        PureListSerializerCtx {
            ctx: self.as_mut(),
            elements: FfiVec::new(),
        }
    }

    pub fn get_field<T: 'static + Serializable>(&mut self, value: &FfiIndexMap<String, SerializedValue>, name: &str) -> Option<T> {
        let serialized_value = value.get(name).unwrap_or_else(|| &SerializedValue::Null);

        self.deserialize(serialized_value)
    }
    pub fn get_element<T: 'static + Serializable>(&mut self, value: &FfiVec<SerializedValue>, idx: usize) -> Option<T> {
        let serialized_value = value.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null);

        self.deserialize(serialized_value)
    }

    pub fn serialize<T: 'static + Serializable>(&mut self, value: &T) -> SerializedValue {
        value.serialize(self.as_mut())
    }

    pub fn deserialize<T: 'static + Serializable>(&mut self, data: &SerializedValue) -> Option<T> {
        T::deserialize(self.as_mut(), data)
    }
}

pub struct PureMapSerializerCtx<'a> {
    ctx: PureSerializerCtx<'a>,
    fields: FfiIndexMap<FfiString, SerializedValue>,
}

impl<'a> PureMapSerializerCtx<'a> {
    pub fn with_field<T: 'static + Serializable>(mut self, name: impl Into<FfiString>, value: &T) -> Self {
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
pub struct PureListSerializerCtx<'a> {
    ctx: PureSerializerCtx<'a>,
    elements: FfiVec<SerializedValue>,
}

impl<'a> PureListSerializerCtx<'a> {
    pub fn with_element<T: 'static + Serializable>(mut self, value: &T) -> Self {
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
pub struct PureMapDeserializerCtx<'a> {
    ctx: PureSerializerCtx<'a>,
    fields: FfiIndexMap<FfiString, SerializedValue>,
}

impl<'a> PureMapDeserializerCtx<'a> {
    pub fn new(ctx: PureSerializerCtx<'a>, value: &SerializedValue) -> Self {
        let fields = match value {
            SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::Map(value), .. }) => value.values.clone(),
            _ => FfiIndexMap::new()
        };

        Self {
            ctx,
            fields,
        }
    }

    pub fn get_field<T: 'static + Serializable>(&mut self, name: &str) -> Option<T> {
        let serialized_value = self.fields.get(name).unwrap_or_else(|| &SerializedValue::Null);

        self.ctx.deserialize(serialized_value)
    }
}

// todo: ffi
pub struct PureListDeserializerCtx<'a> {
    ctx: PureSerializerCtx<'a>,
    elements: FfiVec<SerializedValue>,
}

impl<'a> PureListDeserializerCtx<'a> {
    pub fn new(ctx: PureSerializerCtx<'a>, value: &SerializedValue) -> Self {
        let elements = match value {
            SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::List(value), .. }) => value.clone(),
            _ => FfiVec::new()
        };

        Self {
            ctx,
            elements,
        }
    }

    pub fn get_element<T: 'static + Serializable>(&mut self, idx: usize) -> Option<T> {
        let serialized_value = self.elements.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null);

        self.ctx.deserialize(serialized_value)
    }

    pub fn get_serialized_element<T: 'static>(&mut self, idx: usize) -> &SerializedValue {
        self.elements.get(idx as u64).unwrap_or_else(|| &SerializedValue::Null)
    }
}

// todo: ffi
pub struct PureEnumDeserializerCtx<'a, T> {
    ctx: PureSerializerCtx<'a>,
    variants: FfiVec<(FfiString, Box<dyn for<'b> FnMut(PureSerializerCtx<'b>, &SerializedValue) -> Option<T>>)>,
}

impl<'a, T> PureEnumDeserializerCtx<'a, T> {
    pub fn new(ctx: PureSerializerCtx<'a>) -> Self {
        Self {
            ctx,
            variants: FfiVec::new(),
        }
    }
    pub fn variant(mut self, variant: impl Into<FfiString>, deserializer: impl 'static + for<'b> FnMut(PureSerializerCtx<'b>, &SerializedValue) -> Option<T>) -> Self {
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
