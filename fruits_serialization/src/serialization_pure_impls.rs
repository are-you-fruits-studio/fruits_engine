use fruits_ffi::{FfiOption, FfiSmallString, FfiString, FfiVec};
use fruits_math::{Mat, Quat, Vec2, Vec3, Vec4};

use crate::{Serializable, SerializationError, SerializedComposite, SerializedCompositeValues, SerializedPrimitive, SerializedValue, PureSerializerCtx};

// todo: other types

impl Serializable for SerializedValue {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        self.clone()
    }

    fn deserialize(_ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        Some(value.clone())
    }
}

impl Serializable for String {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::String(self.clone().into()))
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        Some(match value {
            SerializedValue::Primitive(value) => match value {
                SerializedPrimitive::Bool(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing string from bool".into() });
                    value.to_string()
                },
                SerializedPrimitive::Int(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing string from int".into() });
                    value.to_string()
                },
                SerializedPrimitive::Float(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing string from float".into() });
                    value.to_string()
                },
                SerializedPrimitive::String(value) => value.to_string(),
            },
            SerializedValue::Null => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing string from null, using empty string".into() });
                String::new()
            },
            SerializedValue::Composite { .. } => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing string from composite, using empty string".into() });
                String::new()
            },
        })
    }
}

impl Serializable for FfiString {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::String(self.clone()))
    }

    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        <String as Serializable>::deserialize(ctx, value).map(FfiString::from)
    }
}

impl Serializable for FfiSmallString {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::String(self.as_str().into()))
    }

    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        <String as Serializable>::deserialize(ctx, value).map(|s| s.as_str().into())
    }
}

impl Serializable for &'static str {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::String((*self).into()))
    }

    fn deserialize(mut ctx: PureSerializerCtx, _value: &SerializedValue) -> Option<Self> {
        ctx.report_err(SerializationError::InvalidInput { message: "&str cannot be desrialized, using empty string".into() });
        Some("")
    }
}

impl Serializable for u128 {
    fn serialize(&self, mut ctx: PureSerializerCtx) -> SerializedValue {
        let value = if *self > i128::MAX as u128 {
            ctx.report_err(SerializationError::InvalidInput { message: "Int value is too high and is clamped.".into() });
            i128::MAX
        } else {
            *self as i128
        };

        SerializedValue::Primitive(SerializedPrimitive::Int(value))
    }

    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        Some(deserialize_int(ctx, value, 0, i128::MAX) as Self)
    }
}

macro_rules! trans_serializable_int_impl {
    ($T: ident) => {
        impl Serializable for $T {
            fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
                serialize_int(*self as i128)
            }
       
            fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
                Some(deserialize_int(ctx, value, Self::MIN as i128, Self::MAX as i128) as Self)
            }
        }
    };
}
macro_rules! trans_serializable_float_impl {
    ($T: ident) => {
        impl Serializable for $T {
            fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
                serialize_float(*self as f64)
            }
       
            fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
                Some(deserialize_float(ctx, value) as Self)
            }
        }
    };
}

fn serialize_int(value: i128) -> SerializedValue {
    SerializedValue::Primitive(SerializedPrimitive::Int(value))
}

fn serialize_float(value: f64) -> SerializedValue {
    SerializedValue::Primitive(SerializedPrimitive::Float(value))
}

fn deserialize_int(mut ctx: PureSerializerCtx, value: &SerializedValue, min: i128, max: i128) -> i128 {
    match value {
        SerializedValue::Null => {
            ctx.report_err(SerializationError::InvalidInput { message: "Deserializing int from null, using 0".into() });
            0
        },
        SerializedValue::Composite { .. } => {
            ctx.report_err(SerializationError::InvalidInput { message: "Deserializing int from composite, using 0".into() });
            0
        },
        SerializedValue::Primitive(value) => {
            match value {
                SerializedPrimitive::Bool(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing int from bool".into() });
                    if *value { 1 } else { 0 }
                },
                SerializedPrimitive::Int(value) => {
                    let clamped_value = (*value).clamp(min, max);
                    if *value != clamped_value {
                        ctx.report_err(SerializationError::InvalidInput { message: "Int value is too high/low and is clamped.".into() });
                    }
                    clamped_value
                },
                SerializedPrimitive::Float(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing int from float".into() });
                    (*value as i128).clamp(min, max)
                },
                SerializedPrimitive::String(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing int from string".into() });
                    value.parse::<i128>().unwrap_or(0).clamp(min, max)
                },
            }
        },
    }
}

