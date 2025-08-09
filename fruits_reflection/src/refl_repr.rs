use std::fmt::Debug;

pub enum ReflRepr {
    Struct(ReflReprStruct),
    Enum(ReflReprEnum),
    Primitive(ReflReprPrimitive),
}

pub enum ReflReprPrimitive {
    Int(i128),
    Float(f64),
    Char(char),
    Str(String),
    Bool(bool),
    Unit,
}

pub struct ReflReprStruct {
    pub name: String,
    pub fields: ReflReprFields,
}

pub enum ReflReprFields {
    Unit,
    Tuple(Vec<ReflRepr>),
    Named(Vec<(String, ReflRepr)>),
}

pub struct ReflReprEnum {
    pub name: String,
    pub variant: String,
    pub fields: ReflReprFields,
}

impl Debug for ReflRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReflRepr::Struct(repr) => {
                write!(f, "{} {:?}", &repr.name, &repr.fields)?;
            },
            ReflRepr::Enum(repr) => {
                write!(f, "{}::{} {:?}", &repr.name, &repr.variant, &repr.fields)?;
            },
            ReflRepr::Primitive(repr) => {
                match repr {
                    ReflReprPrimitive::Int(repr) => write!(f, "{}", repr).unwrap(),
                    ReflReprPrimitive::Float(repr) => write!(f, "{}", repr).unwrap(),
                    ReflReprPrimitive::Char(repr) => write!(f, "'{}'", repr).unwrap(),
                    ReflReprPrimitive::Str(repr) => write!(f, "\"{}\"", repr).unwrap(),
                    ReflReprPrimitive::Bool(repr) => write!(f, "{}", repr).unwrap(),
                    ReflReprPrimitive::Unit => write!(f, "()").unwrap(),
                }
            },
        }

        Ok(())
    }
}

impl Debug for ReflReprFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReflReprFields::Unit => (),
            ReflReprFields::Tuple(fields) => {
                write!(f, "(")?;
                for (i, field_repr) in fields.iter().enumerate() {
                    write!(f, "{:?}", field_repr)?;
                    if i < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
            },
            ReflReprFields::Named(fields) => {
                write!(f, "{{ ")?;
                for (i, (field_name, field_repr)) in fields.iter().enumerate() {
                    write!(f, "{}: {:?}", field_name, field_repr)?;
                    if i < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, " }}")?;
            },
        }

        Ok(())
    }
}

// todo: generics?
// todo: collections

// example

struct ExampleStruct {
    pub age: u8,
    pub name: String,
}

fn example_struct_into_refl_repr(v: &ExampleStruct) -> ReflRepr {
    ReflRepr::Struct(ReflReprStruct {
        name: String::from("ExampleStruct"),
        fields: ReflReprFields::Named([
            (String::from("age"), ReflRepr::Primitive(ReflReprPrimitive::Int(v.age as i128))),
            (String::from("age"), ReflRepr::Primitive(ReflReprPrimitive::Str(v.name.clone()))),
        ].into_iter().collect()),
    })
}

fn example_struct_from_refl_repr(v: &ReflRepr) -> Option<ExampleStruct> {
    let ReflRepr::Struct(v) = v else {
        return None;
    };

    let ReflReprFields::Named(v) = &v.fields else {
        return None;
    };

    let ReflRepr::Primitive(ReflReprPrimitive::Int(age)) = &v.iter().find(|(n, _)| n == "age")?.1 else {
        return None;
    };

    let ReflRepr::Primitive(ReflReprPrimitive::Str(name)) = &v.iter().find(|(n, _)| n == "name")?.1 else {
        return None;
    };

    Some(ExampleStruct {
        age: (*age).try_into().ok()?,
        name: name.clone(),
    })
}