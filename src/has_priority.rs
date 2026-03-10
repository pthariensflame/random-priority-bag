use std::cmp::Reverse;

pub trait HasPriority {
    type Priority: Ord;

    /// Expected complexity: O(1)
    #[must_use]
    fn get_priority(&self) -> Self::Priority;

    /// Expected complexity: O(1)
    #[inline]
    #[must_use]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        num_elements.isqrt() // generic heuristic
    }
}

impl<T> HasPriority for &T
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<T> HasPriority for &mut T
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<T> HasPriority for Box<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<T> HasPriority for Option<T>
where
    T: HasPriority,
{
    type Priority = Option<T::Priority>;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        self.as_ref().map(T::get_priority)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<'a, T> HasPriority for std::borrow::Cow<'a, T>
where
    T: ?Sized + HasPriority + ToOwned,
{
    type Priority = T::Priority;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<T> HasPriority for std::rc::Rc<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<T> HasPriority for std::rc::Weak<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = Option<T::Priority>;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        self.upgrade().map(|this| this.get_priority())
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
            .cast_signed()
            .saturating_add(1)
            .cast_unsigned()
    }
}

impl<T> HasPriority for std::sync::Arc<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

impl<T> HasPriority for std::sync::Weak<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = Option<T::Priority>;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        self.upgrade().map(|this| this.get_priority())
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
            .cast_signed()
            .saturating_add(1)
            .cast_unsigned()
    }
}

impl<T, const N: usize> HasPriority for [T; N]
where
    T: HasPriority,
{
    type Priority = [T::Priority; N];

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        self.each_ref().map(T::get_priority)
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
            .cast_signed()
            .saturating_mul(N.cast_signed())
            .cast_unsigned()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
#[repr(transparent)]
pub struct SelfPriority<T: ?Sized>(pub T);

impl<T: ?Sized + ToOwned> SelfPriority<T> {
    #[inline]
    pub fn to_owned(&self) -> SelfPriority<T::Owned> {
        SelfPriority(self.0.to_owned())
    }
}

impl<T: ?Sized + ToOwned<Owned: Ord>> HasPriority for SelfPriority<T> {
    type Priority = T::Owned;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        self.0.to_owned()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct AttachedPriority<T: ?Sized, P> {
    pub priority: P,
    pub value: T,
}

impl<T: ?Sized + ToOwned, P: Clone> AttachedPriority<T, P> {
    #[inline]
    pub fn to_owned(&self) -> AttachedPriority<T::Owned, P> {
        AttachedPriority {
            priority: self.priority.clone(),
            value: self.value.to_owned(),
        }
    }
}

impl<T: ?Sized, P: Ord + Clone> HasPriority for AttachedPriority<T, P> {
    type Priority = P;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        self.priority.clone()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
#[repr(transparent)]
pub struct ReversedPriority<T: ?Sized>(pub T);

impl<T: ?Sized + ToOwned> ReversedPriority<T> {
    #[inline]
    pub fn to_owned(&self) -> ReversedPriority<T::Owned> {
        ReversedPriority(self.0.to_owned())
    }
}

impl<T: ?Sized + HasPriority> HasPriority for ReversedPriority<T> {
    type Priority = Reverse<T::Priority>;

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        Reverse(self.0.get_priority())
    }

    #[inline]
    fn estimate_distinct_priorities(num_elements: usize) -> usize {
        T::estimate_distinct_priorities(num_elements)
    }
}

#[cfg(test)]
mod tests {}
