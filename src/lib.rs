// Copyright 2026 Laine Taffin Altman
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

use parking_lot::Mutex;
use rand::{prelude::*, seq::index};
use std::{iter::Rev, slice};

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

pub struct RandomPriorityBag<T: HasPriority, R: ?Sized> {
    group_ends: Vec<(T::Priority, usize)>,
    elems: Vec<T>,
    rng: Mutex<R>,
}

impl<T: HasPriority, R> RandomPriorityBag<T, R> {
    #[inline]
    #[must_use]
    pub const fn new(rng: R) -> Self {
        Self {
            group_ends: Vec::new(),
            elems: Vec::new(),
            rng: Mutex::new(rng),
        }
    }
}

impl<T: HasPriority, R: Default> Default for RandomPriorityBag<T, R> {
    #[inline]
    fn default() -> Self {
        Self {
            group_ends: Vec::new(),
            elems: Vec::new(),
            rng: Mutex::default(),
        }
    }
}

impl<T: HasPriority<Priority: Clone> + Clone, R: Rng + SeedableRng> Clone
    for RandomPriorityBag<T, R>
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            group_ends: self.group_ends.clone(),
            elems: self.elems.clone(),
            rng: Mutex::new(self.rng.lock().fork()),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.group_ends.clone_from(&source.group_ends);
        self.elems.clone_from(&source.elems);
        *self.rng.get_mut() = source.rng.lock().fork();
    }
}

impl<T: HasPriority, R: ?Sized> RandomPriorityBag<T, R> {
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        let res = self.elems.is_empty();
        assert!(self.group_ends.is_empty() == res);
        res
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.elems.shrink_to_fit();
        self.group_ends.shrink_to_fit();
    }

    #[inline]
    pub fn reserve(&mut self, additional_elements: usize, additional_priorities: usize) {
        self.elems.reserve(additional_elements);
        self.group_ends.reserve(additional_priorities);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_elements: usize, additional_priorities: usize) {
        self.elems.reserve_exact(additional_elements);
        self.group_ends.reserve_exact(additional_priorities);
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.elems.len()
    }

    #[inline]
    #[must_use]
    pub const fn num_priority_groups(&self) -> usize {
        self.group_ends.len()
    }
}

impl<T: HasPriority, R: ?Sized + Rng> RandomPriorityBag<T, R> {
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let final_end = self.group_ends.last().unwrap().1;
        let preceding_end = if let Some(preceding_end_ix) = self.group_ends.len().checked_sub(2) {
            self.group_ends[preceding_end_ix].1
        } else {
            0
        };

        let pos = self.rng.lock().random_range(preceding_end..final_end);
        let best = self.elems.swap_remove(pos);

        self.group_ends.pop_if(|(_, position)| {
            *position -= 1;
            *position == preceding_end
        });

        Some(best)
    }

    pub fn push(&mut self, new_elem: T) {
        let new_elem_priority = new_elem.get_priority();

        let group_pos = self
            .group_ends
            .partition_point(|(group_priority, _)| new_elem_priority <= *group_priority);

        if let Some(&(ref group_priority, group_end)) = self.group_ends.get(group_pos)
            && *group_priority == new_elem_priority
        {
            // including current (existing) group
            if let Some(later_groups) = self.group_ends.get_mut(group_pos..) {
                later_groups.iter_mut().for_each(|(_, later_group_end)| {
                    *later_group_end += 1;
                });
            }

            self.elems.insert(group_end, new_elem);
        } else {
            let new_elem_pos = self
                .group_ends
                .get(group_pos)
                .map_or(self.elems.len(), |(_, pos)| *pos);
            self.group_ends
                .insert(group_pos, (new_elem_priority, new_elem_pos));

            // excluding current (new) group
            if let Some(later_groups) = self.group_ends.get_mut((group_pos + 1)..) {
                later_groups.iter_mut().for_each(|(_, later_group_end)| {
                    *later_group_end += 1;
                });
            }

            self.elems.insert(new_elem_pos, new_elem);
        }
    }
}

pub struct ElementsIter<T: HasPriority, R: ?Sized> {
    rpb: RandomPriorityBag<T, R>,
}

impl<T: HasPriority<Priority: Clone> + Clone, R: Rng + SeedableRng> Clone for ElementsIter<T, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            rpb: self.rpb.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.rpb.clone_from(&source.rpb)
    }
}

impl<T: HasPriority, R: Rng> IntoIterator for RandomPriorityBag<T, R> {
    type Item = T;

    type IntoIter = ElementsIter<T, R>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ElementsIter { rpb: self }
    }
}

impl<T: HasPriority, R: ?Sized + Rng> Iterator for ElementsIter<T, R> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.rpb.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T: HasPriority, R: ?Sized + Rng> ExactSizeIterator for ElementsIter<T, R> {
    #[inline]
    fn len(&self) -> usize {
        self.rpb.len()
    }
}

impl<T: HasPriority, R> ElementsIter<T, R> {
    pub fn into_random_priority_bag(self) -> RandomPriorityBag<T, R> {
        self.rpb
    }
}

pub struct ElementsIterRef<'a, T: HasPriority, R: ?Sized> {
    current_group_elems: &'a [T],
    current_group_ixs: index::IndexVecIntoIter,
    remaining_elems: &'a [T],
    remaining_group_ends: Rev<slice::Iter<'a, (T::Priority, usize)>>,
    rng: &'a Mutex<R>,
}

impl<'a, T: HasPriority, R: Clone> Clone for ElementsIterRef<'a, T, R> {
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

impl<'a, T: HasPriority, R: Rng + SeedableRng> IntoIterator for &'a RandomPriorityBag<T, R> {
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

pub struct ElementsIterMut<'a, T: HasPriority, R: ?Sized> {
    current_group_elems: &'a mut [T],
    remaining_elems: &'a mut [T],
    remaining_group_ends: Rev<slice::Iter<'a, (T::Priority, usize)>>,
    rng: &'a mut R,
}

impl<'a, T: HasPriority, R: Rng + SeedableRng> IntoIterator for &'a mut RandomPriorityBag<T, R> {
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
        let mut elems = Vec::from_iter(iter);
        let mut group_ends = Vec::with_capacity(elems.len().isqrt()); // heuristic size
        elems.sort_unstable_by_key(T::get_priority);
        elems.iter().enumerate().for_each(|(ix, elem)| {
            let prio = elem.get_priority();
            if let Some((existing_prio, existing_ix)) = group_ends.last_mut()
                && *existing_prio == prio
            {
                *existing_ix = ix;
            } else {
                group_ends.push((prio, ix));
            }
        });
        Self {
            group_ends,
            elems,
            rng: Mutex::default(),
        }
    }
}

#[cfg(test)]
mod tests {}
