use fruits_ffi::{FfiOption, FfiString, FfiVec};

use crate::{GlobalSerializer, SerializationError, SerializationResult, SerializedComposite, SerializedCompositeValues, SerializedPrimitive, SerializedValue, SerializerCtx, StandardTransSerializer, TransSerializable};

// todo: other types

impl TransSerializable for SerializedValue {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            result: self.clone(),
            err: FfiVec::new(),
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        SerializationResult {
            result: Some(value.clone()),
            err: FfiVec::new(),
        }
    }
}

impl TransSerializable for String {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            result: SerializedValue::Primitive(SerializedPrimitive::String(self.clone().into())),
            err: FfiVec::new(),
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        let mut err = FfiVec::new();

        SerializationResult {
            result: Some(match value {
                SerializedValue::Primitive(value) => match value {
                    SerializedPrimitive::Bool(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing string from bool") });
                        value.to_string()
                    },
                    SerializedPrimitive::Int(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing string from int") });
                        value.to_string()
                    },
                    SerializedPrimitive::Float(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing string from float") });
                        value.to_string()
                    },
                    SerializedPrimitive::String(value) => value.to_string(),
                },
                SerializedValue::Null => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing string from null, using empty string") });
                    String::new()
                },
                SerializedValue::Composite { .. } => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing string from composite, using empty string") });
                    String::new()
                },
            }),
            err,
        }
    }
}

impl TransSerializable for FfiString {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            result: SerializedValue::Primitive(SerializedPrimitive::String(self.clone())),
            err: FfiVec::new(),
        }
    }

    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        <String as TransSerializable>::deserialize(ctx, value).map(|o| o.map(FfiString::from))
    }
}

impl TransSerializable for &'static str {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            result: SerializedValue::Primitive(SerializedPrimitive::String((*self).into())),
            err: FfiVec::new(),
        }
    }

    fn deserialize(_ctx: &SerializerCtx, _value: &SerializedValue) -> SerializationResult<Option<Self>> {
        SerializationResult {
            result: Some(Default::default()),
            err: vec![SerializationError::InvalidInput { message: String::from("&str cannot be desrialized, using empty string") }].into(),
        }
    }
}

impl TransSerializable for u128 {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        let mut err = FfiVec::new();

        let value = if *self > i128::MAX as u128 {
            err.push(SerializationError::InvalidInput { message: String::from("Int value is too high and is clamped.") });
            i128::MAX
        } else {
            *self as i128
        };

        SerializationResult {
            result: SerializedValue::Primitive(SerializedPrimitive::Int(value)),
            err,
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        deserialize_int(value, 0, i128::MAX).map(|v| Some(v as Self))
    }
}

macro_rules! trans_serializable_int_impl {
    ($T: ident) => {
        impl TransSerializable for $T {
            fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
                serialize_int(*self as i128)
            }
        
            fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
                deserialize_int(value, Self::MIN as i128, Self::MAX as i128).map(|v| Some(v as Self))
            }
        }
    };
}
macro_rules! trans_serializable_float_impl {
    ($T: ident) => {
        impl TransSerializable for $T {
            fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
                serialize_float(*self as f64)
            }
        
            fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
                deserialize_float(value).map(|v| Some(v as Self))
            }
        }
    };
}

fn serialize_int(value: i128) -> SerializationResult<SerializedValue> {
    SerializationResult {
        err: FfiVec::new(),
        result: SerializedValue::Primitive(SerializedPrimitive::Int(value))
    }
}

fn serialize_float(value: f64) -> SerializationResult<SerializedValue> {
    SerializationResult {
        err: FfiVec::new(),
        result: SerializedValue::Primitive(SerializedPrimitive::Float(value))
    }
}

