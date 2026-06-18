//! # fruits_json
//!
//! A self-contained JSON representation, parser, and serializer for the engine,
//! with derive support for turning your own types into JSON and back.
//!
//! # How to use
//!
//! #### Derive serialization for your own types
//!
//! Add `#[derive(JsonSerializable)]` to a struct or a fieldless enum to get
//! conversion to and from a [`JsonValue`]. The derive needs [`JsonValue`],
//! [`JsonObject`], and the [`JsonSerializable`](trait@JsonSerializable) trait in
//! scope.
//!
//! ```
//! use fruits_json::{JsonObject, JsonSerializable, JsonValue};
//!
//! #[derive(JsonSerializable)]
//! struct Player {
//!     name: String,
//!     score: u32,
//! }
//!
//! let player = Player { name: String::from("Mei"), score: 42 };
//!
//! // Convert to a JSON value, render it as text, then read it back.
//! let text = player.to_json().to_string();
//! let value = JsonValue::parse(&mut text.chars()).unwrap();
//! let restored = Player::from_json(&value).unwrap();
//!
//! assert_eq!(restored.name, "Mei");
//! assert_eq!(restored.score, 42);
//! ```
//!
//! #### Parse JSON text into a value
//!
//! [`JsonValue::parse`] reads a value from any `char` iterator.
//!
//! ```
//! use fruits_json::JsonValue;
//!
//! let value = JsonValue::parse(&mut r#"{"a":1,"b":[2,3]}"#.chars()).unwrap();
//! assert_eq!(value.to_string(), r#"{"a":1,"b":[2,3]}"#);
//! ```
//!
//! #### Render a value as compact or indented text
//!
//! `Display` writes compact JSON by default; passing a precision selects the
//! indentation width for a pretty-printed form.
//!
//! ```
//! use fruits_json::JsonValue;
//!
//! let value = JsonValue::parse(&mut r#"{"a":1,"b":2}"#.chars()).unwrap();
//!
//! let compact = format!("{value}");
//! assert_eq!(compact, r#"{"a":1,"b":2}"#);
//!
//! let pretty = format!("{value:.2}"); // 2-space indentation
//! assert!(pretty.contains("\n  \"a\": 1"));
//! ```
//!
//! #### Build a JSON value by hand
//!
//! [`JsonObject`] keeps fields in insertion order; anything implementing
//! `Into<JsonValue>` (numbers, `bool`, `String`, ...) can be pushed as a value.
//!
//! ```
//! use fruits_json::{JsonObject, JsonValue};
//!
//! let mut obj = JsonObject::new();
//! obj.push_field("hp", 100u32).ok().unwrap();
//! obj.push_field("name", String::from("torch")).ok().unwrap();
//!
//! let value: JsonValue = obj.into();
//! assert_eq!(value.to_string(), r#"{"hp":100,"name":"torch"}"#);
//! ```
//!
//! # How to maintain
//!
//! [`JsonValue`] is the in-memory JSON tree. Numbers are unified by
//! [`JsonNumber`], which stores either an `i128` or an `f64` and converts
//! between them on demand, so integer and floating-point JSON numbers share one
//! variant.
//!
//! [`JsonObject`] deliberately preserves field insertion order. It pairs a
//! `HashMap` for value lookup with a `field_names` vector that records order;
//! [`JsonObject::fields`] and [`JsonObject::into_fields`] iterate in that order.
//! Ordered output keeps rendered JSON stable and diff-friendly. `push_field`
//! rejects duplicate names by returning the rejected field rather than
//! overwriting.
//!
//! Parsing (`json_str_deserialization`) is a hand-written state machine: a
//! `State` enum is advanced one `char` at a time over a `Peekable` iterator,
//! with [`JsonValue::parse`] as the entry point. Nested arrays and objects
//! currently recurse, and failures are reported only as `None` — there is no
//! error position or diagnostic yet. The number parser handles plain integers
//! and a single decimal point; exponent/scientific forms are not supported, and
//! string parsing does not yet decode escape sequences (see the `// todo`s).
//!
//! Serialization to text (`json_str_serialization`) walks the tree and writes
//! either compact output or, when an indent size is supplied, a newline- and
//! space-indented form. The `Display` impl threads the formatter's precision
//! through as that indent size, which is why `{:.2}` pretty-prints. Like the
//! parser, the writer does not yet escape special characters or emit exponent
//! number formats.
//!
//! The [`JsonSerializable`](trait@JsonSerializable) trait (`json_static`) is the
//! static conversion layer. It is implemented for primitives, `String`, `char`,
//! `()`, `Option`, `Vec`, `HashMap`, `BTreeMap`, and `HashSet`; map keys are
//! encoded by rendering the key's own JSON to a string, and `HashMap`/`HashSet`
//! output is sorted so it is deterministic. `fill_partially_from_json` merges
//! incoming fields into an existing value instead of replacing it wholesale.
//! The derive macro lives in `fruits_json_macros` and generates these three
//! methods; it supports field structs (encoded as objects) and fieldless enums
//! (encoded as their variant name string), and panics on enums with fields,
//! unions, or lifetime parameters.
//!
//! A second, dynamic serialization subsystem (`json_map` and
//! `json_map_terminal`) provides type-tagged virtual (de)serialization through a
//! `SerializerRegistry`, writing a `$type` field so values can be deserialized
//! without knowing the concrete type up front. It is still in progress and is
//! **not** re-exported (see the `// todo` on the `pub use` below), so it is not
//! part of the public API.

mod json_map;
mod json_map_terminal;
mod json_repr;
mod json_static;
mod json_str_deserialization;
mod json_str_serialization;

// todo
pub use {/*json_map::*, json_map_terminal::*, */json_repr::*, json_static::*, json_str_serialization::*};

pub use fruits_json_macros::*;
