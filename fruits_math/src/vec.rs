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
        pub struct $V<T: Number> {
            $(pub $I: T),+
        }

        impl<T: Number> $V<T> {
            #[inline]
            pub const fn new($($I: T),+) -> Self {
                Self { $($I),+ }
            }

            #[inline]
            pub const fn with_all(v: T) -> Self {
                Self { $($I: v),+ }
            }

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

        impl<T: Number> Add for $V<T> {
            type Output = Self;
        
            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I + rhs.$I),+
                }
            }
        }
        
        impl<T: Number> AddAssign for $V<T> {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                $(self.$I += rhs.$I);+
            }
        }
        
        impl<T: Number> Sub for $V<T> {
            type Output = Self;
        
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I - rhs.$I),+
                }
            }
        }
        
        impl<T: Number> SubAssign for $V<T> {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$I -= rhs.$I);+
            }
        }

        impl<T: Number> Mul for $V<T> {
            type Output = Self;
        
            #[inline]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I * rhs.$I),+
                }
            }
        }

        impl<T: Number> Mul<T> for $V<T> {
            type Output = Self;
        
            #[inline]
            fn mul(self, rhs: T) -> Self::Output {
                Self::Output {
                    $($I: self.$I * rhs),+
                }
            }
        }
        
        impl<T: Number> MulAssign for $V<T> {
            #[inline]
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$I *= rhs.$I);+
            }
        }
        
        impl<T: Number> MulAssign<T> for $V<T> {
            #[inline]
            fn mul_assign(&mut self, rhs: T) {
                $(self.$I *= rhs);+
            }
        }
        
        impl<T: Number> Div for $V<T> {
            type Output = Self;
        
            #[inline]
            fn div(self, rhs: Self) -> Self::Output {
                Self::Output {
                    $($I: self.$I / rhs.$I),+
                }
            }
        }
        
        impl<T: Number> Div<T> for $V<T> {
            type Output = Self;
        
            #[inline]
            fn div(self, rhs: T) -> Self::Output {
                Self::Output {
                    $($I: self.$I / rhs),+
                }
            }
        }
        
        impl<T: Number> DivAssign for $V<T> {
            #[inline]
            fn div_assign(&mut self, rhs: Self) {
                $(self.$I /= rhs.$I);+
            }
        }
        
        impl<T: Number> DivAssign<T> for $V<T> {
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

impl<T: Number> Vec2<T> {
    pub const fn xx(&self) -> Vec2<T> { Vec2::<T>::new(self.x, self.x) }
    pub const fn xy(&self) -> Vec2<T> { Vec2::<T>::new(self.x, self.y) }
    pub const fn yx(&self) -> Vec2<T> { Vec2::<T>::new(self.y, self.x) }
    pub const fn yy(&self) -> Vec2<T> { Vec2::<T>::new(self.y, self.y) }
    

    pub const fn xxx(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.x, self.x) }
    pub const fn xxy(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.x, self.y) }
    pub const fn xyx(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.y, self.x) }
    pub const fn xyy(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.y, self.y) }
    pub const fn yxx(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.x, self.x) }
    pub const fn yxy(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.x, self.y) }
    pub const fn yyx(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.y, self.x) }
    pub const fn yyy(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.y, self.y) }

    
    pub const fn xxxx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.x) }
    pub const fn xxxy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.y) }
    pub const fn xxyx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.y, self.x) }
    pub const fn xxyy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.y, self.y) }
    pub const fn xyxx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.y, self.x, self.x) }
    pub const fn xyxy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.y, self.x, self.y) }
    pub const fn xyyx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.y, self.y, self.x) }
    pub const fn xyyy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.y, self.y, self.y) }
    pub const fn yxxx(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.x, self.x, self.x) }
    pub const fn yxxy(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.x, self.x, self.y) }
    pub const fn yxyx(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.x, self.y, self.x) }
    pub const fn yxyy(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.x, self.y, self.y) }
    pub const fn yyxx(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.y, self.x, self.x) }
    pub const fn yyxy(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.y, self.x, self.y) }
    pub const fn yyyx(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.y, self.y, self.x) }
    pub const fn yyyy(&self) -> Vec4<T> { Vec4::<T>::new(self.y, self.y, self.y, self.y) }
}

impl<T: Number> Vec3<T> {
    pub const fn xx(&self) -> Vec2<T> { Vec2::<T>::new(self.x, self.x) }
    pub const fn xy(&self) -> Vec2<T> { Vec2::<T>::new(self.x, self.y) }
    pub const fn xz(&self) -> Vec2<T> { Vec2::<T>::new(self.x, self.z) }
    pub const fn yx(&self) -> Vec2<T> { Vec2::<T>::new(self.y, self.x) }
    pub const fn yy(&self) -> Vec2<T> { Vec2::<T>::new(self.y, self.y) }
    pub const fn yz(&self) -> Vec2<T> { Vec2::<T>::new(self.y, self.z) }
    pub const fn zx(&self) -> Vec2<T> { Vec2::<T>::new(self.z, self.x) }
    pub const fn zy(&self) -> Vec2<T> { Vec2::<T>::new(self.z, self.y) }
    pub const fn zz(&self) -> Vec2<T> { Vec2::<T>::new(self.z, self.z) }
    

    pub const fn xxx(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.x, self.x) }
    pub const fn xxy(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.x, self.y) }
    pub const fn xxz(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.x, self.z) }
    pub const fn xyx(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.y, self.x) }
    pub const fn xyy(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.y, self.y) }
    pub const fn xyz(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.y, self.z) }
    pub const fn xzx(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.z, self.x) }
    pub const fn xzy(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.z, self.y) }
    pub const fn xzz(&self) -> Vec3<T> { Vec3::<T>::new(self.x, self.z, self.z) }
    pub const fn yxx(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.x, self.x) }
    pub const fn yxy(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.x, self.y) }
    pub const fn yxz(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.x, self.z) }
    pub const fn yyx(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.y, self.x) }
    pub const fn yyy(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.y, self.y) }
    pub const fn yyz(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.y, self.z) }
    pub const fn yzx(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.z, self.x) }
    pub const fn yzy(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.z, self.y) }
    pub const fn yzz(&self) -> Vec3<T> { Vec3::<T>::new(self.y, self.z, self.z) }
    pub const fn zxx(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.x, self.x) }
    pub const fn zxy(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.x, self.y) }
    pub const fn zxz(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.x, self.z) }
    pub const fn zyx(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.y, self.x) }
    pub const fn zyy(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.y, self.y) }
    pub const fn zyz(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.y, self.z) }
    pub const fn zzx(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.z, self.x) }
    pub const fn zzy(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.z, self.y) }
    pub const fn zzz(&self) -> Vec3<T> { Vec3::<T>::new(self.z, self.z, self.z) }

    
    pub const fn xxxx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.x) }
    pub const fn xxxy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.y) }
    pub const fn xxxz(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.z) }
    
    pub const fn xxyx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.y, self.x) }
    pub const fn xxyy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.y, self.y) }
    pub const fn xxyz(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.y, self.z) }
    
    pub const fn xxxx(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.x) }
    pub const fn xxxy(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.y) }
    pub const fn xxxz(&self) -> Vec4<T> { Vec4::<T>::new(self.x, self.x, self.x, self.z) }
}

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

// todo: swizzling
// todo: maybe unconstraint from Number trait
// todo: VectorIndex for optimized consecutive lookup.
// todo: const fn and inline where possible/needed.