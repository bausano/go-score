use std::cmp::Ordering;
use std::ops::Sub;

pub trait NumExt
where
    Self: Sub<Output = Self> + PartialOrd<Self> + Copy + Sized,
{
    fn diff(self, other: Self) -> Self {
        if self > other {
            self - other
        } else {
            other - self
        }
    }

    fn partial_ord(self, other: Self) -> Ordering {
        if self > other {
            Ordering::Greater
        } else if self < other {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl NumExt for u8 {}
impl NumExt for u32 {}
impl NumExt for f32 {}
