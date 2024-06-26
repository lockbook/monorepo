use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Range, Sub, SubAssign};

/// A byte position in a buffer
#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DocByteOffset(pub usize);

/// A byte offset from a position in a buffer or a distance between two positions
#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RelByteOffset(pub usize);

/// A character position in a buffer
#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DocCharOffset(pub usize);

/// A character offset from a position in a buffer or a distance between two positions
#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RelCharOffset(pub usize);

// rel +/- rel = rel, doc +/- rel = doc, doc - doc = rel
impl Add<RelByteOffset> for RelByteOffset {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<RelByteOffset> for RelByteOffset {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl AddAssign<RelByteOffset> for RelByteOffset {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign<RelByteOffset> for RelByteOffset {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}
impl Add<RelByteOffset> for DocByteOffset {
    type Output = Self;

    fn add(self, rhs: RelByteOffset) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<RelByteOffset> for DocByteOffset {
    type Output = Self;

    fn sub(self, rhs: RelByteOffset) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl AddAssign<RelByteOffset> for DocByteOffset {
    fn add_assign(&mut self, rhs: RelByteOffset) {
        self.0 += rhs.0
    }
}

impl SubAssign<RelByteOffset> for DocByteOffset {
    fn sub_assign(&mut self, rhs: RelByteOffset) {
        self.0 -= rhs.0
    }
}

impl Sub<DocByteOffset> for DocByteOffset {
    type Output = RelByteOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        RelByteOffset(self.0.saturating_sub(rhs.0))
    }
}

impl Add<RelCharOffset> for RelCharOffset {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<RelCharOffset> for RelCharOffset {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl AddAssign<RelCharOffset> for RelCharOffset {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign<RelCharOffset> for RelCharOffset {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}
impl Add<RelCharOffset> for DocCharOffset {
    type Output = Self;

    fn add(self, rhs: RelCharOffset) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<RelCharOffset> for DocCharOffset {
    type Output = Self;

    fn sub(self, rhs: RelCharOffset) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl AddAssign<RelCharOffset> for DocCharOffset {
    fn add_assign(&mut self, rhs: RelCharOffset) {
        self.0 += rhs.0
    }
}

impl SubAssign<RelCharOffset> for DocCharOffset {
    fn sub_assign(&mut self, rhs: RelCharOffset) {
        self.0 -= rhs.0
    }
}

impl Sub<DocCharOffset> for DocCharOffset {
    type Output = RelCharOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        RelCharOffset(self.0.saturating_sub(rhs.0))
    }
}

// all offset types impl From<usize>, PartialEq<usize>, PartialOrd<usize>, Add<usize>, Sub<usize>, AddAssign<usize>, SubAssign<usize>
impl From<usize> for DocByteOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl PartialEq<usize> for DocByteOffset {
    fn eq(&self, other: &usize) -> bool {
        &self.0 == other
    }
}

impl PartialOrd<usize> for DocByteOffset {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<DocByteOffset> for usize {
    fn eq(&self, other: &DocByteOffset) -> bool {
        self == &other.0
    }
}

impl PartialOrd<DocByteOffset> for usize {
    fn partial_cmp(&self, other: &DocByteOffset) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Add<usize> for DocByteOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        let sum = self.0 + rhs;
        Self(sum)
    }
}

impl Sub<usize> for DocByteOffset {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let sum = self.0.saturating_sub(rhs);
        Self(sum)
    }
}

impl AddAssign<usize> for DocByteOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl SubAssign<usize> for DocByteOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

impl From<usize> for RelByteOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl PartialEq<usize> for RelByteOffset {
    fn eq(&self, other: &usize) -> bool {
        &self.0 == other
    }
}

impl PartialOrd<usize> for RelByteOffset {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<RelByteOffset> for usize {
    fn eq(&self, other: &RelByteOffset) -> bool {
        self == &other.0
    }
}

impl PartialOrd<RelByteOffset> for usize {
    fn partial_cmp(&self, other: &RelByteOffset) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Add<usize> for RelByteOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        let sum = self.0 + rhs;
        Self(sum)
    }
}

impl Sub<usize> for RelByteOffset {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let sum = self.0.saturating_sub(rhs);
        Self(sum)
    }
}

impl AddAssign<usize> for RelByteOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl SubAssign<usize> for RelByteOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

impl From<usize> for DocCharOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl PartialEq<usize> for DocCharOffset {
    fn eq(&self, other: &usize) -> bool {
        &self.0 == other
    }
}

impl PartialOrd<usize> for DocCharOffset {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<DocCharOffset> for usize {
    fn eq(&self, other: &DocCharOffset) -> bool {
        self == &other.0
    }
}

impl PartialOrd<DocCharOffset> for usize {
    fn partial_cmp(&self, other: &DocCharOffset) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Add<usize> for DocCharOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        let sum = self.0 + rhs;
        Self(sum)
    }
}

impl Sub<usize> for DocCharOffset {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let sum = self.0.saturating_sub(rhs);
        Self(sum)
    }
}

impl AddAssign<usize> for DocCharOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl SubAssign<usize> for DocCharOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

impl From<usize> for RelCharOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl PartialEq<usize> for RelCharOffset {
    fn eq(&self, other: &usize) -> bool {
        &self.0 == other
    }
}

impl PartialOrd<usize> for RelCharOffset {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<RelCharOffset> for usize {
    fn eq(&self, other: &RelCharOffset) -> bool {
        self == &other.0
    }
}

impl PartialOrd<RelCharOffset> for usize {
    fn partial_cmp(&self, other: &RelCharOffset) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Add<usize> for RelCharOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        let sum = self.0 + rhs;
        Self(sum)
    }
}

impl Sub<usize> for RelCharOffset {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let sum = self.0.saturating_sub(rhs);
        Self(sum)
    }
}

impl AddAssign<usize> for RelCharOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl SubAssign<usize> for RelCharOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

impl Debug for DocByteOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Debug for RelByteOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Debug for DocCharOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Debug for RelCharOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub trait RangeExt<Element: Sub<Element> + Ord + Sized + Copy>: Sized {
    fn start(&self) -> Element;
    fn end(&self) -> Element;

