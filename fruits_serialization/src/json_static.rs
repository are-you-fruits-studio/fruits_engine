use std::{any::{Any, TypeId}, collections::{BTreeMap, HashMap, HashSet}, hash::Hash};

use crate::{JsonObject, JsonValue};

pub trait JsonSerializable: Sized {
    fn to_json(&self) -> JsonValue;
    fn from_json(json: &JsonValue) -> Option<Self>;
    fn fill_partially_from_json(&mut self, json: &JsonValue);
}

macro_rules! json_serializable_impl_int {
    ($($t: ident),+) => {
        $(
            impl JsonSerializable for $t {
                fn to_json(&self) -> JsonValue {
                    JsonValue::Number((*self).into())
                }
            
                fn from_json(json: &JsonValue) -> Option<Self> {
                    match json {
                        JsonValue::Number(v) => v.to_i().try_into().ok(),
                        JsonValue::String(v) => v.parse().ok(),
                        _ => None,
                    }
                }
                
                fn fill_partially_from_json(&mut self, json: &JsonValue) {
                    if let Some(parsed) = Self::from_json(json) {
                        *self = parsed;
                    }
                }
            }
        )+
    };
}

macro_rules! json_serializable_impl_float {
    ($($t: ident),+) => {
        $(
            impl JsonSerializable for $t {
                fn to_json(&self) -> JsonValue {
                    JsonValue::Number((*self).into())
                }
            
                fn from_json(json: &JsonValue) -> Option<Self> {
                    match json {
                        JsonValue::Number(v) => Some(v.to_f() as _),
                        JsonValue::String(v) => v.parse().ok(),
                        _ => None,
                    }
                }
                
                fn fill_partially_from_json(&mut self, json: &JsonValue) {
                    if let Some(parsed) = Self::from_json(json) {
                        *self = parsed;
                    }
                }
            }
        )+
    };
}

json_serializable_impl_int!(u8, i8, u16, i16, u32, i32, u64, i64, i128, usize, isize);
json_serializable_impl_float!(f32, f64);

impl JsonSerializable for bool {
    fn to_json(&self) -> JsonValue {
        JsonValue::Bool(*self)
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        match json {
            JsonValue::Bool(v) => Some(*v),
            JsonValue::String(v) => v.parse().ok(),
            _ => None,
        }
    }
                
    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        if let Some(parsed) = Self::from_json(json) {
            *self = parsed;
        }
    }
}

impl JsonSerializable for char {
    fn to_json(&self) -> JsonValue {
        JsonValue::String(self.to_string())
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::String(v) = json else {
            return None;
        };

        let mut chars = v.chars();

        let v = chars.next()?;

        if chars.next().is_some() {
            return None;
        }
    
        Some(v)
    }
                
    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        if let Some(parsed) = Self::from_json(json) {
            *self = parsed;
        }
    }
}

impl JsonSerializable for String {
    fn to_json(&self) -> JsonValue {
        JsonValue::String(self.clone())
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::String(v) = json else {
            return None;
        };

        Some(v.clone())
    }

    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        if let Some(parsed) = Self::from_json(json) {
            *self = parsed;
        }
    }
}

impl JsonSerializable for () {
    fn to_json(&self) -> JsonValue {
        JsonValue::Object(JsonObject::new())
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::Object(_) = json else {
            return None;
        };

        Some(())
    }
                
    fn fill_partially_from_json(&mut self, json: &JsonValue) { }
}

impl<T: JsonSerializable> JsonSerializable for Option<T> {
    fn to_json(&self) -> JsonValue {
        match self {
            Some(v) => v.to_json(),
            None => JsonValue::Null,
        }
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        match json {
            JsonValue::Null => None,
            _ => Some(T::from_json(json)),
        }
    }
                
    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        match self {
            Some(this) => {
                match json {
                    JsonValue::Null => *self = None,
                    _ => this.fill_partially_from_json(json),
                }
            },
            None => {
                if let Some(parsed) = Self::from_json(json) {
                    *self = parsed;
                }
            },
        }
    }
}

