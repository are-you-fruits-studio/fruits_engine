use fruits_ffi::{FfiOption, FfiSmallString, FfiString, FfiVec};
use fruits_math::{Mat, Quat, Vec2, Vec3, Vec4};

use crate::{
    Serializable, SerializationError, SerializedComposite, SerializedCompositeValues, SerializedValue, SerializerCtx,
    TransSerializable,
};

// todo: other types

macro_rules! impl_route_to_serializable {
    ($T: ty) => {
        impl TransSerializable for $T {
            fn serialize(&self, mut ctx: SerializerCtx) -> SerializedValue {
                Serializable::serialize(self, ctx.as_pure())
            }

            fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
                Serializable::deserialize(ctx.as_pure(), value)
            }
        }
    };
}

impl_route_to_serializable!(SerializedValue);
impl_route_to_serializable!(String);
impl_route_to_serializable!(FfiString);
impl_route_to_serializable!(FfiSmallString);
impl_route_to_serializable!(&'static str);
impl_route_to_serializable!(u8);
impl_route_to_serializable!(i8);
impl_route_to_serializable!(u16);
impl_route_to_serializable!(i16);
impl_route_to_serializable!(u32);
impl_route_to_serializable!(i32);
impl_route_to_serializable!(u64);
impl_route_to_serializable!(i64);
impl_route_to_serializable!(i128);
impl_route_to_serializable!(u128);
impl_route_to_serializable!(f32);
impl_route_to_serializable!(f64);
impl_route_to_serializable!(bool);
impl_route_to_serializable!(char);
impl_route_to_serializable!(());

fn serialize_slice<T: 'static>(slice: &[T], mut ctx: SerializerCtx) -> SerializedValue {
    let mut vec = FfiVec::new();

    for element in slice {
        vec.push(ctx.serialize(element));
    }

    SerializedValue::Composite(SerializedComposite {
        is_rigid: false,
        values: SerializedCompositeValues::List(vec),
    })
}

impl<T: 'static> TransSerializable for Vec<T> {
    fn serialize(&self, ctx: SerializerCtx) -> SerializedValue {
        serialize_slice(self.as_slice(), ctx)
    }

    fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
        let values = match value {
            SerializedValue::Null => {
                ctx.report_err(SerializationError::InvalidInput {
                    message: "Deserializing vec from null, using default".into(),
                });
                &[]
            }
            SerializedValue::Primitive { .. } => {
                ctx.report_err(SerializationError::InvalidInput {
                    message: "Deserializing vec from primitive".into(),
                });
                std::slice::from_ref(value)
            }
            SerializedValue::Composite(composite) => {
                if composite.is_rigid {
                    ctx.report_err(SerializationError::InvalidInput {
                        message: "Deserializing vec from something rigid: like struct/tuple instead of list/map".into(),
                    });
                }

                match &composite.values {
                    SerializedCompositeValues::Map { .. } => {
                        ctx.report_err(SerializationError::InvalidInput {
                            message: "Deserializing vec from named values".into(),
                        });
                        std::slice::from_ref(value)
                    }
                    SerializedCompositeValues::List(tuple) => tuple.as_slice(),
                }
            }
        };

        let mut vec = Vec::new();

        for element in values {
            let result = ctx.deserialize::<T>(element);

            if let Some(result) = result {
                vec.push(result);
            };
        }

        Some(vec)
    }
}

impl<T: 'static> TransSerializable for FfiVec<T> {
    fn serialize(&self, ctx: SerializerCtx) -> SerializedValue {
        serialize_slice(self.as_slice(), ctx)
    }

    fn deserialize(ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
        <Vec<T> as TransSerializable>::deserialize(ctx, value).map(FfiVec::from)
    }
}

fn serialize_option<T: 'static>(option: Option<&T>, mut ctx: SerializerCtx) -> SerializedValue {
    let variants = ["None", "Some"].into_iter().map(FfiString::from).collect();

    match option {
        None => ctx.serialize_map().finish_as_enum(true, "None", variants),
        Some(value) => ctx
            .serialize_map()
            .with_field("0", value)
            .finish_as_enum(true, "Some", variants),
    }
}

impl<T: 'static> TransSerializable for Option<T> {
    fn serialize(&self, ctx: SerializerCtx) -> SerializedValue {
        serialize_option(self.as_ref(), ctx)
    }

    fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_enum()
            .variant("None", |mut ctx, value| ctx.deserialize_map(value, |_ctx| Some(None)))
            .variant("Some", |mut ctx, value| {
                ctx.deserialize_map(value, |mut ctx| Some(Some(ctx.get_field("0")?)))
            })
            .finish(value)
    }
}

impl<T: 'static> TransSerializable for FfiOption<T> {
    fn serialize(&self, ctx: SerializerCtx) -> SerializedValue {
        serialize_option(self.as_ref(), ctx)
    }

    fn deserialize(ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
        <Option<T> as TransSerializable>::deserialize(ctx, value).map(FfiOption::from)
    }
}

// todo: Mat, Quat, VecN
impl<T: 'static> TransSerializable for Quat<T> {
    fn serialize(&self, mut ctx: SerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .with_field("z", &self.z)
            .with_field("w", &self.w)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
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

impl<T: 'static> TransSerializable for Vec2<T> {
    fn serialize(&self, mut ctx: SerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_map(value, |mut ctx| {
            Some(Self {
                x: ctx.get_field("x")?,
                y: ctx.get_field("y")?,
            })
        })
    }
}

impl<T: 'static> TransSerializable for Vec3<T> {
    fn serialize(&self, mut ctx: SerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .with_field("z", &self.z)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
        ctx.deserialize_map(value, |mut ctx| {
            Some(Self {
                x: ctx.get_field("x")?,
                y: ctx.get_field("y")?,
                z: ctx.get_field("z")?,
            })
        })
    }
}

impl<T: 'static> TransSerializable for Vec4<T> {
    fn serialize(&self, mut ctx: SerializerCtx) -> SerializedValue {
        ctx.serialize_map()
            .with_field("x", &self.x)
            .with_field("y", &self.y)
            .with_field("z", &self.z)
            .with_field("w", &self.w)
            .finish_as_map(true)
    }

    fn deserialize(mut ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
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
impl<const N: usize, T: 'static> TransSerializable for Mat<N, T> {
    fn serialize(&self, ctx: SerializerCtx) -> SerializedValue {
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

    fn deserialize(ctx: SerializerCtx, value: &SerializedValue) -> Option<Self> {
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
