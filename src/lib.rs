use num_traits::ops::overflowing::OverflowingAdd;
use std::{cmp::Ordering, ops::Add};

/// A dynamically allocated integer. The inner byte representation is in little-endian format
#[derive(Clone, Debug)]
pub struct DynUint {
    inner: Vec<u8>,
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

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn get_last_nzero(&self) -> Option<(usize, &u8)> {
        self.inner.iter().enumerate().rfind(|(_, v)| v > &&0)
    }

    /// return the latest non-zero byte
    fn last_nzero(&self) -> (usize, &u8) {
        self.get_last_nzero().unwrap_or((0, &0))
    }

    fn removed_trailing(&mut self) -> &Self {
        if let Some((i, _)) = self.get_last_nzero() {
            self.inner.drain(i..);
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
        // println!("{:?} {:?}", &self, &rhs);
        let (long, short) = if self.len() > rhs.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };

        let mut ret = Vec::with_capacity(long.len() + 1); // in case of overflow. TODO be smarter about it !
        let mut carry = false;
        for (i, lby) in long.inner.iter().enumerate() {
            let sby = short.inner.get(i).unwrap_or(&0);
            let mut aby = *lby as u16 + *sby as u16 + carry as u16;
            if aby >= u8::max_value().into() {
                carry = true;
                aby -= u8::max_value() as u16 + 1;
            } else {
                carry = false;
            }

            ret.push(aby as u8);
        }
        if carry {
            ret.push(1);
        }

        Self { inner: ret }
    }
}

impl OverflowingAdd for DynUint {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        todo!();
        // let overflow_byte = max(self.len(), rhs.len());
        // (self.clone(), false)
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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::DynUint;

    #[test]
    fn inner_repres() {
        let zero = DynUint::from(0u8);
        assert!(zero.clone().removed_trailing().is_empty());
    }

    #[test]
    fn ord() {
        let zero = DynUint::from(0u8);
        let one = DynUint::from(1u8);

        assert_eq!(zero, zero);
        assert_eq!(one, one);

        assert!(zero < one);
        assert!(zero <= one);
        assert!(zero <= zero);
        assert!(one > zero);
        assert!(one >= zero);
    }

    #[test]
    fn add() {
        let zero = DynUint::from(0usize);
        assert_eq!(zero.clone() + zero.clone(), zero.clone());

        let one = DynUint::from(1usize);
        assert_eq!(zero.clone() + one.clone(), one.clone());

        let two = DynUint::from(2usize);
        assert_eq!(one.clone() + one.clone(), two.clone());

        let nib = DynUint::from(128usize);
        let obyte = DynUint::from(256usize);
        assert_eq!(nib.clone() + nib.clone(), obyte.clone());
    }

    #[test]
    fn allocate() {
        let obyte = DynUint::from(256usize);
        let byte = DynUint::from(255u8);
        let one = DynUint::from(1u8);
        let res = byte.clone() + one.clone();
        assert_eq!(res, obyte);
        assert_eq!(res.len(), 2);
    }
}
