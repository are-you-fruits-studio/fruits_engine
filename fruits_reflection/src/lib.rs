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

// todo
trait Reflect: 'static {
    fn refl_ty() -> ReflTy;
    fn to_refl_repr(&self) -> ReflRepr;
    fn from_refl_repr(repr: &ReflRepr) -> Self;
}
