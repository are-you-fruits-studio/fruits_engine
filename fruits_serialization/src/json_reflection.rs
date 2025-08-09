use std::collections::HashMap;

#[derive(Clone, Default)]
pub enum JsonValue {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonObject),
}

#[derive(Copy, Clone, Default)]
pub enum JsonIndentation {
    #[default]
    None,
    Indented { indent_level: usize }
}

pub struct JsonField {
    pub name: String,
    pub value: JsonValue,
}

impl JsonField {
    pub fn new(name: String, value: JsonValue) -> Self {
        Self {
            name,
            value,
        }
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

impl Into<JsonValue> for bool { fn into(self) -> JsonValue { JsonValue::Bool(self) } }

impl Into<JsonValue> for i8 { fn into(self) -> JsonValue { JsonValue::Int(self as i64) } }
impl Into<JsonValue> for u8 { fn into(self) -> JsonValue { JsonValue::Int(self as i64) } }
impl Into<JsonValue> for i16 { fn into(self) -> JsonValue { JsonValue::Int(self as i64) } }
impl Into<JsonValue> for u16 { fn into(self) -> JsonValue { JsonValue::Int(self as i64) } }
impl Into<JsonValue> for i32 { fn into(self) -> JsonValue { JsonValue::Int(self as i64) } }
impl Into<JsonValue> for u32 { fn into(self) -> JsonValue { JsonValue::Int(self as i64) } }
impl Into<JsonValue> for i64 { fn into(self) -> JsonValue { JsonValue::Int(self) } }

impl Into<JsonValue> for f32 { fn into(self) -> JsonValue { JsonValue::Float(self as f64) } }
impl Into<JsonValue> for f64 { fn into(self) -> JsonValue { JsonValue::Float(self) } }

impl Into<JsonValue> for String { fn into(self) -> JsonValue { JsonValue::String(self) } }

impl Into<JsonValue> for JsonObject { fn into(self) -> JsonValue { JsonValue::Object(self) } }

impl Into<JsonValue> for Vec<JsonValue> { fn into(self) -> JsonValue { JsonValue::Array(self) } }
