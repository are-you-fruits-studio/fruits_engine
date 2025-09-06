mod refl_ty;
mod refl_repr;
mod refl_map;
mod tuple_from_vec_of_any;
mod registry;
mod refl_repr_reg_entries;

pub use refl_ty::*;
pub use refl_repr::*;
pub use refl_map::*;
pub use tuple_from_vec_of_any::*;
pub use registry::*;
pub use refl_repr_reg_entries::*;

pub use fruits_reflection_macros::*;

pub trait ReflectTy: 'static {
    fn refl_ty() -> ReflTy;
}

pub trait ReflectTyVal: 'static {
    fn refl_ty_val(&self) -> ReflTy;
}

impl<T: ReflectTy> ReflectTyVal for T {
    fn refl_ty_val(&self) -> ReflTy {
        T::refl_ty()
    }
}