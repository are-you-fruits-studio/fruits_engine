use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

#[derive(Copy, Clone)]
pub enum JsonNumber {
    I(i128),
    F(f64),
}

impl JsonNumber {
    pub const fn to_i(self) -> i128 {
        match self {
            JsonNumber::I(v) => v,
            JsonNumber::F(v) => v as i128,
        }
    }

    pub const fn to_f(self) -> f64 {
        match self {
            JsonNumber::I(v) => v as f64,
            JsonNumber::F(v) => v,
        }
    }
}

macro_rules! into_number_repr_impl {
    ($real: ident, $variant: ident, $convert_fn: ident, $casted: ident) => {
        impl Into<JsonNumber> for $real {
            fn into(self) -> JsonNumber {
                JsonNumber::$variant(self as $casted)
            }
        }
        impl Into<$real> for JsonNumber {
            fn into(self) -> $real {
                self.$convert_fn() as $real
            }
        }
    };
}

into_number_repr_impl! { u8, I, to_i, i128 }
into_number_repr_impl! { i8, I, to_i, i128 }
into_number_repr_impl! { u16, I, to_i, i128 }
into_number_repr_impl! { i16, I, to_i, i128 }
into_number_repr_impl! { u32, I, to_i, i128 }
into_number_repr_impl! { i32, I, to_i, i128 }
into_number_repr_impl! { u64, I, to_i, i128 }
into_number_repr_impl! { i64, I, to_i, i128 }
into_number_repr_impl! { i128, I, to_i, i128 }
into_number_repr_impl! { usize, I, to_i, i128 }
into_number_repr_impl! { isize, I, to_i, i128 }
into_number_repr_impl! { f32, F, to_f, f64 }
into_number_repr_impl! { f64, F, to_f, f64 }

impl Debug for JsonNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I(v) => Debug::fmt(v, f),
            Self::F(v) => Debug::fmt(v, f),
        }
    }
}

impl Display for JsonNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I(v) => Debug::fmt(v, f),
            Self::F(v) => Debug::fmt(v, f),
        }
    }
}

#[derive(Clone, Default)]
pub enum JsonValue {
    #[default]
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonObject),
}

#[derive(Copy, Clone, Default)]
pub enum JsonIndentation {
    #[default]
    None,
    Indented {
        indent_level: usize,
    },
}

pub struct JsonField {
    pub name: String,
    pub value: JsonValue,
}

impl JsonField {
    pub fn new(name: String, value: JsonValue) -> Self {
        Self { name, value }
    }
}

#[derive(Clone, Default)]
pub struct JsonObject {
    fields: HashMap<String, JsonValue>,
    field_names: Vec<String>,
}

impl JsonObject {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            field_names: Vec::new(),
        }
    }

    pub fn push_field(&mut self, name: impl Into<String>, value: impl Into<JsonValue>) -> Result<(), JsonField> {
        let field = JsonField::new(name.into(), value.into());

        if self.fields.contains_key(&field.name) {
            return Err(field);
        }

        self.fields.insert(field.name.clone(), field.value);
        self.field_names.push(field.name);

        Ok(())
    }

    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<JsonValue>) -> Result<Self, (Self, JsonField)> {
        match self.push_field(name, value) {
            Ok(_) => Ok(self),
            Err(field) => Err((self, field)),
        }
    }

    pub fn field_names(&self) -> &[String] {
        &self.field_names
    }

    pub fn get_value(&self, name: &str) -> Option<&JsonValue> {
        self.fields.get(name)
    }

    pub fn into_fields(mut self) -> impl Iterator<Item = JsonField> {
        self.field_names.into_iter().map(move |name| {
            let value = self.fields.remove(&name).unwrap();
            JsonField::new(name, value)
        })
    }

    pub fn fields(&self) -> impl Iterator<Item = (&String, &JsonValue)> {
        self.field_names.iter().map(|name| {
            let value = &self.fields[name];
            (name, value)
        })
    }
}

impl Into<JsonValue> for bool {
    fn into(self) -> JsonValue {
        JsonValue::Bool(self)
    }
}
impl Into<JsonValue> for JsonNumber {
    fn into(self) -> JsonValue {
        JsonValue::Number(self)
    }
}

impl Into<JsonValue> for u8 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for i8 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for u16 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for i16 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for u32 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for i32 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for u64 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for i64 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for i128 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for usize {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for isize {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for f32 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}
impl Into<JsonValue> for f64 {
    fn into(self) -> JsonValue {
        <Self as Into<JsonNumber>>::into(self).into()
    }
}

impl Into<JsonValue> for String {
    fn into(self) -> JsonValue {
        JsonValue::String(self)
    }
}

impl Into<JsonValue> for JsonObject {
    fn into(self) -> JsonValue {
        JsonValue::Object(self)
    }
}

impl Into<JsonValue> for Vec<JsonValue> {
    fn into(self) -> JsonValue {
        JsonValue::Array(self)
    }
}
