use crate::{SerializationError, SerializationResult, SerializerCtx, TransSerializable};

// todo: other types

impl TransSerializable for String {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
        SerializationResult {
            result: serde_json::Value::String(self.clone()),
            err: None,
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
        Ok(match value {
            serde_json::Value::String(string) => SerializationResult { result: string.clone(), err: None },
            _ => SerializationResult { result: Default::default(), err: Some(SerializationError::InvalidInput) },
        })
    }
}

impl TransSerializable for &'static str {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
        SerializationResult {
            result: serde_json::Value::String(String::from(*self)),
            err: None,
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
        Ok(match value {
            serde_json::Value::String(string) if string.len() == 0 => SerializationResult { result: "", err: None },
            _ => SerializationResult { result: Default::default(), err: Some(SerializationError::InvalidInput) },
        })
    }
}

macro_rules! trans_serializable_int_impl {
    ($T: ident) => {
        impl TransSerializable for $T {
            fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
                SerializationResult { result: number_from_i(*self as i128).into(), err: None }
            }
        
            fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
                Ok(match value {
                    serde_json::Value::Number(num) => SerializationResult { result: number_to_i(num, $T::MIN as i128, $T::MAX as i128) as $T, err: None },
                    _ => SerializationResult { result: Default::default(), err: Some(SerializationError::InvalidInput) },
                })
            }
        }
    };
}
macro_rules! trans_serializable_float_impl {
    ($T: ident) => {
        impl TransSerializable for $T {
            fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
                SerializationResult { result: number_from_f(*self as f64).into(), err: None }
            }
        
            fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
                Ok(match value {
                    serde_json::Value::Number(num) => SerializationResult { result: number_to_f(num, $T::MIN as f64, $T::MAX as f64) as $T, err: None },
                    _ => SerializationResult { result: Default::default(), err: Some(SerializationError::InvalidInput) },
                })
            }
        }
    };
}

fn number_to_i(number: &serde_json::Number, min: i128, max: i128) -> i128 {
    if let Some(number) = number.as_u64() {
        (number as i128).clamp(min, max)
    } else if let Some(number) = number.as_i64() {
        (number as i128).clamp(min, max)
    } else if let Some(number) = number.as_f64() {
        if number <= min as f64 {
            min
        } else if number >= max as f64 {
            max
        } else {
            number as i128
        }
    } else {
        0
    }
}

fn number_to_f(number: &serde_json::Number, min: f64, max: f64) -> f64 {
    if let Some(number) = number.as_u64() {
        (number as f64).clamp(min, max)
    } else if let Some(number) = number.as_i64() {
        (number as f64).clamp(min, max)
    } else if let Some(number) = number.as_f64() {
        (number as f64).clamp(min, max)
    } else {
        0.0
    }
}

fn number_from_i(mut number: i128) -> serde_json::Number {
    if number > u64::MAX as i128 {
        number = u64::MAX as i128
    } else if number < i64::MIN as i128 {
        number = u64::MAX as i128
    }

    serde_json::Number::from_i128(number).unwrap()
}

fn number_from_f(mut number: f64) -> serde_json::Number {
    if !number.is_finite() {
        number = 0.0;
    }

    serde_json::Number::from_f64(number).unwrap()
}

trans_serializable_int_impl!(u8);
trans_serializable_int_impl!(i8);
trans_serializable_int_impl!(u16);
trans_serializable_int_impl!(i16);
trans_serializable_int_impl!(u32);
trans_serializable_int_impl!(i32);
trans_serializable_int_impl!(u64);
trans_serializable_int_impl!(i64);
trans_serializable_int_impl!(u128);
trans_serializable_int_impl!(i128);
trans_serializable_float_impl!(f32);
trans_serializable_float_impl!(f64);

impl TransSerializable for char {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
        SerializationResult {
            result: serde_json::Value::String(String::from(*self)),
            err: None,
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
        Ok(match value {
            serde_json::Value::String(string) if string.len() > 1 => SerializationResult { result: string.chars().next().unwrap(), err: None },
            _ => SerializationResult { result: Default::default(), err: Some(SerializationError::InvalidInput) },
        })
    }
}

impl TransSerializable for bool {
    fn serialize(&self, _ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
        SerializationResult {
            result: serde_json::Number::from_u128(if *self { 1 } else { 0 }).unwrap().into(),
            err: None,
        }
    }

    fn deserialize(_ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
        Ok(match value {
            serde_json::Value::String(string) if string.len() > 1 => SerializationResult { result: string.trim().to_lowercase() == "true", err: None },
            serde_json::Value::Number(number) => SerializationResult { result: number_to_i(number, i128::MIN, i128::MAX) != 0, err: None },
            _ => SerializationResult { result: Default::default(), err: Some(SerializationError::InvalidInput) },
        })
    }
}

impl<T: 'static> TransSerializable for Vec<T> {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
        let mut err = None;
        let mut vec = Vec::new();

        for element in self {
            let result = ctx.serialize(element);

            vec.push(result.result);

            if let Some(result_err) = result.err && err.is_none() {
                err = Some(result_err);
            }
        }
        
        SerializationResult { result: serde_json::Value::Array(vec), err }
    }

    fn deserialize(ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
        let values = match value {
            serde_json::Value::Null => &[],
            serde_json::Value::Array(values) => values.as_slice(),
            _ => std::slice::from_ref(value),
        };

        let mut err = None;
        let mut vec = Vec::new();

        for element in values {
            match ctx.deserialize::<T>(element) {
                Ok(result) => {
                    vec.push(result.result);

                    if let Some(result_err) = result.err && err.is_none() {
                        err = Some(result_err);
                    }
                },
                Err(result_err) => {
                    if err.is_none() {
                        err = Some(result_err);
                    }
                },
            };
        };

        Ok(SerializationResult {
            result: vec,
            err: err,
        })
    }
}

impl<T: 'static> TransSerializable for Option<T> {
    fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> {
        match self {
            Some(value) => ctx.serialize(value),
            None => SerializationResult { result: serde_json::Value::Null, err: None },
        }
    }

    fn deserialize(ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> {
        match value {
            serde_json::Value::Null => Ok(SerializationResult { result: None, err: None }),
            _ => ctx.deserialize(value),
        }
    }
}