use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign
};

use super::num::Number;

macro_rules! members_count {
    () => (0_usize);
    ($x: ident) => (1_usize);
    ($x: ident, $($xs: ident),*) => (1_usize + members_count!($($xs),*));
}

macro_rules! vec_impl {
    ($V: ident, $($I: ident),+) => {
        #[derive(Copy, Clone, Debug, Hash, Default)]
        #[repr(C)]
        pub struct $V<T> {
            $(pub $I: T),+
        }

        impl<T> $V<T> {
            #[inline]
            pub const fn new($($I: T),+) -> Self {
                Self { $($I),+ }
            }
        }

        impl<T: Copy> $V<T> {
            #[inline]
            pub const fn with_all(v: T) -> Self {
                Self { $($I: v),+ }
            }
        }

        impl<T: Clone> $V<T> {
            #[inline]
            pub fn with_all_cloned(v: &T) -> Self {
                Self { $($I: v.clone()),+ }
            }
        }

        impl<T: Number> $V<T> {
            #[inline]
            pub const fn as_array(&self) -> &[T; members_count!($($I),+)] {
                unsafe { std::mem::transmute(self) }
            }

            #[inline]
            pub const fn as_array_mut(&mut self) -> &mut [T; members_count!($($I),+)] {
                unsafe { std::mem::transmute(self) }
            }

            #[inline]
            pub const fn from_array_ref(a: &[T; members_count!($($I),+)]) -> &Self {
                unsafe { std::mem::transmute(a) }
            }

            #[inline]
            pub const fn from_array_mut(a: &mut [T; members_count!($($I),+)]) -> &mut Self {
                unsafe { std::mem::transmute(a) }
            }

            #[inline]
            pub const fn into_array(self) -> [T; members_count!($($I),+)] {
                [
                    $(self.$I),+
                ]
            }

            #[inline]
            pub const fn from_array(a: [T; members_count!($($I),+)]) -> Self {
                let [$($I),+] = a;
                Self {
                    $($I),+
                }
            }
            
            #[inline]
            pub fn dot(&self, rhs: &Self) -> T {
                dot(self.as_array(), rhs.as_array())
            }

            #[inline]
            pub fn length_sqrt(&self) -> T {
                length_sqrt(self.as_array())
            }

            #[inline]
            pub fn length(&self) -> f64 {
                length(self.as_array())
            }

            #[inline]
            pub fn normalized(&self) -> Self {
                Self::from_array(normalized(self.as_array()))
            }

            #[inline]
            pub fn map<U: Number>(&self, f: impl Fn(T) -> U) -> $V<U> {
                $V::<U>::from_array(self.as_array().map(f))
            }
        }

        impl<T: Number> PartialEq for $V<T> {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.as_array() == rhs.as_array()
            }
        }

        impl<T: Number + Eq> Eq for $V<T> { }

        impl<T: Number> Into<[T; members_count!($($I),+)]> for $V<T> {
            #[inline]
            fn into(self) -> [T; members_count!($($I),+)] {
                self.into_array()
            }
        }

        impl<T: Number> From<[T; members_count!($($I),+)]> for $V<T> {
            #[inline]
            fn from(a: [T; members_count!($($I),+)]) -> Self {
                Self::from_array(a)
            }
        }

        impl<T: Add> Add for $V<T> {
            type Output = $V<<T as Add>::Output>;
        
            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I + rhs.$I),+
                }
            }
        }
        
        impl<T: AddAssign> AddAssign for $V<T> {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                $(self.$I += rhs.$I);+
            }
        }
        
        impl<T: Sub> Sub for $V<T> {
            type Output = $V<<T as Sub>::Output>;
        
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I - rhs.$I),+
                }
            }
        }
        
        impl<T: SubAssign> SubAssign for $V<T> {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$I -= rhs.$I);+
            }
        }

        impl<T: Mul> Mul for $V<T> {
            type Output = $V<<T as Mul>::Output>;
        
            #[inline]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I * rhs.$I),+
                }
            }
        }

        impl<T: Mul + Copy> Mul<T> for $V<T> {
            type Output = $V<<T as Mul>::Output>;
        
            #[inline]
            fn mul(self, rhs: T) -> Self::Output {
                Self::Output {
                    $($I: self.$I * rhs),+
                }
            }
        }
        
        impl<T: MulAssign> MulAssign for $V<T> {
            #[inline]
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$I *= rhs.$I);+
            }
        }
        
        impl<T: MulAssign + Copy> MulAssign<T> for $V<T> {
            #[inline]
            fn mul_assign(&mut self, rhs: T) {
                $(self.$I *= rhs);+
            }
        }
        
        impl<T: Div> Div for $V<T> {
            type Output = $V<<T as Div>::Output>;
        
            #[inline]
            fn div(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I / rhs.$I),+
                }
            }
        }
        
        impl<T: Div + Copy> Div<T> for $V<T> {
            type Output = $V<<T as Div>::Output>;
        
            #[inline]
            fn div(self, rhs: T) -> Self::Output {
                Self::Output {
                    $($I: self.$I / rhs),+
                }
            }
        }
        
        impl<T: DivAssign> DivAssign for $V<T> {
            #[inline]
            fn div_assign(&mut self, rhs: Self) {
                $(self.$I /= rhs.$I);+
            }
        }
        
        impl<T: DivAssign + Copy> DivAssign<T> for $V<T> {
            #[inline]
            fn div_assign(&mut self, rhs: T) {
                $(self.$I /= rhs);+
            }
        }

        impl<T: Number> Index<usize> for $V<T> {
            type Output = T;
        
            #[inline]
            fn index(&self, i: usize) -> &Self::Output {
                &self.as_array()[i]
            }
        }

        impl<T: Number> IndexMut<usize> for $V<T> {
            #[inline]
            fn index_mut(&mut self, i: usize) -> &mut Self::Output {
                &mut self.as_array_mut()[i]
            }
        }
    };
}

