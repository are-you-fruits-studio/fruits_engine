use std::ops::{BitAnd, BitOr};

// todo: rewrite Debug impl
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct LineBoundType(u8);
impl LineBoundType {
    pub const UNRESTRICTED: LineBoundType = LineBoundType(0);
    pub const START_RESTRICTED: LineBoundType = LineBoundType(1 << 0);
    pub const END_RESTRICTED: LineBoundType = LineBoundType(1 << 1);
    pub const SEGMENT: LineBoundType = LineBoundType(LineBoundType::START_RESTRICTED.0 | LineBoundType::END_RESTRICTED.0);

    pub const fn is_start_restricted(&self) -> bool {
        self.0 & Self::START_RESTRICTED.0 != 0
    }
    pub const fn is_end_restricted(&self) -> bool {
        self.0 & Self::END_RESTRICTED.0 != 0
    }
}
impl BitOr for LineBoundType {
    type Output = LineBoundType;

    fn bitor(self, rhs: Self) -> Self::Output {
        LineBoundType(self.0 | rhs.0)
    }
}
impl BitAnd for LineBoundType {
    type Output = LineBoundType;

    fn bitand(self, rhs: Self) -> Self::Output {
        LineBoundType(self.0 & rhs.0)
    }
}