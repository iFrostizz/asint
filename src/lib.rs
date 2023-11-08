use num_traits::ops::overflowing::OverflowingAdd;
use std::{
    cmp::Ordering,
    ops::{Add, BitAnd, Mul, Rem, Shl, Shr, Sub},
};

/// A dynamically allocated integer. The inner byte representation is in little-endian format
#[derive(Clone, Debug)]
pub struct DynUint {
    inner: Vec<u8>,
    // TODO
    // signed: bool,
}

impl DynUint {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Returns the length of the inner bytes
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Is the inner byte Vec empty ?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const ZERO: Self = Self { inner: vec![] };

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn get_ls<'a>(lhs: &'a DynUint, rhs: &'a DynUint) -> (&'a DynUint, &'a DynUint) {
        if lhs.len() > rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        }
    }

    fn get_ls_owned(lhs: DynUint, rhs: DynUint) -> (DynUint, DynUint) {
        if lhs.len() > rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        }
    }

    fn get_last_nzero(&self) -> Option<(usize, &u8)> {
        self.inner.iter().enumerate().rfind(|(_, v)| v > &&0)
    }

    // TODO !
    // fn signextend(&mut self) {
    //      !*self.signed;
    // }

    /// return the latest non-zero byte
    fn last_nzero(&self) -> (usize, &u8) {
        self.get_last_nzero().unwrap_or((0, &0))
    }

    fn removed_trailing(&mut self) -> &Self {
        if let Some((i, _)) = self.get_last_nzero() {
            self.inner.drain((i + 1)..);
        } else {
            self.inner.clear();
        };

        self
    }

    /// change the length of the inner bytes vector to a size, set the value to max in case of overflow
    pub fn resize(&mut self, size: usize) {
        let cap = self.capacity();
        if size >= cap {
            self.inner.resize(size, 0);
        } else if self
            .inner
            .iter()
            .enumerate()
            .filter(|(i, _)| i > &size)
            .any(|(_, v)| v > &0u8)
        {
            self.inner.reserve(cap - size);
            self.inner.iter_mut().for_each(|b| *b = 255);
        } else {
            self.inner.shrink_to(size);
        }
    }
}

// TODO macro that does it for all numeric types
impl From<usize> for DynUint {
    fn from(value: usize) -> Self {
        Self {
            inner: value.to_le_bytes().to_vec(),
        }
    }
}

impl From<i32> for DynUint {
    fn from(value: i32) -> Self {
        Self {
            inner: value.to_le_bytes().to_vec(),
        }
    }
}

impl From<u8> for DynUint {
    fn from(value: u8) -> Self {
        Self {
            inner: value.to_le_bytes().to_vec(),
        }
    }
}

impl From<bool> for DynUint {
    fn from(value: bool) -> Self {
        Self {
            inner: vec![value.into()],
        }
    }
}

impl Add for DynUint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut long, short) = Self::get_ls_owned(self, rhs);

        let mut carry = false;
        for (i, lby) in long.inner.iter_mut().enumerate() {
            let sby = short.inner.get(i).unwrap_or(&0);
            let mut aby = *lby as u16 + *sby as u16 + carry as u16;
            if aby > u8::max_value().into() {
                carry = true;
                aby -= 256;
            } else {
                carry = false;
            }

            *lby = aby as u8;
        }
        if carry {
            long.inner.push(1);
        }

        long
    }
}

impl Sub for DynUint {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        if rhs > self {
            todo!("signed sub support not done yet");
        }

        let mut carry = false;
        for (i, lby) in self.inner.iter_mut().enumerate() {
            let rby = rhs.inner.get(i).unwrap_or(&0);
            let mut aby = *lby as i16 - *rby as i16 - carry as i16;
            if aby < 0 {
                carry = true;
                aby += 256;
            } else {
                carry = false;
            }

            *lby = aby as u8;
        }
        if carry {
            // todo!();
            self.inner.push(1);
        }

        self
    }
}

// TODO ye
// impl AddAssign for DynUint {
//     fn add_assign(&mut self, rhs: Self) {
//         self = self + rhs;
//     }
// }

impl OverflowingAdd for DynUint {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        todo!();
        // let overflow_byte = max(self.len(), rhs.len());
        // (self.clone(), false)
    }
}

impl BitAnd for DynUint {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let (long, short) = Self::get_ls(&self, &rhs);
        Self {
            inner: long
                .inner
                .iter()
                .enumerate()
                .map(|(i, lb)| {
                    let rb = short.inner.get(i).unwrap_or(&0);
                    lb & rb
                })
                .collect(),
        }
    }
}

// TODO yee
// impl ShlAssign for DynUint {
//     fn shl_assign(&mut self, rhs: Self) {