impl<T: JsonSerializable> JsonSerializable for Vec<T> {
    fn to_json(&self) -> JsonValue {
        let mut vec = Vec::with_capacity(self.len());

        for item in self {
            vec.push(item.to_json());
        }

        JsonValue::Array(vec)
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::Array(v) = json else {
            return None;
        };

        let mut vec = Vec::with_capacity(v.len());

        for item in v {
            vec.push(T::from_json(item)?);
        }

        Some(vec)
    }
                
    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        let JsonValue::Array(v) = json else {
            return;
        };

        self.clear();

        self.extend(v.iter().filter_map(|i| T::from_json(i)));
    }
}

impl<K: 'static + JsonSerializable + Eq + Hash, V: JsonSerializable> JsonSerializable for HashMap<K, V> {
    fn to_json(&self) -> JsonValue {
        let mut json = JsonObject::new();

        for (key, val) in self {
            let key_json = key.to_json();

            let key_str = match key_json {
                JsonValue::Null => String::from("null"),
                JsonValue::Bool(v) => v.to_string(),
                JsonValue::Number(v) => v.to_string(),
                JsonValue::String(v) => v.clone(),
                _ => String::new(),
            };

            json.push_field(key_str, val.to_json()).ok().unwrap();
        }

        json.into()
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::Object(v) = json else {
            return None;
        };

        let mut map = Self::new();

        for (key, val) in v.fields() {
            let key_json = JsonValue::parse(&mut std::iter::once('"').chain(key.chars()).chain(std::iter::once('"')))?;
            map.insert(K::from_json(&key_json)?, V::from_json(val)?);
        }

        Some(map)
    }

    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        let JsonValue::Object(v) = json else {
            return;
        };

        self.clear();

        for (key, val) in v.fields() {
            let Some(key_json) = JsonValue::parse(&mut std::iter::once('"').chain(key.chars()).chain(std::iter::once('"'))) else { continue; };
            let Some(key) = K::from_json(&key_json) else { continue; };
            let Some(val) = V::from_json(val) else { continue; };

            self.insert(key, val);
        }
    }
}

impl<K: 'static + JsonSerializable + Ord, V: JsonSerializable> JsonSerializable for BTreeMap<K, V> {
    fn to_json(&self) -> JsonValue {
        let mut json = JsonObject::new();

        for (key, val) in self {
            let key_json = key.to_json();

            let key_str = match key_json {
                JsonValue::Null => String::from("null"),
                JsonValue::Bool(v) => v.to_string(),
                JsonValue::Number(v) => v.to_string(),
                JsonValue::String(v) => v.clone(),
                _ => String::new(),
            };

            json.push_field(key_str, val.to_json()).ok().unwrap();
        }

        json.into()
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::Object(v) = json else {
            return None;
        };

        let mut map = Self::new();

        for (key, val) in v.fields() {
            let key_json = JsonValue::parse(&mut std::iter::once('"').chain(key.chars()).chain(std::iter::once('"')))?;
            map.insert(K::from_json(&key_json)?, V::from_json(val)?);
        }

        Some(map)
    }

    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        let JsonValue::Object(v) = json else {
            return;
        };

        self.clear();

        for (key, val) in v.fields() {
            let Some(key_json) = JsonValue::parse(&mut std::iter::once('"').chain(key.chars()).chain(std::iter::once('"'))) else { continue; };
            let Some(key) = K::from_json(&key_json) else { continue; };
            let Some(val) = V::from_json(val) else { continue; };

            self.insert(key, val);
        }
    }
}

impl<T: 'static + JsonSerializable + Eq + Hash> JsonSerializable for HashSet<T> {
    fn to_json(&self) -> JsonValue {
        let mut vec = Vec::with_capacity(self.len());

        for item in self {
            vec.push(item.to_json());
        }

        JsonValue::Array(vec)
    }

    fn from_json(json: &JsonValue) -> Option<Self> {
        let JsonValue::Array(v) = json else {
            return None;
        };

        let mut set = HashSet::new();

        for item in v {
            set.insert(T::from_json(item)?);
        }

        Some(set)
    }
                
    fn fill_partially_from_json(&mut self, json: &JsonValue) {
        let JsonValue::Array(v) = json else {
            return;
        };

        self.clear();

        self.extend(v.iter().filter_map(|i| T::from_json(i)));
    }
}