fn deserialize_int(value: &SerializedValue, min: i128, max: i128) -> SerializationResult<i128> {
    let mut err = FfiVec::new();

    SerializationResult {
        result: match value {
            SerializedValue::Null => {
                err.push(SerializationError::InvalidInput { message: String::from("Deserializing int from null, using 0") });
                0
            },
            SerializedValue::Composite { .. } => {
                err.push(SerializationError::InvalidInput { message: String::from("Deserializing int from composite, using 0") });
                0
            },
            SerializedValue::Primitive(value) => {
                match value {
                    SerializedPrimitive::Bool(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing int from bool") });
                        if *value { 1 } else { 0 }
                    },
                    SerializedPrimitive::Int(value) => {
                        let clamped_value = (*value).clamp(min, max);
                        if *value != clamped_value {
                            err.push(SerializationError::InvalidInput { message: String::from("Int value is too high/low and is clamped.") });
                        }
                        clamped_value
                    },
                    SerializedPrimitive::Float(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing int from float") });
                        (*value as i128).clamp(min, max)
                    },
                    SerializedPrimitive::String(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing int from string") });
                        value.parse::<i128>().unwrap_or(0).clamp(min, max)
                    },
                }
            },
        },
        err,
    }
}

fn deserialize_float(value: &SerializedValue) -> SerializationResult<f64> {
    let mut err = FfiVec::new();

    SerializationResult {
        result: match value {
            SerializedValue::Null => {
                err.push(SerializationError::InvalidInput { message: String::from("Deserializing float from null, using 0") });
                0.0
            },
            SerializedValue::Composite { .. } => {
                err.push(SerializationError::InvalidInput { message: String::from("Deserializing float from composite, using 0") });
                0.0
            },
            SerializedValue::Primitive(value) => {
                match value {
                    SerializedPrimitive::Bool(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing float from bool") });
                        if *value { 1.0 } else { 0.0 }
                    },
                    SerializedPrimitive::Int(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing float from int") });
                        *value as f64
                    },
                    SerializedPrimitive::Float(value) => {
                        *value
                    },
                    SerializedPrimitive::String(value) => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing float from string") });
                        value.parse::<f64>().unwrap_or(0.0)
                    },
                }
            },
        },
        err,
    }
}

trans_serializable_int_impl!(u8);
trans_serializable_int_impl!(i8);
trans_serializable_int_impl!(u16);
trans_serializable_int_impl!(i16);
trans_serializable_int_impl!(u32);
trans_serializable_int_impl!(i32);
trans_serializable_int_impl!(u64);
trans_serializable_int_impl!(i64);
trans_serializable_int_impl!(i128);
trans_serializable_float_impl!(f32);
trans_serializable_float_impl!(f64);

impl TransSerializable for bool {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            err: FfiVec::new(),
            result: SerializedValue::Primitive(SerializedPrimitive::Bool(*self))
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        let mut err = FfiVec::new();

        SerializationResult {
            result: Some(match value {
                SerializedValue::Null => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing bool from null, using false") });
                    false
                },
                SerializedValue::Composite { .. } => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing bool from composite, using false") });
                    false
                },
                SerializedValue::Primitive(value) => {
                    match value {
                        SerializedPrimitive::Bool(value) => {
                            *value
                        },
                        SerializedPrimitive::Int(value) => {
                            err.push(SerializationError::InvalidInput { message: String::from("Deserializing bool from int") });
                            *value != 0
                        },
                        SerializedPrimitive::Float(value) => {
                            err.push(SerializationError::InvalidInput { message: String::from("Deserializing bool from float") });
                            *value != 0.0
                        },
                        SerializedPrimitive::String(value) => {
                            err.push(SerializationError::InvalidInput { message: String::from("Deserializing bool from string") });
                            match value.as_str().trim() {
                                "1" | "true" | "True" | "TRUE" => true,
                                _ => false
                            }
                        },
                    }
                },
            }),
            err,
        }
    }
}

impl TransSerializable for char {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            result: SerializedValue::Primitive(SerializedPrimitive::String(String::from(*self).into())),
            err: FfiVec::new(),
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        let mut err = FfiVec::new();

        SerializationResult {
            result: Some(match value {
                SerializedValue::Null => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing char from null, using default") });
                    char::default()
                },
                SerializedValue::Composite { .. } => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing char from composite, using default") });
                    char::default()
                },
                SerializedValue::Primitive(value) => {
                    match value {
                        SerializedPrimitive::Bool(value) => {
                            err.push(SerializationError::InvalidInput { message: String::from("Deserializing char from bool") });
                            value.to_string().chars().next().unwrap_or_default()
                        },
                        SerializedPrimitive::Int(value) => {
                            err.push(SerializationError::InvalidInput { message: String::from("Deserializing char from int") });
                            value.to_string().chars().next().unwrap_or_default()
                        },
                        SerializedPrimitive::Float(value) => {
                            err.push(SerializationError::InvalidInput { message: String::from("Deserializing char from float") });
                            value.to_string().chars().next().unwrap_or_default()
                        },
                        SerializedPrimitive::String(value) => {
                            let mut chars = value.chars();

                            if !(chars.next().is_some() && chars.next().is_none()) {
                                err.push(SerializationError::InvalidInput { message: String::from("Deserializing char from string") });
                            }

                            value.chars().next().unwrap_or_default()
                        },
                    }
                },
            }),
            err,
        }
    }
}

