use crate::{RandomPriorityBag, has_priority::HasPriority};
use parking_lot::Mutex;
use rand::{prelude::*, seq::index};
use std::{
    iter::{FusedIterator, Rev},
    slice,
};

#[must_use]
pub struct ElementsIter<T: HasPriority, R: ?Sized> {
    rpb: RandomPriorityBag<T, R>,
}

impl<T: HasPriority, R: Rng> IntoIterator for RandomPriorityBag<T, R> {
    type Item = T;

    type IntoIter = ElementsIter<T, R>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ElementsIter { rpb: self }
    }
}

impl<T: HasPriority, R> ElementsIter<T, R> {
    #[inline]
    #[must_use]
    pub fn into_random_priority_bag(self) -> RandomPriorityBag<T, R> {
        self.rpb
    }
}

impl<T: HasPriority, R: ?Sized + Rng> Iterator for ElementsIter<T, R> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.rpb.pop_best()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T: HasPriority, R: ?Sized + Rng> DoubleEndedIterator for ElementsIter<T, R> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rpb.pop_worst()
    }
}

impl<T: HasPriority, R: ?Sized + Rng> ExactSizeIterator for ElementsIter<T, R> {
    #[inline]
    fn len(&self) -> usize {
        self.rpb.len()
    }
}

impl<T: HasPriority, R: ?Sized + Rng> FusedIterator for ElementsIter<T, R> {}

#[must_use]
pub struct ElementsIterRef<'a, T: HasPriority, R: ?Sized> {
    current_group_elems: &'a [T],
    current_group_ixs: index::IndexVecIntoIter,
    remaining_elems: &'a [T],
    remaining_group_ends: Rev<slice::Iter<'a, (T::Priority, usize)>>,
    rng: &'a Mutex<R>,
}

impl<'a, T: HasPriority, R: ?Sized> Clone for ElementsIterRef<'a, T, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            current_group_elems: self.current_group_elems,
            current_group_ixs: self.current_group_ixs.clone(),
            remaining_elems: self.remaining_elems,
            remaining_group_ends: self.remaining_group_ends.clone(),
            rng: self.rng,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.current_group_elems = source.current_group_elems;
        self.current_group_ixs.clone_from(&source.current_group_ixs);
        self.remaining_elems = source.remaining_elems;
        self.remaining_group_ends
            .clone_from(&source.remaining_group_ends);
        self.rng = source.rng
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> IntoIterator for &'a RandomPriorityBag<T, R> {
    type Item = &'a T;

    type IntoIter = ElementsIterRef<'a, T, R>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let rng = &self.rng;
        ElementsIterRef {
            current_group_elems: &[],
            current_group_ixs: index::sample(&mut rng.lock(), 0, 0).into_iter(),
            remaining_elems: &self.elems,
            remaining_group_ends: self.group_ends.iter().rev(),
            rng,
        }
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> Iterator for ElementsIterRef<'a, T, R> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_ix) = self.current_group_ixs.next() {
            self.current_group_elems.get(next_ix)
        } else if let Some(&(_, next_end)) = self.remaining_group_ends.next()
            && let Some(new_group) = self.remaining_elems.split_off(..next_end)
        {
            assert!(!new_group.is_empty());
            let new_group_len = new_group.len();
            self.current_group_elems = new_group;
            self.current_group_ixs =
                index::sample(&mut self.rng.lock(), new_group_len, new_group_len).into_iter();
            self.current_group_elems
                .get(self.current_group_ixs.next().unwrap())
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> ExactSizeIterator for ElementsIterRef<'a, T, R> {
    #[inline]
    fn len(&self) -> usize {
        self.current_group_ixs.len() + self.remaining_elems.len()
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> FusedIterator for ElementsIterRef<'a, T, R> {}

#[must_use]
pub struct ElementsIterMut<'a, T: HasPriority, R: ?Sized> {
    current_group_elems: &'a mut [T],
    remaining_elems: &'a mut [T],
    remaining_group_ends: Rev<slice::Iter<'a, (T::Priority, usize)>>,
    rng: &'a mut R,
}

impl<'a, T: HasPriority, R: ?Sized + Rng> IntoIterator for &'a mut RandomPriorityBag<T, R> {
    type Item = &'a mut T;

    type IntoIter = ElementsIterMut<'a, T, R>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let rng = self.rng.get_mut();
        ElementsIterMut {
            current_group_elems: &mut [],
            remaining_elems: &mut self.elems,
            remaining_group_ends: self.group_ends.iter().rev(),
            rng,
        }
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> Iterator for ElementsIterMut<'a, T, R> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_val) = self.current_group_elems.split_off_last_mut() {
            Some(next_val)
        } else if let Some(&(_, next_end)) = self.remaining_group_ends.next()
            && let Some(new_group) = self.remaining_elems.split_off_mut(..next_end)
        {
            assert!(!new_group.is_empty());
            new_group.shuffle(self.rng);
            self.current_group_elems = new_group;
            self.current_group_elems.split_off_last_mut()
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> ExactSizeIterator for ElementsIterMut<'a, T, R> {
    #[inline]
    fn len(&self) -> usize {
        self.current_group_elems.len() + self.remaining_elems.len()
    }
}

impl<T, R> FromIterator<T> for RandomPriorityBag<T, R>
where
    T: HasPriority,
    R: Default,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let elems = Vec::from_iter(iter);
        Self::from_vec(elems, R::default())
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> FusedIterator for ElementsIterMut<'a, T, R> {}

impl<T, R> Extend<T> for RandomPriorityBag<T, R>
where
    T: HasPriority,
    R: ?Sized,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let old_len = self.elems.len();
        self.elems.extend(iter);
        let increase = self.elems.len() - old_len;

        if increase > 0 {
            // only do work if needed
            self.elems.sort_unstable_by_key(T::get_priority);
            self.group_ends.reserve(increase.isqrt()); // heuristic size increase
            self.group_ends.clear();
            self.elems.iter().enumerate().for_each(|(ix, elem)| {
                let prio = elem.get_priority();
                if let Some((existing_prio, existing_ix)) = self.group_ends.last_mut()
                    && *existing_prio == prio
                {
                    *existing_ix = ix;
                } else {
                    self.group_ends.push((prio, ix));
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {}
