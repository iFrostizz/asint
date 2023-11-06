use num_traits::{ops::overflowing::OverflowingAdd, ToBytes};
use std::{
    cmp::{max, Ordering},
    ops::Add,
};

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

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn last_nzero(&self) -> (usize, &u8) {
        self.inner
            .iter()
            .enumerate()
            .rfind(|(i, v)| v > &&0)
            .unwrap_or((0, &0))
    }

    /// change the length of the inner bytes vector to a size, set the value to max in case of overflow
    pub fn resize(&mut self, size: usize) {
        if size >= self.capacity() {
            self.inner.resize(size, 0);
        } else if self
            .inner
            .iter()
            .enumerate()
            .filter(|(i, _)| i > &size)
            .any(|(_, v)| v > &0u8)
        {
            self.inner.shrink_to(size);
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
            inner: value.to_be_bytes().to_vec(),
        }
    }
}

impl Add for DynUint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self
    }
}

impl OverflowingAdd for DynUint {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        todo!();
        let overflow_byte = max(self.len(), rhs.len());
        (self.clone(), false)
    }
}

impl PartialEq for DynUint {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
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
    fn ord() {
        let zero = DynUint::from(0);
        let one = DynUint::from(1);

        assert_eq!(zero, zero);
        assert_eq!(one, one);

        assert!(zero < one);
        assert!(zero <= one);
        assert!(zero <= zero);
        assert!(one > zero);
        assert!(one >= zero);
    }
}
