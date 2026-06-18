//! # fruits_serialization
//!
//! A format-agnostic serialization framework: it converts Rust values to and from an
//! in-memory value tree that a concrete format (currently JSON) can be projected onto,
//! with the serializer for each type chosen at runtime rather than fixed at compile time.
//! The engine uses it to persist and load components, prefabs, and other world data.
//!
//! # How to use
//!
//! The unit of data is [`SerializedValue`], an in-memory tree of nulls, primitives, and
//! composites. A type becomes serializable by implementing [`TransSerializable`] (usually
//! through the [`derive macro`](derive@TransSerializable)), and you serialize by registering
//! that type on a [`GlobalSerializer`] and calling its methods. Inside the engine the
//! registry already lives in the world as `SerializersResource` (a newtype over
//! [`GlobalSerializer`] inserted by the engine's default-modules setup); construct your own
//! [`GlobalSerializer`] only outside that context.
//!
//! #### Round-trip a value with the derive macro
//!
//! Derive [`TransSerializable`] on a type, register it together with every type it contains,
//! then serialize and deserialize through a [`GlobalSerializer`]:
//!
//! ```
//! use fruits_serialization::*;
//!
//! #[derive(TransSerializable, PartialEq, Debug)]
//! struct Player {
//!     name: String,
//!     score: u32,
//! }
//!
//! let mut serializer = GlobalSerializer::new();
//! // every type that appears in the data must have a serializer registered
//! serializer.register(StandardTransSerializer::<String>::default());
//! serializer.register(StandardTransSerializer::<u32>::default());
//! serializer.register(StandardTransSerializer::<Player>::default());
//!
//! let player = Player { name: String::from("Ada"), score: 42 };
//!
//! let serialized = serializer.serialize(&player, None).unwrap();
//! let restored: Option<Player> = serializer.deserialize(&serialized, None).unwrap();
//!
//! assert_eq!(restored, Some(player));
//! ```
//!
//! #### Project to and from JSON
//!
//! [`SerializedValue::to_json`] renders the value tree as a [`serde_json::Value`], and
//! [`SerializedValue::from_json`] reads one back:
//!
//! ```
//! use fruits_serialization::*;
//!
//! let value = SerializedValue::Primitive(SerializedPrimitive::Int(42));
//!
//! let json = value.to_json();
//! assert_eq!(json.to_string(), "42");
//!
//! let restored = SerializedValue::from_json(&json);
//! assert_eq!(restored.to_json().to_string(), "42");
//! ```
//!
//! #### Implement a serializer by hand
//!
//! When the derive does not fit, implement [`TransSerializable`] directly. Build a composite
//! with [`SerializerCtx::serialize_map`] and read it back with [`SerializerCtx::deserialize_map`];
//! nested fields are (de)serialized through the context, so their types must be registered too:
//!
//! ```
//! use fruits_serialization::*;
//!
//! #[derive(PartialEq, Debug)]
//! struct Point { x: f32, y: f32 }
//!
//! impl TransSerializable for Point {
//!     fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> {
//!         ctx.serialize_map()
//!             .with_field("x", &self.x)
//!             .with_field("y", &self.y)
//!             .finish_as_map(true)
//!     }
//!
//!     fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> {
//!         ctx.deserialize_map(value, |ctx| Some(Self {
//!             x: ctx.get_field("x")?,
//!             y: ctx.get_field("y")?,
//!         }))
//!     }
//! }
//!
//! let mut serializer = GlobalSerializer::new();
//! serializer.register(StandardTransSerializer::<f32>::default());
//! serializer.register(StandardTransSerializer::<Point>::default());
//!
//! let point = Point { x: 1.0, y: 2.0 };
//! let serialized = serializer.serialize(&point, None).unwrap();
//! let restored: Option<Point> = serializer.deserialize(&serialized, None).unwrap();
//!
//! assert_eq!(restored, Some(point));
//! ```
//!
//! # How to maintain
//!
//! #### The value tree
//!
//! [`SerializedValue`] is the format-independent intermediate representation everything
//! passes through. It is `Null`, a [`SerializedPrimitive`] (`Bool`, `Int` widened to `i128`,
//! `Float` widened to `f64`, or `String`), or a [`SerializedComposite`]. A composite holds
//! either an ordered [`SerializedMap`] (an [`indexmap::IndexMap`], so field order is
//! preserved) or a `List`, plus an `is_rigid` flag marking values that came from a fixed
//! shape — a struct, tuple, or enum variant — as opposed to a dynamic collection. A map may
//! also carry [`SerializedEnumMetadata`] recording the active variant and the full variant
//! list.
//!
//! #### Two traits, and the bridge between them
//!
//! [`TransSerializable`] is implemented by a type that knows how to describe *itself*;
//! [`TransSerializer`] is what the registry stores — a serializer keyed to a `Deserialized`
//! type. [`StandardTransSerializer<T>`] is the bridge: it is a [`TransSerializer`] that
//! forwards to `T`'s [`TransSerializable`] impl, which is why callers register
//! `StandardTransSerializer::<T>::default()` rather than the type directly. "Trans" is
//! transitive — a type serializes its fields by handing them back to the context, which
//! looks up *their* registered serializers, so composite types compose without each impl
//! knowing the others.
//!
//! #### Runtime type erasure
//!
//! [`SerializerRegistry`] keys serializers by `std::any::type_name::<T>()`. Internally each
//! entry is a `dyn VirtualSerializer` trait object wrapping an `AbstractSerializer<T>`;
//! lookups recover the concrete type with an unsafe pointer cast (`downcast_serializer_ref`)
//! guarded only by string-equality of the stored type name. **Caveat for maintainers:**
//! `type_name` is explicitly not guaranteed by the compiler to be unique or stable across
//! builds, so this erasure scheme is sound only as long as distinct types never collide on
//! their `type_name` string — keep that assumption in mind before extending the registry.
//!
//! #### Global and local registries
//!
//! A [`SerializerCtx`] borrows a `'static` global [`SerializerRegistry`] and an optional
//! local one, and always consults the local registry first. The local registry's lifetime
//! parameter lets it hold *borrowing* serializers — this is how the prefab loading path in
//! `fruits_asset_loading` registers transient, per-load serializers (for example one that
//! remaps `Entity` ids) on top of the engine's shared [`GlobalSerializer`].
//!
//! #### Errors accumulate; deserialization is lenient
//!
//! [`SerializationResult`] carries the produced `result` *alongside* a `Vec` of
//! [`SerializationError`], rather than short-circuiting. Deserialization is deliberately
//! coercive: reading an int from a string, a bool from a number, a `Vec` from a rigid
//! struct, and so on each yields a best-effort value while pushing an `InvalidInput` error
//! onto the result. A missing registered serializer pushes `MissingSerializer` and yields a
//! null/`None`. [`SerializationResult::unwrap`] panics only if any error was recorded, so it
//! reports *all* problems at once instead of the first.
//!
//! #### The JSON projection is lossy
//!
//! [`SerializedValue::to_json`] clamps `Int` into the `[i64::MIN, u64::MAX]` range and maps
//! non-representable (e.g. non-finite) `Float`s to `Null`. Enum metadata is written under the
//! reserved key `$enum_variant`; a real field literally named `$enum_variant` is skipped with
//! a warning on `stderr`. [`SerializedValue::from_json`] cannot recover the `is_rigid` flag
//! (always reconstructed as `false`) nor the enum variant list (left empty), so a
//! JSON round-trip is not structure-preserving for those fields.
//!
//! #### Enum decoding and the FFI path
//!
//! [`SerializerCtx::deserialize_enum`] builds an [`EnumDeserializerCtx`]; its `finish` first
//! tries the deserializer registered for the variant named in the metadata, and if none
//! matches it records an error and falls back to trying every registered variant in order,
//! returning the first that produces a value. [`SerializerCtx::deserialize_any`] is the
//! string-keyed, type-erased entry point used by the FFI/editor surface: it dispatches by a
//! serializer id and returns a [`fruits_ffi::FfiAny`]. The numerous `// todo: ffi` markers
//! across the crate flag where that surface is still being built out, alongside the crate
//! root's `todo` list of not-yet-implemented impls (tuples, maps, arrays) and the
//! commented-out blanket `serde`-backed impl in `serialization_transitive.rs`.
//!
//! #### Code generation
//!
//! The [`TransSerializable`](derive@TransSerializable) derive (in `fruits_serialization_macros`,
//! re-exported here) emits impls by building Rust *source text* and parsing it. Named structs
//! become rigid maps keyed by field name; tuple structs become rigid maps keyed by the field
//! index as a string (`"0"`, `"1"`, …); enums become rigid maps tagged with the variant name
//! and full variant list. Unions are rejected with a panic.

mod serialization_transitive;
mod serialization_registry;
mod serialization_impls;
mod serialization_model;

pub use fruits_serialization_macros::*;
pub use serialization_transitive::*;
pub use serialization_registry::*;
pub use serialization_impls::*;
pub use serialization_model::*;

// todo:
// + impls for standard types
// + macros
// - editor
// - ffi
// - names tuple/struct -> list/map
// - ecs resource (and other public APIs)
// - refactor?