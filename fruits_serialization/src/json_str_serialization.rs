use std::fmt::{Display, Write};

// todo: automatic escape symbols
// todo: support exp and other numeric formats

use crate::*;

#[derive(Copy, Clone)]
pub struct JsonIndentState {
    pub size: usize,
    pub count: usize,
}

impl JsonIndentState {
    pub fn root(size: usize) -> Self{
        Self {
            count: 0,
            size,
        }
    }
}

impl Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f, f.precision())
    }
}

impl JsonValue {
    pub fn write(&self, w: &mut impl Write, indent_size: Option<usize>) -> Result<(), std::fmt::Error> {
        write_preindented(self, w, indent_size.map(JsonIndentState::root))
    }
}

pub fn write(v: &JsonValue, w: &mut impl Write, indent_size: Option<usize>) -> Result<(), std::fmt::Error> {
    write_preindented(v, w, indent_size.map(JsonIndentState::root))
}

fn write_preindented(v: &JsonValue, w: &mut impl Write, indent: Option<JsonIndentState>) -> Result<(), std::fmt::Error> {
    Ok(match v {
        JsonValue::Null => write!(w, "null")?,
        JsonValue::Bool(bool) => write!(w, "{bool}")?,
        JsonValue::Number(number) => write!(w, "{number}")?,
        JsonValue::String(string) => write!(w, "\"{string}\"")?,
        JsonValue::Array(json_array) => {
            if json_array.is_empty() {
                return write!(w, "[]");
            }

            let mut new_indent = None;

            if let Some(i) = indent {
                new_indent = Some(JsonIndentState { count: i.count + 1, ..i })
            }
            write!(w, "[")?;

            let mut is_first = true;

            for ele in json_array {
                if !std::mem::replace(&mut is_first, false) {
                    write!(w, ",")?;
                }
                try_write_new_line_and_indents(w, new_indent)?;
                // todo: remove recursion
                write_preindented(ele, w, new_indent)?;
            }
            try_write_new_line_and_indents(w, indent)?;
            write!(w, "]")?;
        },
        JsonValue::Object(json_object) => {
            if json_object.field_names().len() == 0 {
                return write!(w, "{{}}");
            }

            let mut new_indent = None;

            if let Some(i) = indent {
                new_indent = Some(JsonIndentState { count: i.count + 1, ..i })
            }
            write!(w, "{{")?;

            let mut is_first = true;

            for (field_name, field_value) in json_object.fields() {
                if !std::mem::replace(&mut is_first, false) {
                    write!(w, ",")?;
                }
                try_write_new_line_and_indents(w, new_indent)?;
                write!(w, "\"{field_name}\":")?;
                if let Some(_) = new_indent {
                    write!(w, " ")?;
                }
                // todo: remove recursion
                write_preindented(field_value, w, new_indent)?;
            }
            try_write_new_line_and_indents(w, indent)?;
            write!(w, "}}")?;
        },
    })
}

fn try_write_new_line_and_indents(w: &mut impl Write, indent: Option<JsonIndentState>) -> Result<(), std::fmt::Error> {
    if let Some(i) = indent {
        writeln!(w)?;

        for _ in 0..(i.count * i.size) {
            w.write_char(' ')?;
        }
    }

    Ok(())
}

impl std::fmt::Debug for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(self, f, None)
    }
}

impl std::fmt::Debug for crate::json_repr::JsonObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(&self.clone().into(), f, None)
    }
}