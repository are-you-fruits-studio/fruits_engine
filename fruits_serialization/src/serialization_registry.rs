use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use fruits_ffi::FfiAny;

use crate::{SerializationResult, SerializedValue, SerializerCtx, TransSerializable};

pub trait TransSerializer {
    type Deserialized: 'static;

    fn serialize(&self, ctx: &SerializerCtx, value: &Self::Deserialized) -> SerializationResult<SerializedValue>;
    fn deserialize(&self, ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self::Deserialized>>;
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

    fn serialize(&self, ctx: &SerializerCtx, value: &Self::Deserialized) -> SerializationResult<SerializedValue> {
        T::serialize(value, ctx)
    }

    fn deserialize(&self, ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self::Deserialized>> {
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
    pub fn serialize(&self, ctx: &SerializerCtx, value: &T) -> SerializationResult<SerializedValue> {
        self.serializer.serialize(ctx, value)
    }
    pub fn deserialize(&self, ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<T>> {
        self.serializer.deserialize(ctx, value)
    }
}

pub(crate) trait VirtualSerializer<'se>: 'se {
    fn deserialized_type_name(&self) -> &'static str;
    fn deserialize_any(&self, ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<FfiAny>>;
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
    
    fn deserialize_any(&self, ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<FfiAny>> {
        self.deserialize(ctx, value).map(|o| o.map(FfiAny::new))
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

    pub fn serialize<'r, 'ctx: 'r, T: 'static>(&'r self, value: &T, ctx: Option<&'r SerializerRegistry<'ctx>>) -> SerializationResult<SerializedValue> {
        self.to_ctx(ctx).serialize(value)
    }

    pub fn deserialize<'r, 'ctx: 'r, T: 'static>(&'r self, data: &SerializedValue, ctx: Option<&'r SerializerRegistry<'ctx>>) -> SerializationResult<Option<T>> {
        self.to_ctx(ctx).deserialize(data)
    }

    pub fn to_ctx<'r, 'ctx: 'r>(&'r self, ctx: Option<&'r SerializerRegistry<'ctx>>) -> SerializerCtx<'r, 'ctx> {
        SerializerCtx::new(&self.serializers, ctx)
    }

    pub fn registry(&self) -> &SerializerRegistry<'static> {
        &self.serializers
    }
}
