use std::cmp::Reverse;

pub trait HasPriority {
    type Priority: Ord;

    fn get_priority(&self) -> Self::Priority;
}

impl<T> HasPriority for &T
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

impl<T> HasPriority for &mut T
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

impl<T> HasPriority for Box<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

impl<T> HasPriority for Option<T>
where
    T: HasPriority,
{
    type Priority = Option<T::Priority>;

    fn get_priority(&self) -> Self::Priority {
        self.as_ref().map(T::get_priority)
    }
}

impl<'a, T> HasPriority for std::borrow::Cow<'a, T>
where
    T: ?Sized + HasPriority + ToOwned,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

impl<T> HasPriority for std::rc::Rc<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

impl<T> HasPriority for std::rc::Weak<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = Option<T::Priority>;

    fn get_priority(&self) -> Self::Priority {
        self.upgrade().map(|this| this.get_priority())
    }
}

impl<T> HasPriority for std::sync::Arc<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

impl<T> HasPriority for std::sync::Weak<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = Option<T::Priority>;

    fn get_priority(&self) -> Self::Priority {
        self.upgrade().map(|this| this.get_priority())
    }
}

impl<T, const N: usize> HasPriority for [T; N]
where
    T: HasPriority,
{
    type Priority = [T::Priority; N];

    fn get_priority(&self) -> Self::Priority {
        self.each_ref().map(T::get_priority)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SelfPriority<T: ?Sized>(pub T);

impl<T: ?Sized + ToOwned> SelfPriority<T> {
    pub fn to_owned(&self) -> SelfPriority<T::Owned> {
        SelfPriority(self.0.to_owned())
    }
}

impl<T: ?Sized + ToOwned<Owned: Ord>> HasPriority for SelfPriority<T> {
    type Priority = T::Owned;

    fn get_priority(&self) -> Self::Priority {
        self.0.to_owned()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttachedPriority<T: ?Sized, P> {
    pub priority: P,
    pub value: T,
}

impl<T: ?Sized + ToOwned, P: Clone> AttachedPriority<T, P> {
    pub fn to_owned(&self) -> AttachedPriority<T::Owned, P> {
        AttachedPriority {
            priority: self.priority.clone(),
            value: self.value.to_owned(),
        }
    }
}

impl<T: ?Sized, P: Ord + Clone> HasPriority for AttachedPriority<T, P> {
    type Priority = P;

    fn get_priority(&self) -> Self::Priority {
        self.priority.clone()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ReversedPriority<T: ?Sized>(pub T);

impl<T: ?Sized + ToOwned> ReversedPriority<T> {
    pub fn to_owned(&self) -> ReversedPriority<T::Owned> {
        ReversedPriority(self.0.to_owned())
    }
}

impl<T: ?Sized + HasPriority> HasPriority for ReversedPriority<T> {
    type Priority = Reverse<T::Priority>;

    fn get_priority(&self) -> Self::Priority {
        Reverse(self.0.get_priority())
    }
}

#[cfg(test)]
mod tests {}
