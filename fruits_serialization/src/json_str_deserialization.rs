use crate::{json_repr::{JsonObject, JsonValue}, JsonNumber};

const NULL_CHARS: &[char] = &['n', 'u', 'l', 'l'];

enum InsideObjectState {
    BetweenFields,
    FieldName(String),
    FieldBeforeColon(String),
    FieldAfterValue,
}

enum InsideArrayState {
    Element,
    Colon,
}

enum State {
    None,
    InsideNull { matching_chars: usize },
    InsideBool { result: bool, compared_chars: &'static [char], matching_chars: usize },
    InsideNumber { buf: String, contains_dot: bool },
    InsideString { buf: String },
    InsideObject(JsonObject, InsideObjectState),
    InsideArray(Vec<JsonValue>, InsideArrayState),
}

// todo: return some info about errors
impl JsonValue {
    pub fn parse(chars: &mut impl Iterator<Item = char>) -> Option<JsonValue> {
        to_json_peekable(&mut chars.peekable())
    }
}

fn to_json_peekable<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<&mut I>) -> Option<JsonValue> {
    let mut state = State::None;

    loop {
        state = 'm: {
            match state {
                State::None => {
                    let Some(c) = chars.next() else {
                        return None;
                    };

                    if c.is_whitespace() {
                        break 'm state;
                    }

                    if c == 'n' {
                        break 'm State::InsideNull { matching_chars: 1 };
                    }

                    if c == 't' {
                        break 'm State::InsideBool { result: true, compared_chars: &['t', 'r', 'u', 'e'], matching_chars: 1 };
                    }

                    if c == 'f' {
                        break 'm State::InsideBool { result: false, compared_chars: &['f', 'a', 'l', 's', 'e'], matching_chars: 1 };
                    }

                    if c.is_ascii_digit() || c == '.' {
                        let mut buf = String::new();

                        buf.push(c);

                        break 'm State::InsideNumber { buf, contains_dot: c == '.' };
                    }

                    if c == '"' {
                        break 'm State::InsideString { buf: String::new() };
                    }

                    if c == '{' {
                        break 'm State::InsideObject(JsonObject::new(), InsideObjectState::BetweenFields);
                    }

                    if c == '[' {
                        break 'm State::InsideArray(Vec::new(), InsideArrayState::Element);
                    }

                    return None;
                },
                State::InsideNull { matching_chars } => {
                    let Some(c) = chars.next() else {
                        return None;
                    };

                    if matching_chars >= (NULL_CHARS.len() - 1) {
                        return Some(JsonValue::Null);
                    }

                    if c == NULL_CHARS[matching_chars] {
                        break 'm State::InsideNull { matching_chars: matching_chars + 1 };
                    }

                    return None;
                },
                State::InsideBool { result, compared_chars, matching_chars } => {
                    let Some(c) = chars.next() else {
                        return None;
                    };

                    if matching_chars >= (NULL_CHARS.len() - 1) {
                        return Some(JsonValue::Bool(result));
                    }
                    
                    if c == compared_chars[matching_chars] {
                        break 'm State::InsideBool { result, compared_chars, matching_chars: matching_chars + 1 };
                    }

                    return None;
                },
                State::InsideNumber { mut buf, mut contains_dot } => {
                    let exit_fn = || {
                        Some(if contains_dot {
                            JsonNumber::F(buf.parse().ok()?).into()
                        } else {
                            JsonNumber::I(buf.parse().ok()?).into()
                        })
                    };

                    let Some(&c) = chars.peek() else {
                        return exit_fn();
                    };

                    if c.is_ascii_digit() {
                        chars.next();
                        buf.push(c);
                    } else if c == '.' && !contains_dot {
                        chars.next();
                        buf.push(c);
                        contains_dot = true;
                    } else {
                        return exit_fn();
                    }

                    break 'm State::InsideNumber { buf, contains_dot };
                },
                State::InsideString { mut buf } => {
                    let Some(c) = chars.next() else {
                        return None;
                    };
                    
                    if c == '"' {
                        return Some(JsonValue::String(buf));
                    }

                    buf.push(c);
                    break 'm State::InsideString { buf };
                },
                State::InsideObject(mut json_object, inside_object_state) => {
                    match inside_object_state {
                        InsideObjectState::BetweenFields => {
                            let Some(c) = chars.next() else {
                                return None;
                            };
                            
                            if c.is_whitespace() {
                                break 'm State::InsideObject(json_object, inside_object_state);
                            }

                            if c == '}' {
                                return Some(JsonValue::Object(JsonObject::new()));
                            }

                            if c == '"' {
                                break 'm State::InsideObject(json_object, InsideObjectState::FieldName(String::new()));
                            }

                            return None;
                        },
                        InsideObjectState::FieldName(mut name) => {
                            let Some(c) = chars.next() else {
                                return None;
                            };
                            
                            if c == '"' {
                                break 'm State::InsideObject(json_object, InsideObjectState::FieldBeforeColon(name));
                            }

                            name.push(c);

                            break 'm State::InsideObject(json_object, InsideObjectState::FieldName(name));
                        },
                        InsideObjectState::FieldBeforeColon(name) => {
                            let Some(c) = chars.next() else {
                                return None;
                            };
                            
                            if c.is_whitespace() {
                                break 'm State::InsideObject(json_object, InsideObjectState::FieldBeforeColon(name));
                            }

                            if c == ':' {
                                // todo: remove recursion.
                                json_object.push_field(name, to_json_peekable(chars)?).ok()?;
                                break 'm State::InsideObject(json_object, InsideObjectState::FieldAfterValue);
                            }

                            return None;
                        },
                        InsideObjectState::FieldAfterValue => {
                            let Some(c) = chars.next() else {
                                return None;
                            };
                            
                            if c.is_whitespace() {
                                break 'm State::InsideObject(json_object, inside_object_state);
                            }

                            if c == ',' {
                                break 'm State::InsideObject(json_object, InsideObjectState::BetweenFields);
                            }

                            if c == '}' {
                                return Some(json_object.into());
                            }

                            return None;
                        },
                    }
                },
                State::InsideArray(mut json_array, inside_array_state) => {
                    match inside_array_state {
                        InsideArrayState::Element => {
                            let Some(&nc) = chars.peek() else {
                                return None;
                            };

                            if nc == ']' {
                                chars.next();
                                return Some(JsonValue::Array(json_array));
                            }
                            // todo: remove recursion.
                            json_array.push(to_json_peekable(chars)?);
                            break 'm State::InsideArray(json_array, InsideArrayState::Colon);
                        },
                        InsideArrayState::Colon => {
                            let Some(c) = chars.next() else {
                                return None;
                            };

                            if c.is_whitespace() {
                                break 'm State::InsideArray(json_array, inside_array_state)
                            }

                            if c == ',' {
                                break 'm State::InsideArray(json_array, InsideArrayState::Element);
                            }

                            if c == ']' {
                                return Some(JsonValue::Array(json_array));
                            }

                            return None;
                        },
                    }
                }
            }
        }
    }
}