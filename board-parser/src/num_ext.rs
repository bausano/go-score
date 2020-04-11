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
}

impl NumExt for u8 {}
impl NumExt for u32 {}
impl NumExt for f32 {}