//     }
// }

impl Rem for DynUint {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        if rhs == Self::ZERO {
            panic!("panik!");
        } else if rhs == Self::from(true) {
            Self::ZERO
        } else {
            // TODO only works for unsigned tho
            self & (rhs - Self::from(true))
        }
    }
}

impl Shl for DynUint {
    type Output = Self;

    fn shl(mut self, rhs: Self) -> Self::Output {
        if self != Self::ZERO || rhs != Self::ZERO {
            let mut rem = rhs;
            let mut shifted = 0usize;
            while rem >= 8u8.into() {
                rem = rem - 8u8.into();
                shifted += 1;
            }
            if shifted > 0 {
                for i in (shifted..self.len()).rev() {
                    self.inner[i - shifted] = self.inner[i];
                }
                for _ in 0..shifted {
                    self.inner.pop();
                }
            }
            if rem > Self::ZERO {
                let bit_shift = u8::from_le_bytes([*rem.inner.first().unwrap_or(&0)]);
                if !self.is_empty() {
                    let len = self.len();
                    for i in 0..len {
                        let b = self.inner.get_mut(i).unwrap();
                        *b >>= bit_shift;
                        if i < len - 1 {
                            let nb = self.inner[i + 1];
                            let carry = nb << (8 - bit_shift);
                            self.inner[i] |= carry;
                        }
                    }
                }
            }
        }

        self
    }
}

impl Shr for DynUint {
    type Output = Self;

    fn shr(mut self, rhs: Self) -> Self::Output {
        if self != Self::ZERO || rhs != Self::ZERO {
            let mut rem = rhs;
            let mut pushed = 0usize;
            while rem >= 8u8.into() {
                // TODO is binary search better for large numbers ?
                rem = rem - 8u8.into();
                self.inner.push(0);
                pushed += 1;
            }
            if pushed > 0 {
                let old_len = self.len() - pushed;
                for i in (0..old_len).rev() {
                    self.inner[i + pushed] = self.inner[i];
                    self.inner[i] = 0;
                }
            }
            if rem > Self::ZERO {
                let bit_shift = u8::from_le_bytes([*rem.inner.first().unwrap_or(&0)]);
                if !self.is_empty() {
                    let len = self.len();
                    self.inner.push(0);
                    for i in (0..len).rev() {
                        let b = self.inner.get_mut(i).unwrap();
                        let shifted = *b << bit_shift;
                        let carry = *b >> (8 - bit_shift);
                        *b = shifted;
                        self.inner[i + 1] += carry;
                    }
                }
            }
        }

        self
    }
}

impl Mul for DynUint {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self == Self::ZERO || rhs == Self::ZERO {
            Self::ZERO
        } else if self == DynUint::from(true) {
            rhs
        } else if rhs == DynUint::from(true) {
            self
        } else {
            let (long, mut short) = Self::get_ls_owned(self, rhs);
            let mut res = DynUint::from(0u8);
            // let diff = long.len() - short.len();
            // for (i, lb) in long.inner.iter().rev().enumerate() {
            // let sb = short.inner.get(i - diff).unwrap_or(&0);
            // let sb = short.inner.get(i).unwrap_or(&0);
            // let mulb = DynUint::from(lb * sb) >> DynUint::from(i * 8);
            // dbg!(lb * sb, &mulb);
            // let mulb = if
            // res = res + mulb;
            // }

            let mut count = DynUint::ZERO;
            while short != DynUint::ZERO {
                if short.inner.first().unwrap() & 1 == 1 {
                    let mulb = long.clone() >> count.clone();
                    res = res + mulb;
                }
                short = short << DynUint::from(true);
                count = count + DynUint::from(true);
            }

            res
        }
    }
}

impl PartialEq for DynUint {
    fn eq(&self, other: &Self) -> bool {
        self.clone().removed_trailing().inner == other.clone().removed_trailing().inner
    }
}

