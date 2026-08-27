use std::{ffi::c_void, marker::PhantomData, mem::MaybeUninit};

use fruits_ffi::{FfiAny, FfiAnyRef, FfiDroppable, FfiFnMutMut, FfiIndexMap, FfiOption, FfiString};

use crate::{SerializationError, SerializedValue, SerializerCtx, TransSerializable};

pub trait TransSerializer {
    type Deserialized: 'static;

    fn serialize(&self, ctx: SerializerCtx, value: &Self::Deserialized) -> SerializedValue;
    fn deserialize(&self, ctx: SerializerCtx, value: &SerializedValue) -> Option<Self::Deserialized>;
}

#[repr(C)]
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

    fn serialize(&self, ctx: SerializerCtx, value: &Self::Deserialized) -> SerializedValue {
        T::serialize(value, ctx)
    }

    fn deserialize(&self, ctx: SerializerCtx, value: &SerializedValue) -> Option<Self::Deserialized> {
        T::deserialize(ctx, value)
    }
}

#[repr(C)]
struct TransSerializerFfiVtable {
    fn_serialize: unsafe extern "C-unwind" fn(*const c_void, ctx: SerializerCtx, value: *const c_void) -> SerializedValue,
    fn_deserialize: unsafe extern "C-unwind" fn(*const c_void, ctx: SerializerCtx, value: &SerializedValue, out: *mut c_void),
    fn_deserialize_any: unsafe extern "C-unwind" fn(*const c_void, ctx: SerializerCtx, value: &SerializedValue) -> FfiOption<FfiAny>,
}

#[repr(C)]
pub struct TransSerializerFfi<'se> {
    data: FfiDroppable,
    vtable: &'static TransSerializerFfiVtable,
    _phantom: PhantomData<&'se mut ()>,
}

impl<'se> TransSerializerFfi<'se> {
    fn new<T: 'static, S: 'se + TransSerializer<Deserialized = T> + Send + Sync>(serializer: S) -> Self {
        unsafe extern "C-unwind" fn ffi_serialize<'se, T, S: 'se + TransSerializer<Deserialized = T> + Send + Sync>(this: *const c_void, ctx: SerializerCtx, value: *const c_void) -> SerializedValue {
            unsafe {
                let serializer = &*(this as *const S);
                let value = &*(value as *const T);

                serializer.serialize(ctx, value)
            }
        }
        unsafe extern "C-unwind" fn ffi_deserialize<'se, T, S: 'se + TransSerializer<Deserialized = T> + Send + Sync>(this: *const c_void, ctx: SerializerCtx, value: &SerializedValue, out: *mut c_void) {
            unsafe {
                let serializer = &*(this as *const S);
                let out = out as *mut Option<T>;

                let result: Option<T> = serializer.deserialize(ctx, value);

                out.write(result);
            }
        }
        unsafe extern "C-unwind" fn ffi_deserialize_any<'se, T: 'static, S: 'se + TransSerializer<Deserialized = T> + Send + Sync>(this: *const c_void, ctx: SerializerCtx, value: &SerializedValue) -> FfiOption<FfiAny> {
            unsafe {
                let serializer = &*(this as *const S);

                let result: Option<T> = serializer.deserialize(ctx, value);

                result.map(FfiAny::new).into()
            }
        }

        Self {
            data: FfiDroppable::new(serializer),
            vtable: &TransSerializerFfiVtable {
                fn_serialize: ffi_serialize::<T, S>,
                fn_deserialize: ffi_deserialize::<T, S>,
                fn_deserialize_any: ffi_deserialize_any::<T, S>,
            },
            _phantom: PhantomData,
        }
    }

    // todo
    unsafe fn serialize<T>(&self, ctx: SerializerCtx, value: &T) -> SerializedValue {
        unsafe {
            let value = value as *const T as *const c_void;

            (self.vtable.fn_serialize)(self.data.get(), ctx, value)
        }
    }
    // todo
    pub unsafe fn serialize_any(&self, ctx: SerializerCtx, value: FfiAnyRef) -> SerializedValue {
        unsafe {
            let value = value.ptr() as *const c_void;

            (self.vtable.fn_serialize)(self.data.get(), ctx, value)
        }
    }
    // todo
    unsafe fn deserialize<T>(&self, ctx: SerializerCtx, value: &SerializedValue) -> Option<T> {
        unsafe {
            let mut out = MaybeUninit::<Option<T>>::uninit();
           
            (self.vtable.fn_deserialize)(self.data.get(), ctx, value, out.as_mut_ptr() as *mut c_void);

            out.assume_init()
        }
    }
    //
    // todo
    // fn deserialized_type_name(&self) -> &'static str {
    //     std::any::type_name::<T>()
    // }
    pub fn deserialize_any(&self, ctx: SerializerCtx, value: &SerializedValue) -> Option<FfiAny> {
        unsafe {
            (self.vtable.fn_deserialize_any)(self.data.get(), ctx, value).into()
        }
    }
}