    fn len(&self) -> <Element as Sub>::Output {
        self.end() - self.start()
    }

    fn is_empty(&self) -> bool {
        self.start() == self.end()
    }

    // todo: lazy af name
    fn contains_ext(&self, value: Element, start_inclusive: bool, end_inclusive: bool) -> bool {
        (self.start() < value || (start_inclusive && self.start() == value))
            && (value < self.end() || (end_inclusive && self.end() == value))
    }

    fn contains_inclusive(&self, value: Element) -> bool {
        self.contains_ext(value, true, true)
    }

    fn intersects<O: RangeExt<Element>>(&self, other: &O, allow_empty_intersection: bool) -> bool {
        (self.start() < other.end() || (allow_empty_intersection && self.start() == other.end()))
            && (other.start() < self.end()
                || (allow_empty_intersection && other.start() == self.end()))
    }

    fn intersects_allow_empty<O: RangeExt<Element>>(&self, other: &O) -> bool {
        self.intersects(other, true)
    }

    fn contains_range<O: RangeExt<Element>>(
        &self, value: &O, start_inclusive: bool, end_inclusive: bool,
    ) -> bool {
        self.contains_ext(value.start(), start_inclusive, end_inclusive)
            && self.contains_ext(value.end(), start_inclusive, end_inclusive)
    }
}

impl<T> RangeExt<T> for Range<T>
where
    T: Ord + Sized + Copy + Sub<T>,
{
    fn start(&self) -> T {
        self.start
    }

    fn end(&self) -> T {
        self.end
    }
}

pub trait ToRangeExt: Sized {
    fn to_range(self) -> (Self, Self);
}

impl<T> ToRangeExt for T
where
    T: Copy,
{
    fn to_range(self) -> (Self, Self) {
        (self, self)
    }
}

pub trait IntoRangeExt<I> {
    fn into_range(self) -> (I, I);
}

impl<T, I> IntoRangeExt<I> for T
where
    T: Copy + Into<I>,
{
    fn into_range(self) -> (I, I) {
        (self.into(), self.into())
    }
}

pub trait RangeIterExt {
    type Item;
    type Iter: DoubleEndedIterator<Item = Self::Item>;
    fn iter(self) -> Self::Iter;
}

impl RangeIterExt for Range<usize> {
    type Item = usize;
    type Iter = RangeIter;
    fn iter(self) -> Self::Iter {
        RangeIter { start_inclusive: self.start, end_exclusive: self.end }
    }
}

pub struct RangeIter {
    start_inclusive: usize,
    end_exclusive: usize,
}

impl Iterator for RangeIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_inclusive < self.end_exclusive {
            let result = self.start_inclusive;
            self.start_inclusive += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for RangeIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start_inclusive < self.end_exclusive {
            self.end_exclusive -= 1;
            Some(self.end_exclusive)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tab::markdown_editor::offset_types::RangeExt as _;

    #[test]
    fn contains_ext() {
        assert!(!(1..3).contains_ext(0, false, false));
        assert!(!(1..3).contains_ext(1, false, false));
        assert!((1..3).contains_ext(2, false, false));
        assert!(!(1..3).contains_ext(3, false, false));
        assert!(!(1..3).contains_ext(4, false, false));

        assert!(!(1..3).contains_ext(0, true, false));
        assert!((1..3).contains_ext(1, true, false));
        assert!((1..3).contains_ext(2, true, false));
        assert!(!(1..3).contains_ext(3, true, false));
        assert!(!(1..3).contains_ext(4, true, false));

        assert!(!(1..3).contains_ext(0, false, true));
        assert!(!(1..3).contains_ext(1, false, true));
        assert!((1..3).contains_ext(2, false, true));
        assert!((1..3).contains_ext(3, false, true));
        assert!(!(1..3).contains_ext(4, false, true));

        assert!(!(1..3).contains_ext(0, true, true));
        assert!((1..3).contains_ext(1, true, true));
        assert!((1..3).contains_ext(2, true, true));
        assert!((1..3).contains_ext(3, true, true));
        assert!(!(1..3).contains_ext(4, true, true));
    }

    #[test]
    fn contains_ext_empty() {
        assert!(!(1..1).contains_ext(0, false, false));
        assert!(!(1..1).contains_ext(1, false, false));
        assert!(!(1..1).contains_ext(2, false, false));

        assert!(!(1..1).contains_ext(0, true, false));
        assert!(!(1..1).contains_ext(1, true, false));
        assert!(!(1..1).contains_ext(2, true, false));

        assert!(!(1..1).contains_ext(0, false, true));
        assert!(!(1..1).contains_ext(1, false, true));
        assert!(!(1..1).contains_ext(2, false, true));

        assert!(!(1..1).contains_ext(0, true, true));
        assert!((1..1).contains_ext(1, true, true));
        assert!(!(1..1).contains_ext(2, true, true));
    }
}
