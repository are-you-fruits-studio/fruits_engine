use std::ops::{BitAnd, BitOr};

// todo: rewrite Debug impl
/// Bitflags describing which ends of a [`CollisionLine`](crate::CollisionLine) are bounded.
///
/// A line whose ends are unrestricted extends to infinity in that direction; restricting
/// both ends turns it into a finite segment ([`SEGMENT`](Self::SEGMENT)). The two flag bits
/// combine with [`BitOr`] and are tested with [`BitAnd`].
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct LineBoundType(u8);
impl LineBoundType {
    /// Neither end is bounded; the line is infinite in both directions.
    pub const UNRESTRICTED: LineBoundType = LineBoundType(0);
    /// The `start` end is bounded (the line is a ray clipped at its start).
    pub const START_RESTRICTED: LineBoundType = LineBoundType(1 << 0);
    /// The `end` end is bounded (the line is a ray clipped at its end).
    pub const END_RESTRICTED: LineBoundType = LineBoundType(1 << 1);
    /// Both ends are bounded; the line is a finite segment.
    pub const SEGMENT: LineBoundType = LineBoundType(LineBoundType::START_RESTRICTED.0 | LineBoundType::END_RESTRICTED.0);

    /// Returns `true` if the [`START_RESTRICTED`](Self::START_RESTRICTED) flag is set.
    pub const fn is_start_restricted(&self) -> bool {
        self.0 & Self::START_RESTRICTED.0 != 0
    }
    /// Returns `true` if the [`END_RESTRICTED`](Self::END_RESTRICTED) flag is set.
    pub const fn is_end_restricted(&self) -> bool {
        self.0 & Self::END_RESTRICTED.0 != 0
    }
}
impl BitOr for LineBoundType {
    type Output = LineBoundType;

    /// Returns the union of the two flag sets.
    fn bitor(self, rhs: Self) -> Self::Output {
        LineBoundType(self.0 | rhs.0)
    }
}
impl BitAnd for LineBoundType {
    type Output = LineBoundType;

    /// Returns the intersection of the two flag sets.
    fn bitand(self, rhs: Self) -> Self::Output {
        LineBoundType(self.0 & rhs.0)
    }
}