unsafe impl<'se> Send for TransSerializerFfi<'se> { }
unsafe impl<'se> Sync for TransSerializerFfi<'se> { }

pub struct TransSerializerFfiTypedRef<'r, 'se, T> {
    serializer: &'r TransSerializerFfi<'se>,
    _phantom: PhantomData<fn(T) -> T>,
}

impl<'r, 'se, T> TransSerializerFfiTypedRef<'r, 'se, T> {
    unsafe fn new(serializer: &'r TransSerializerFfi<'se>) -> Self {
        Self {
            serializer,
            _phantom: PhantomData,
        }
    }

    pub fn serialize(&self, ctx: SerializerCtx, value: &T) -> SerializedValue {
        // todo
        unsafe {
            self.serializer.serialize::<T>(ctx, value)
        }
    }

    pub unsafe fn serialize_any(&self, ctx: SerializerCtx, value: FfiAnyRef) -> SerializedValue {
        // todo
        unsafe {
            self.serializer.serialize_any(ctx, value)
        }
    }

    pub fn deserialize(&self, ctx: SerializerCtx, value: &SerializedValue) -> Option<T> {
        // todo
        unsafe {
            self.serializer.deserialize::<T>(ctx, value)
        }
    }
   
    pub fn deserialize_any(&self, ctx: SerializerCtx, value: &SerializedValue) -> Option<FfiAny> {
        self.serializer.deserialize_any(ctx, value)
    }
}

//

#[repr(C)]
#[derive(Default)]
pub struct SerializerRegistry<'se> {
    serializers: FfiIndexMap<FfiString, TransSerializerFfi<'se>>,
}

impl<'se> SerializerRegistry<'se> {
    pub fn new() -> Self {
        Self {
            serializers: FfiIndexMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, serializer: impl 'se + TransSerializer<Deserialized = T> + Send + Sync) {
        let serializer = TransSerializerFfi::new(serializer);

        let type_name = std::any::type_name::<T>().into();

        self.serializers.insert(type_name, serializer);
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.serializers.keys().map(|s| s.as_str())
    }

    pub(crate) fn get<'r, T: 'static>(&'r self) -> Option<TransSerializerFfiTypedRef<'r, 'se, T>>
        where 'se: 'r
    {
        unsafe {
            let type_name = std::any::type_name::<T>();
            let serializer = self.serializers.get(type_name)?;
            Some(TransSerializerFfiTypedRef::new(serializer))
        }
    }

    pub(crate) fn get_virtual<'r>(&'r self, id: &str) -> Option<&'r TransSerializerFfi<'se>>
        where 'se: 'r
    {
        self.serializers.get(id)
    }
}

#[repr(C)]
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

    pub fn serialize<'r, 'l: 'r, T: 'static>(&'r mut self, value: &T, ctx: Option<&'r SerializerRegistry<'l>>, err_handler: &'r mut impl FnMut(SerializationError)) -> SerializedValue {
        self.to_ctx(ctx, err_handler).serialize(value)
    }

    pub fn deserialize<'r, 'l: 'r, T: 'static>(&'r mut self, data: &SerializedValue, ctx: Option<&'r SerializerRegistry<'l>>, err_handler: &'r mut impl FnMut(SerializationError)) -> Option<T> {
        self.to_ctx(ctx, err_handler).deserialize(data)
    }

    pub fn to_ctx<'r, 'l: 'r>(&'r self, ctx: Option<&'r SerializerRegistry<'l>>, err_handler: &'r mut impl FnMut(SerializationError)) -> SerializerCtx<'r, 'l> {
        SerializerCtx::new(&self.serializers, ctx, FfiFnMutMut::new(err_handler))
    }

    pub fn registry(&self) -> &SerializerRegistry<'static> {
        &self.serializers
    }
}
