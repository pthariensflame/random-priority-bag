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

impl<T> HasPriority for std::sync::Arc<T>
where
    T: ?Sized + HasPriority,
{
    type Priority = T::Priority;

    fn get_priority(&self) -> Self::Priority {
        T::get_priority(self)
    }
}

#[cfg(test)]
mod tests {}