fn serialize_slice<T: 'static>(slice: &[T], ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
    let mut err = FfiVec::new();
    let mut vec = FfiVec::new();

    for element in slice {
        let result = ctx.serialize(element);

        err.extend_from_slice(&result.err);
        vec.push(result.result);
    }
    
    SerializationResult {
        result: SerializedValue::Composite(SerializedComposite {
            is_rigid: false,
            values: SerializedCompositeValues::List(vec),
        }),
        err,
    }
}

impl<T: 'static> TransSerializable for Vec<T> {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        serialize_slice(self.as_slice(), ctx)
    }

    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        let mut err = FfiVec::new();

        let values = match value {
            SerializedValue::Null => {
                err.push(SerializationError::InvalidInput { message: String::from("Deserializing vec from null, using default") });
                &[]
            },
            SerializedValue::Primitive { .. } => {
                err.push(SerializationError::InvalidInput { message: String::from("Deserializing vec from primitive") });
                std::slice::from_ref(value)
            },
            SerializedValue::Composite(composite) => {
                if composite.is_rigid {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing vec from something rigid: like struct/tuple instead of list/map") });
                }

                match &composite.values {
                    SerializedCompositeValues::Map { .. } => {
                        err.push(SerializationError::InvalidInput { message: String::from("Deserializing vec from named values") });
                        std::slice::from_ref(value)
                    },
                    SerializedCompositeValues::List(tuple) => {
                        tuple.as_slice()
                    },
                }
            },
        };

        let mut vec = Vec::new();

        for element in values {
            let result = ctx.deserialize::<T>(element);

            err.extend_from_slice(&result.err);

            if let Some(result) = result.result {
                vec.push(result);
            };
        };

        SerializationResult {
            result: Some(vec),
            err: err,
        }
    }
}

impl<T: 'static> TransSerializable for FfiVec<T> {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        serialize_slice(self.as_slice(), ctx)
    }
    
    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        <Vec<T> as TransSerializable>::deserialize(ctx, value).map(|o| o.map(FfiVec::from))
    }
}

fn serialize_option<T: 'static>(option: Option<&T>, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
    let variants = ["None", "Some"].into_iter().map(FfiString::from).collect();

    match option {    
        None => ctx.serialize_map()
            .finish_as_enum(true, "None", variants),
        Some(value) => ctx.serialize_map()
            .with_field("0", value)
            .finish_as_enum(true, "Some", variants),
    }
}

impl<T: 'static> TransSerializable for Option<T> {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        serialize_option(self.as_ref(), ctx)
    }

    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        ctx.deserialize_enum()
            .variant("None", |ctx, value| {
                ctx.deserialize_map(value, |_ctx| {
                    Some(None)
                })
            })
            .variant("Some", |ctx, value| {
                ctx.deserialize_map(value, |ctx| {
                    Some(Some(ctx.get_field("0")?))
                })
            })
            .finish(value)
    }
}

impl<T: 'static> TransSerializable for FfiOption<T> {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        serialize_option(self.as_ref(), ctx)
    }

    fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        <Option<T> as TransSerializable>::deserialize(ctx, value).map(|o| o.map(FfiOption::from))
    }
}

impl TransSerializable for () {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
        SerializationResult {
            result: SerializedValue::Null,
            err: FfiVec::new(),
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
        let mut err = FfiVec::new();

        SerializationResult {
            result: Some(match value {
                SerializedValue::Null => (),
                SerializedValue::Primitive { .. } => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing unit from primitive") });
                    ()
                },
                SerializedValue::Composite { .. } => {
                    err.push(SerializationError::InvalidInput { message: String::from("Deserializing unit from composite") });
                    ()
                },
            }),
            err
        }
    }
}

// todo: other impls (for tuples, maps, arrays, ...)