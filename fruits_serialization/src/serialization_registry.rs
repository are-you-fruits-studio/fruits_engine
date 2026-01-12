use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use fruits_ffi::FfiAny;

use crate::{DeserializationError, SerializationError, SerializerCtx, TransSerializable};

pub trait TransSerializer {
    type Deserialized: 'static;

    fn serialize(&self, ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError>;
    fn deserialize(&self, ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError>;
}

// todo: ffi
pub struct StandardTransSerializer<T: TransSerializable> {
    _phantom: PhantomData<fn(T) -> T>,
}

impl<T: TransSerializable> Default for StandardTransSerializer<T> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T: TransSerializable> TransSerializer for StandardTransSerializer<T> {
    type Deserialized = T;

    fn serialize(&self, ctx: &SerializerCtx, value: &Self::Deserialized) -> Result<serde_json::Value, SerializationError> {
        T::serialize(value, ctx)
    }

    fn deserialize(&self, ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self::Deserialized, DeserializationError> {
        T::deserialize(ctx, value)
    }
}

// todo: ffi
pub(crate) struct AbstractSerializer<'se, T: 'static> {
    serializer: Box<dyn TransSerializer<Deserialized = T> + 'se + Send + Sync>,
}

impl<'se, T: 'static> AbstractSerializer<'se, T> {
    pub fn new(serializer: impl 'se + TransSerializer<Deserialized = T> + Send + Sync) -> Self {
        Self {
            serializer: Box::new(serializer),
        }
    }
    pub fn serialize(&self, ctx: &SerializerCtx, value: &T) -> Result<serde_json::Value, SerializationError> {
        self.serializer.serialize(ctx, value)
    }
    pub fn deserialize(&self, ctx: &SerializerCtx, value: &serde_json::Value) -> Result<T, DeserializationError> {
        self.serializer.deserialize(ctx, value)
    }
}

pub(crate) trait VirtualSerializer<'se>: 'se {
    fn deserialized_type_name(&self) -> &'static str;
    fn deserialize_any(&self, ctx: &SerializerCtx, value: &serde_json::Value) -> Result<FfiAny, DeserializationError>;
}

impl<'se> dyn VirtualSerializer<'se> + Send + Sync {
    pub(crate) fn downcast_serializer_ref<'r, T: 'se>(&'r self) -> Option<&'r AbstractSerializer<'se, T>>
        where 'se: 'r
    {
        unsafe {
            if self.deserialized_type_name() != std::any::type_name::<T>() {
                return None;
            }

            Some(&*(self as *const dyn VirtualSerializer<'se> as *const AbstractSerializer<T>))
        }
    }
}

impl<'se, T: 'static> VirtualSerializer<'se> for AbstractSerializer<'se, T> {
    fn deserialized_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
    
    fn deserialize_any(&self, ctx: &SerializerCtx, value: &serde_json::Value) -> Result<FfiAny, DeserializationError> {
        self.deserialize(ctx, value).map(FfiAny::new)
    }
}

// todo: ffi
#[derive(Default)]
pub struct SerializerRegistry<'se> {
    serializers: HashMap<Cow<'static, str>, Box<dyn VirtualSerializer<'se> + Send + Sync>>,
}

impl<'se> SerializerRegistry<'se> {
    pub fn new() -> Self {
        Self {
            serializers: HashMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, serializer: impl 'se + TransSerializer<Deserialized = T> + Send + Sync) {
        let serializer = Box::new(AbstractSerializer::<'se>::new(serializer));

        let type_name = std::any::type_name::<T>().into();

        self.serializers.insert(type_name, serializer);
    }

    pub(crate) fn get<'r, T: 'static>(&'r self) -> Option<&'r AbstractSerializer<'se, T>>
        where 'se: 'r
    {
        let type_name = std::any::type_name::<T>();
        let serializer = self.serializers.get(type_name)?;
        let serializer = serializer.downcast_serializer_ref::<T>().unwrap();

        Some(serializer)
    }

    pub(crate) fn get_virtual<'r>(&'r self, id: &str) -> Option<&'r (dyn VirtualSerializer<'se> + Send + Sync)>
        where 'se: 'r
    {
        self.serializers.get(id).map(|b| &**b)
    }
}

// todo: ffi
#[derive(Default)]
pub struct GlobalSerializer {
    serializers: SerializerRegistry<'static>,
}

impl GlobalSerializer {
    pub fn new() -> Self {
        Self {
            serializers: SerializerRegistry::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, serializer: impl 'static + TransSerializer<Deserialized = T> + Send + Sync) {
        self.serializers.register(serializer)
    }

    pub fn serialize<'r, 'ctx: 'r, T: 'static>(&self, value: &T, ctx: Option<&'r SerializerRegistry<'ctx>>) -> Result<serde_json::Value, SerializationError> {
        SerializerCtx::new(&self.serializers, ctx).serialize(value)
    }

    pub fn deserialize<'r, 'ctx: 'r, T: 'static>(&self, data: &serde_json::Value, ctx: Option<&'r SerializerRegistry<'ctx>>) -> Result<T, DeserializationError> {
        SerializerCtx::new(&self.serializers, ctx).deserialize(data)
    }

    pub fn registry(&self) -> &SerializerRegistry<'static> {
        &self.serializers
    }
}