fn deserialize_float(mut ctx: PureSerializerCtx, value: &SerializedValue) -> f64 {
    match value {
        SerializedValue::Null => {
            ctx.report_err(SerializationError::InvalidInput { message: "Deserializing float from null, using 0".into() });
            0.0
        },
        SerializedValue::Composite { .. } => {
            ctx.report_err(SerializationError::InvalidInput { message: "Deserializing float from composite, using 0".into() });
            0.0
        },
        SerializedValue::Primitive(value) => {
            match value {
                SerializedPrimitive::Bool(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing float from bool".into() });
                    if *value { 1.0 } else { 0.0 }
                },
                SerializedPrimitive::Int(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing float from int".into() });
                    *value as f64
                },
                SerializedPrimitive::Float(value) => {
                    *value
                },
                SerializedPrimitive::String(value) => {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing float from string".into() });
                    value.parse::<f64>().unwrap_or(0.0)
                },
            }
        },
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

impl Serializable for bool {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::Bool(*self))
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        Some(match value {
            SerializedValue::Null => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing bool from null, using false".into() });
                false
            },
            SerializedValue::Composite { .. } => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing bool from composite, using false".into() });
                false
            },
            SerializedValue::Primitive(value) => {
                match value {
                    SerializedPrimitive::Bool(value) => {
                        *value
                    },
                    SerializedPrimitive::Int(value) => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing bool from int".into() });
                        *value != 0
                    },
                    SerializedPrimitive::Float(value) => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing bool from float".into() });
                        *value != 0.0
                    },
                    SerializedPrimitive::String(value) => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing bool from string".into() });
                        match value.as_str().trim() {
                            "1" | "true" | "True" | "TRUE" => true,
                            _ => false
                        }
                    },
                }
            },
        })
    }
}

impl Serializable for char {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Primitive(SerializedPrimitive::String(String::from(*self).into()))
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        Some(match value {
            SerializedValue::Null => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing char from null, using default".into() });
                char::default()
            },
            SerializedValue::Composite { .. } => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing char from composite, using default".into() });
                char::default()
            },
            SerializedValue::Primitive(value) => {
                match value {
                    SerializedPrimitive::Bool(value) => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing char from bool".into() });
                        value.to_string().chars().next().unwrap_or_default()
                    },
                    SerializedPrimitive::Int(value) => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing char from int".into() });
                        value.to_string().chars().next().unwrap_or_default()
                    },
                    SerializedPrimitive::Float(value) => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing char from float".into() });
                        value.to_string().chars().next().unwrap_or_default()
                    },
                    SerializedPrimitive::String(value) => {
                        let mut chars = value.chars();

                        if !(chars.next().is_some() && chars.next().is_none()) {
                            ctx.report_err(SerializationError::InvalidInput { message: "Deserializing char from string".into() });
                        }

                        value.chars().next().unwrap_or_default()
                    },
                }
            },
        })
    }
}

fn serialize_slice<T: 'static + Serializable>(slice: &[T], mut ctx: PureSerializerCtx) -> SerializedValue {
    let mut vec = FfiVec::new();

    for element in slice {
        vec.push(ctx.serialize(element));
    }
   
    SerializedValue::Composite(SerializedComposite {
        is_rigid: false,
        values: SerializedCompositeValues::List(vec),
    })
}

impl<T: 'static + Serializable> Serializable for Vec<T> {
    fn serialize(&self, ctx: PureSerializerCtx) -> SerializedValue {
        serialize_slice(self.as_slice(), ctx)
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        let values = match value {
            SerializedValue::Null => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing vec from null, using default".into() });
                &[]
            },
            SerializedValue::Primitive { .. } => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing vec from primitive".into() });
                std::slice::from_ref(value)
            },
            SerializedValue::Composite(composite) => {
                if composite.is_rigid {
                    ctx.report_err(SerializationError::InvalidInput { message: "Deserializing vec from something rigid: like struct/tuple instead of list/map".into() });
                }

                match &composite.values {
                    SerializedCompositeValues::Map { .. } => {
                        ctx.report_err(SerializationError::InvalidInput { message: "Deserializing vec from named values".into() });
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

            if let Some(result) = result {
                vec.push(result);
            };
        };

        Some(vec)
    }
}

impl<T: 'static + Serializable> Serializable for FfiVec<T> {
    fn serialize(&self, ctx: PureSerializerCtx) -> SerializedValue {
        serialize_slice(self.as_slice(), ctx)
    }
   
    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        <Vec<T> as Serializable>::deserialize(ctx, value).map(FfiVec::from)
    }
}