impl PartialOrd for DynUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ((i1, v1), (i2, v2)) = (self.last_nzero(), other.last_nzero());
        if i1 > i2 || (i1 == i2 && v1 > v2) {
            Some(Ordering::Greater)
        } else if i1 < i2 || (i1 == i2 && v1 < v2) {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DynUint;

    #[test]
    fn inner_repres() {
        let zero = DynUint::ZERO;
        assert!(zero.clone().removed_trailing().is_empty());
    }

    #[test]
    fn ord() {
        let zero = DynUint::ZERO;
        let one = DynUint::from(1);

        assert_eq!(zero, zero);
        assert_eq!(one, one);

        assert!(zero < one);
        assert!(zero <= one);
        assert!(zero <= zero);
        assert!(one > zero);
        assert!(one >= zero);
    }

    #[test]
    fn eq() {
        let zero1 = DynUint::ZERO;
        let zero2 = DynUint::from(0);
        assert_eq!(zero1, zero2);

        let one1 = DynUint::from(true);
        let one2 = DynUint::from(1);
        assert_eq!(one1, one2);

        assert_ne!(one1, zero1);
        assert_ne!(one2, zero1);
        assert_ne!(one1, zero2);
        assert_ne!(one2, zero2);
    }

    #[test]
    fn add() {
        assert_eq!(DynUint::ZERO + DynUint::ZERO, DynUint::ZERO);

        let one = DynUint::from(1);
        assert_eq!(DynUint::ZERO + one.clone(), one.clone());

        let two = DynUint::from(2);
        assert_eq!(one.clone() + one.clone(), two.clone());

        let nib = DynUint::from(128);
        let obyte = DynUint::from(256usize);
        assert_eq!(nib.clone() + nib.clone(), obyte.clone());

        assert_eq!(
            DynUint::from(254u8) + DynUint::from(1u8),
            DynUint::from(255u8)
        );
    }

    #[test]
    fn sub() {
        assert_eq!(DynUint::ZERO - DynUint::ZERO, DynUint::ZERO);

        let one = DynUint::from(1);
        assert_eq!(one.clone() - DynUint::ZERO, one.clone());
        assert_eq!(one.clone() - one.clone(), DynUint::ZERO);

        let two = DynUint::from(2);
        assert_eq!(two.clone() - one.clone(), one.clone());

        let obyte = DynUint::from(256);
        let byte = DynUint::from(255);
        assert_eq!(obyte.clone() - one.clone(), byte.clone());
    }

    #[ignore = "signed not supported ~yet"]
    #[test]
    fn sub_signed() {
        // TODO signed
        // let mone = DynUint::from(-1);
        // assert_eq!(zero.clone() - one.clone(), mone.clone());
    }

    #[test]
    fn and() {
        let one = DynUint::from(1);
        assert_eq!(DynUint::ZERO & one.clone(), DynUint::ZERO);

        let umax = DynUint::from(usize::max_value());
        assert_eq!(umax.clone() & one.clone(), one.clone());
    }

    #[ignore = "rem TODO"]
    #[test]
    fn rem() {
        let one = DynUint::from(1);
        assert_eq!(one.clone() % one.clone(), DynUint::ZERO);

        let two = DynUint::from(2);
        let five = DynUint::from(5);
        assert_eq!(five.clone() % two.clone(), one.clone());
        assert_eq!(two.clone() % five.clone(), two.clone());
    }

    #[test]
    fn shl() {
        assert_eq!(DynUint::ZERO << DynUint::ZERO, DynUint::ZERO);

        let one = DynUint::from(1);
        assert_eq!(one.clone() << DynUint::ZERO, one.clone());

        let two = DynUint::from(2);
        assert_eq!(two.clone() << one.clone(), one.clone());

        let seven = DynUint::from(7);
        let byte = DynUint::from(255);
        assert_eq!(byte.clone() << seven.clone(), one.clone());

        assert_eq!(DynUint::from(1234) << 5.into(), 38.into());
    }

    #[test]
    fn shr() {
        assert_eq!(DynUint::ZERO >> DynUint::ZERO, DynUint::ZERO);

        let one = DynUint::from(1);
        assert_eq!(one.clone() >> DynUint::ZERO, one.clone());

        let two = DynUint::from(2);
        assert_eq!(one.clone() >> one.clone(), two.clone());

        let eight = DynUint::from(8);
        let byte = DynUint::from(255);
        assert_eq!((one.clone() >> eight.clone()) - one.clone(), byte);

        assert_eq!(
            DynUint::from(25) >> DynUint::from(12),
            DynUint::from(102400)
        );
    }

    #[test]
    fn allocate() {
        let obyte = DynUint::from(256usize);
        let byte = DynUint::from(255u8);
        let one = DynUint::from(1u8);
        let res = byte.clone() + one.clone();
        assert_eq!(res, obyte);
        assert_eq!(res.len(), 2);

        let two = DynUint::from(2u8);
        let otwenhei = DynUint::from(128u8);
        // TODO two.pow(otwenhei)
    }

    #[test]
    fn mul() {
        assert_eq!(DynUint::ZERO * DynUint::from(100), DynUint::ZERO);
        assert_eq!(DynUint::from(true) * DynUint::from(100), DynUint::from(100));
        assert_eq!(DynUint::from(2u8) * DynUint::from(2u8), DynUint::from(4u8));
        assert_eq!(DynUint::from(69) * DynUint::from(420), DynUint::from(28980));
    }
}
