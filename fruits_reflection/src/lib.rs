use std::any::{Any, TypeId};

pub struct ReflType {
    type_id: TypeId,
    type_name: &'static str,
    true_factory: *const (),
    factory_box_params_box_result: fn(*const (), Vec<Box<dyn Any>>) -> Box<dyn Any>,
    factory_box_params: *const (),
    fields: (),
}

impl ReflType {
    pub fn new<T: 'static, P: TupleFromVecOfAny>(factory: fn(P) -> T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            fields: (),
            true_factory: factory as *const (),
            factory_box_params_box_result: move |f, p| {
                let true_factory = unsafe { std::mem::transmute::<_, fn(P) -> T>(f) };

                Box::new(true_factory(P::tuple_from_vec_of_any(p).unwrap())) as Box<dyn Any>
            },
            factory_box_params: (move |f, p| {
                let true_factory = unsafe { std::mem::transmute::<_, fn(P) -> T>(f) };

                true_factory(P::tuple_from_vec_of_any(p).unwrap())
            }) as fn(*const (), Vec<Box<dyn Any>>) -> T as *const (),
        }
    }

    pub fn create_instance<T: 'static>(&self, params: Vec<Box<dyn Any>>) -> Option<T> {
        if &self.type_id != &TypeId::of::<T>() {
            return None;
        }
        
        let factory_box_params = unsafe { std::mem::transmute::<_, fn(*const (), Vec<Box<dyn Any>>) -> T>(self.factory_box_params) };

        Some(factory_box_params(self.true_factory, params))
    }

    pub fn create_instance_any(&self, parameters: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        (self.factory_box_params_box_result)(self.true_factory, parameters)
    }


}

pub trait TupleFromVecOfAny: 'static + Sized {
    fn tuple_from_vec_of_any(v: Vec<Box<dyn Any>>) -> Option<Self>;
}

macro_rules! tuple_from_vec_of_any_impl {
    ($($P: ident),*) => {
        impl<$($P: 'static),*> TupleFromVecOfAny for ($($P,)*) {
            fn tuple_from_vec_of_any(mut _v: Vec<Box<dyn Any>>) -> Option<Self> {
                let boxed_array: Box<[Box<dyn Any>; count!($($P),*)]> = _v.into_boxed_slice().try_into().ok()?;
                
                #[allow(non_snake_case)]
                let [$($P),*] = *boxed_array;
                
                Some((
                    $(*$P.downcast().ok()?,)*
                ))
            }
        }
    };
}

macro_rules! count {
    () => { 0 };
    ($t: ident$(, $ts: ident)*) => { 1 + count!($($ts),*) };
}

tuple_from_vec_of_any_impl!();
tuple_from_vec_of_any_impl!(P0);
tuple_from_vec_of_any_impl!(P0, P1);
tuple_from_vec_of_any_impl!(P0, P1, P2);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14);
tuple_from_vec_of_any_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15);