fn serialize_option<T: 'static + Serializable>(option: Option<&T>, mut ctx: PureSerializerCtx) -> SerializedValue {
    let variants = ["None", "Some"].into_iter().map(FfiString::from).collect();

    match option {   
        None => ctx.serialize_map()
            .finish_as_enum(true, "None", variants),
        Some(value) => ctx.serialize_map()
            .with_field("0", value)
            .finish_as_enum(true, "Some", variants),
    }
}

impl<T: 'static + Serializable> Serializable for Option<T> {
    fn serialize(&self, ctx: PureSerializerCtx) -> SerializedValue {
        serialize_option(self.as_ref(), ctx)
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_enum()
            .variant("None", |mut ctx, value| {
                ctx.deserialize_map(value, |_ctx| {
                    Some(None)
                })
            })
            .variant("Some", |mut ctx, value| {
                ctx.deserialize_map(value, |mut ctx| {
                    Some(Some(ctx.get_field("0")?))
                })
            })
            .finish(value)
    }
}

impl<T: 'static + Serializable> Serializable for FfiOption<T> {
    fn serialize(&self, ctx: PureSerializerCtx) -> SerializedValue {
        serialize_option(self.as_ref(), ctx)
    }

    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        <Option<T> as Serializable>::deserialize(ctx, value).map(FfiOption::from)
    }
}

impl Serializable for () {
    fn serialize(&self, _ctx: PureSerializerCtx) -> SerializedValue {
        SerializedValue::Null
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        match value {
            SerializedValue::Null => (),
            SerializedValue::Primitive { .. } => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing unit from primitive".into() });
            },
            SerializedValue::Composite { .. } => {
                ctx.report_err(SerializationError::InvalidInput { message: "Deserializing unit from composite".into() });
            },
        };

        Some(())
    }
}

// todo: Mat, Quat, VecN
impl<T: 'static + Serializable> Serializable for Quat<T> {
    fn serialize(&self, mut ctx: PureSerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .with_field("z", &self.z)
            .with_field("w", &self.w)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_map(value, |mut ctx| {
            Some(Self {
                x: ctx.get_field("x")?,
                y: ctx.get_field("y")?,
                z: ctx.get_field("z")?,
                w: ctx.get_field("w")?,
            })
        })
    }
}

impl<T: 'static + Serializable> Serializable for Vec2<T> {
    fn serialize(&self, mut ctx: PureSerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_map(value, |mut ctx| {
            Some(Self {
                x: ctx.get_field("x")?,
                y: ctx.get_field("y")?,
            })
        })
    }
}

impl<T: 'static + Serializable> Serializable for Vec3<T> {
    fn serialize(&self, mut ctx: PureSerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .with_field("z", &self.z)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_map(value, |mut ctx| {
            Some(Self {
                x: ctx.get_field("x")?,
                y: ctx.get_field("y")?,
                z: ctx.get_field("z")?,
            })
        })
    }
}

impl<T: 'static + Serializable> Serializable for Vec4<T> {
    fn serialize(&self, mut ctx: PureSerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .with_field("z", &self.z)
            .with_field("w", &self.w)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_map(value, |mut ctx| {
            Some(Self {
                x: ctx.get_field("x")?,
                y: ctx.get_field("y")?,
                z: ctx.get_field("z")?,
                w: ctx.get_field("w")?,
            })
        })
    }
}

// todo
impl<const N: usize, T: 'static + Serializable> Serializable for Mat<N, T> {
    fn serialize(&self, ctx: PureSerializerCtx) -> SerializedValue {
        todo!();
        // let mut columns_list = ctx.serialize_list();

        // for column in self.as_array() {
        //     let mut elements_list = ctx.serialize_list();

        //     for ele in column {
        //         elements_list = elements_list.with_element(ele);
        //     }

        //     let elements_list = elements_list.finish_as_list(true);
        //     columns_list = columns_list.with_serialized_element(elements_list);
        // }

        // columns_list.finish_as_list(true)
    }

    fn deserialize(ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> {
        todo!();
        // ctx.deserialize_list(value, |ctx_list| {
        //     let mut columns = [MaybeUninit::<[T; N]>::uninit(); N];

        //     for column_idx in 0..N {
        //         let mut elements = [MaybeUninit::<T>::uninit(); N];

        //         for i in 0..N {
        //             elements[i].write(ctx.deserialize(ctx.get_serialized_element(idx)));
        //         }
               
        //         columns[column_idx].write(unsafe { transmute(elements) });
        //     }

        //     unsafe { Some(transmute(columns)) }
        // })

        // //

        // ctx.deserialize_list(value, |ctx| {

        //     for i in 0..N {
        //         elements[i].write(ctx.get_element(i)?);
        //     }

        // })
    }
}

// todo: other impls (for tuples, maps, arrays, ...)