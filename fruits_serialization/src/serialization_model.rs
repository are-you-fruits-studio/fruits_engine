use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub enum SerializedValue {
    Null,
    Primitive(SerializedPrimitive),
    Composite(SerializedComposite),
}

#[derive(Clone, Debug)]
pub enum SerializedPrimitive {
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
}

#[derive(Clone, Debug)]
pub struct SerializedComposite {
    pub values: SerializedCompositeValues,
    pub is_rigid: bool,
}

#[derive(Clone, Debug)]
pub struct SerializedEnumMetadata {
    pub variant: String,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum SerializedCompositeValues {
    Map(SerializedMap),
    List(Vec<SerializedValue>),
}

#[derive(Clone, Debug, Default)]
pub struct SerializedMap {
    pub values: IndexMap<String, SerializedValue>,
    pub enum_metadata: Option<SerializedEnumMetadata>,
}

impl SerializedValue {
    pub fn to_json(&self) -> serde_json::Value {
        let value = self;

        match value {
            Self::Null => serde_json::Value::Null,
            Self::Primitive(value) => {
                match value {
                    SerializedPrimitive::Bool(value) => serde_json::Value::Bool(*value),
                    SerializedPrimitive::Int(value) => serde_json::Number::from_i128((*value).clamp(i64::MIN as i128, u64::MAX as i128)).map(serde_json::Value::Number).unwrap_or_else(|| serde_json::Value::Null),
                    SerializedPrimitive::Float(value) => serde_json::Number::from_f64(*value).map(serde_json::Value::Number).unwrap_or_else(|| serde_json::Value::Null),
                    SerializedPrimitive::String(value) => serde_json::Value::String(value.clone()),
                }
            },
            Self::Composite(value) => {
                match &value.values {
                    SerializedCompositeValues::List(serialized_values) => {
                        serde_json::Value::Array(serialized_values.iter().map(|v| v.to_json()).collect())
                    },
                    SerializedCompositeValues::Map(serialized_map) => {
                        let mut map = serde_json::Map::new();

                        if let Some(serialized_enum_metadata) = &serialized_map.enum_metadata {
                            map.insert(String::from("$enum_variant"), serde_json::Value::String(serialized_enum_metadata.variant.clone()));
                        }

                        for (key, serialized_value) in &serialized_map.values {
                            if key.as_str() == "$enum_variant" {
                                eprintln!("Reserverd name \"$enum_variant\" is used in SerializedValue, skipped");
                                continue;
                            }

                            map.insert(key.clone(), serialized_value.to_json());
                        }

                        serde_json::Value::Object(map)
                    },
                }
            },
        }
    }

    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Primitive(SerializedPrimitive::Bool(*value)),
            serde_json::Value::Number(value) => {
                Self::Primitive(
                    if value.is_u64() {
                        SerializedPrimitive::Int(value.as_u64().unwrap() as i128)
                    } else if value.is_i64() {
                        SerializedPrimitive::Int(value.as_i64().unwrap() as i128)
                    } else {
                        SerializedPrimitive::Float(value.as_f64().unwrap())
                    }
                )
            },
            serde_json::Value::String(value) => Self::Primitive(SerializedPrimitive::String(value.clone())),
            serde_json::Value::Array(value) => {
                Self::Composite(SerializedComposite {
                    is_rigid: false,
                    values: SerializedCompositeValues::List(value.iter().map(Self::from_json).collect())
                })
            },
            serde_json::Value::Object(value) => {
                let mut enum_variant = None;
                let mut map = IndexMap::new();

                for (key, json_value) in value {
                    if key.as_str() == "$enum_variant" {
                        if let serde_json::Value::String(json_value) = json_value {
                            enum_variant = Some(json_value.clone());
                        } else {
                            eprintln!("Tried to deserialize invalid \"$enum_variant\"");
                        }

                        continue;
                    }

                    map.insert(key.clone(), SerializedValue::from_json(json_value));
                }

                Self::Composite(SerializedComposite {
                    values: SerializedCompositeValues::Map(SerializedMap {
                        values: map,
                        enum_metadata: enum_variant.map(|v| SerializedEnumMetadata {
                            variant: v,
                            variants: Vec::new(),
                        })
                    }),
                    is_rigid: false,
                })
            },
        }
    }
}
