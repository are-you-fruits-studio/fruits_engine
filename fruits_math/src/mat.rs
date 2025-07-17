use std::ops::{Index, IndexMut, Mul};

use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};

use crate::{num::Number, Primitive};

/// Column-major
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Mat<const N: usize, T> {
    data: [[T; N]; N],
}

impl<const N: usize, T: Primitive> Mat<N, T> {
    pub const fn from_array(data: [[T; N]; N]) -> Self {
        Self {
            data,
        }
    }

    pub const fn into_array(self) -> [[T; N]; N] { self.data }
    pub const fn as_array(&self) -> &[[T; N]; N] { &self.data }

    pub const fn try_col(&self, x: usize) -> Option<&[T; N]> {
        if x >= N {
            return None;
        }

        Some(&self.data[x])
    }

    pub const fn col(&self, x: usize) -> &[T; N] {
        &self.data[x]
    }

    pub const fn try_col_mut(&mut self, x: usize) -> Option<&mut [T; N]> {
        if x >= N {
            return None;
        }

        Some(&mut self.data[x])
    }

    pub const fn col_mut(&mut self, x: usize) -> &mut [T; N] {
        &mut self.data[x]
    }
    
    pub const fn try_row(&self, y: usize) -> Option<[T; N]> {
        if y >= N {
            return None;
        }

        let mut array = [T::ZERO; N];

        let mut i = 0;

        while i < N {
            array[i] = self.data[i][y];
            i += 1;
        }

        Some(array)
    }

    pub const fn row(&self, y: usize) -> [T; N] {
        let mut array = [T::ZERO; N];

        let mut i = 0;

        while i < N {
            array[i] = self.data[i][y];
            i += 1;
        }

        array
    }
    
    pub const fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x >= N || y >= N {
            return None;
        }

        Some(&self.data[x][y])
    }
    
    pub const fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= N || y >= N {
            return None;
        }

        Some(&mut self.data[x][y])
    }

    pub const fn with_all(v: T) -> Self {
        Self::from_array([[v; N]; N])
    }
    
    pub const fn transpose(&mut self) {
        let mut i = 0;

        while i < N {
            let mut j = 0;

            while j < i {
                (self.data[i][j], self.data[j][i]) = (self.data[j][i], self.data[i][j]);

                j += 1;
            }

            i += 1;
        }
    }

    pub const IDENTITY: Self = {
        let mut mat = Mat::<N, T>::with_all(T::ZERO);

        let mut i = 0;
        while i < N {
            mat.data[i][i] = T::ONE;
            i += 1;
        }

        mat
    };

    pub(crate) const fn ignored_element(&self, ignored_x: usize, ignored_y: usize, index_x: usize, index_y: usize) -> T {
        let data = self.as_array();

        data[index_x + (ignored_x <= index_x) as usize][index_y + (ignored_y <= index_y) as usize]
    }
}

impl<const N: usize, T: Primitive> Into<[[T; N]; N]> for Mat<N, T> {
    fn into(self) -> [[T; N]; N] {
        self.into_array()
    }
}

impl<const N: usize, T: Primitive> From<[[T; N]; N]> for Mat<N, T> {
    fn from(data: [[T; N]; N]) -> Self {
        Self::from_array(data)
    }
}

impl<const N: usize, T: Primitive> Index<(usize, usize)> for Mat<N, T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.data[x][y]
    }
}

impl<const N: usize, T: Primitive> IndexMut<(usize, usize)> for Mat<N, T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.data[x][y]
    }
}

impl<const N: usize, T: Primitive> Index<usize> for Mat<N, T> {
    type Output = [T; N];

    fn index(&self, i: usize) -> &Self::Output {
        &self.data[i]
    }
}

impl<const N: usize, T: Primitive> IndexMut<usize> for Mat<N, T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.data[i]
    }
}

impl<const N: usize, T: Number> Mul<T> for Mat<N, T> {
    type Output = Mat<N, T>;

    fn mul(mut self, rhs: T) -> Self::Output {
        let mut i = 0;
        
        while i < N {
            let mut j = 0;

            while j < N {
                self.data[i][j] *= rhs;

                j += 1;
            }

            i += 1;
        }

        self
    }
}

unsafe impl<const N: usize, T: AllBitVariationsValid> AllBitVariationsValid for Mat<N, T> { }
unsafe impl<const N: usize, T: AllBitsInit> AllBitsInit for Mat<N, T> { }