vec_impl!{Vec2, x, y}
vec_impl!{Vec3, x, y, z}
vec_impl!{Vec4, x, y, z, w}

fruits_swizzling::swizzling!{}

impl<T: Number> Vec3<T> {
    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self::from_array(cross(self.as_array(), rhs.as_array()))
    }
}

#[inline]
pub fn dot<T: Number, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> T {
    let mut sum = T::ZERO;

    for i in 0..N {
        sum += lhs[i] * rhs[i];
    }
    
    sum
}
#[inline]
pub fn length_sqrt<T: Number, const N: usize>(v: &[T; N]) -> T {
    dot(v, v)
}
#[inline]
pub fn length<T: Number, const N: usize>(v: &[T; N]) -> f64 {
    length_sqrt(v).into_f64().sqrt()
}
#[inline]
pub fn normalized<T: Number, const N: usize>(v: &[T; N]) -> [T; N] {
    if *v == [T::ZERO; N] {
        *v
    } else {
        div_by(v, T::from_f64(length(v)))
    }
}
#[inline]
pub fn add<T: Number, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> [T; N] {
    let mut result = [T::ZERO; N];

    for i in 0..N {
        result[i] = lhs[i] + rhs[i];
    }

    result
}
#[inline]
pub fn add_assign<T: Number, const N: usize>(lhs: &mut [T; N], rhs: &[T; N]) {
    for i in 0..N {
        lhs[i] += rhs[i];
    }
}
#[inline]
pub fn sub<T: Number, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> [T; N] {
    let mut result = [T::ZERO; N];

    for i in 0..N {
        result[i] = lhs[i] - rhs[i];
    }

    result
}
#[inline]
pub fn sub_assign<T: Number, const N: usize>(lhs: &mut [T; N], rhs: &[T; N]) {
    for i in 0..N {
        lhs[i] -= rhs[i];
    }
}
#[inline]
pub fn mul<T: Number, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> [T; N] {
    let mut result = [T::ZERO; N];

    for i in 0..N {
        result[i] = lhs[i] * rhs[i];
    }

    result
}
#[inline]
pub fn mul_assign<T: Number, const N: usize>(lhs: &mut [T; N], rhs: &[T; N]) {
    for i in 0..N {
        lhs[i] *= rhs[i];
    }
}
#[inline]
pub fn div<T: Number, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> [T; N] {
    let mut result = [T::ZERO; N];

    for i in 0..N {
        result[i] = lhs[i] / rhs[i];
    }

    result
}
#[inline]
pub fn div_assign<T: Number, const N: usize>(lhs: &mut [T; N], rhs: &[T; N]) {
    for i in 0..N {
        lhs[i] /= rhs[i];
    }
}
#[inline]
pub fn mul_by<T: Number, const N: usize>(lhs: &[T; N], rhs: T) -> [T; N] {
    let mut result = [T::ZERO; N];

    for i in 0..N {
        result[i] = lhs[i] * rhs;
    }

    result
}
#[inline]
pub fn mul_by_assign<T: Number, const N: usize>(lhs: &mut [T; N], rhs: T) {
    for i in 0..N {
        lhs[i] *= rhs;
    }
}
#[inline]
pub fn div_by<T: Number, const N: usize>(lhs: &[T; N], rhs: T) -> [T; N] {
    let mut result = [T::ZERO; N];

    for i in 0..N {
        result[i] = lhs[i] / rhs;
    }

    result
}
#[inline]
pub fn div_by_assign<T: Number, const N: usize>(lhs: &mut [T; N], rhs: T) {
    for i in 0..N {
        lhs[i] /= rhs;
    }
}
#[inline]
pub fn cross<T: Number>(lhs: &[T; 3], rhs: &[T; 3]) -> [T; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}
#[inline]
pub fn cross_2d<T: Number>(lhs: &[T; 2], rhs: &[T; 2]) -> T {
    lhs[0] * rhs[1] - lhs[1] * rhs[0]
}
#[inline]
pub const fn swizzle<T: Number, const N: usize, const S: usize>(v: &[T; N], i: &[usize; N]) -> [T; S] {
    let mut result = [T::ZERO; S];

    let mut j  = 0;

    while j < S {
        result[j] = v[i[j]];

        j += 1;
    }

    result
}

//

// todo: maybe unconstraint from Number trait.
// todo: const fn and inline where possible/needed.