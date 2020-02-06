//! This is like [cycle](std::iter::Cycle) but after an iterator end the last element is returned
//! infinitely.

use std::iter::Fuse;

pub trait IteratorExt: Iterator {
    /// Creates an iterator that will be fused on last element forever.
    ///
    /// After an inner iterator `iter` returns None, future calls may or may not yield Some(T) again.
    /// `fuse_last()` adapts an iterator, ensuring that after a None is given, it will always return last received element forever.
    ///
    /// # Examples
    /// ```
    /// use futures_test_sink::fuse_last::IteratorExt;
    ///
    /// let iter = vec![1, 2].into_iter();
    /// let mut fuse_last_iter = iter.fuse_last();
    /// assert_eq!(Some(1), fuse_last_iter.next());
    /// assert_eq!(Some(2), fuse_last_iter.next());
    /// assert_eq!(Some(2), fuse_last_iter.next());
    /// ```
    fn fuse_last(self) -> FuseLast<Self, Self::Item>
    where
        Self: Sized,
        Self::Item: Clone;
}

impl<T: Iterator> IteratorExt for T {
    fn fuse_last(self) -> FuseLast<Self, Self::Item>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        FuseLast::new(self)
    }
}

pub struct FuseLast<I, Item> {
    iter: Fuse<I>,
    last_item: Option<Item>,
}

impl<I> FuseLast<I, I::Item>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(iter: I) -> Self {
        Self {
            iter: iter.fuse(),
            last_item: None,
        }
    }
}

impl<I> Iterator for FuseLast<I, I::Item>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(v) => {
                self.last_item = Some(v.clone());
                Some(v)
            }
            None => self.last_item.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compile() {
        let iter = vec![1, 2].into_iter();
        let mut fuse_last_iter = iter.fuse_last();
        assert_eq!(Some(1), fuse_last_iter.next());
        assert_eq!(Some(2), fuse_last_iter.next());
        assert_eq!(Some(2), fuse_last_iter.next());
    }
}
