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

#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use core::{iter::Rev, slice};
use rand::{prelude::*, seq::index};

pub trait HasPriority {
    type Priority: Ord;

    fn get_priority(&self) -> Self::Priority;
}

pub struct RandomPriorityBag<T: HasPriority, R: ?Sized> {
    group_ends: Vec<(T::Priority, usize)>,
    elems: Vec<T>,
    rng: R,
}

impl<T: HasPriority, R: Default> RandomPriorityBag<T, R> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            group_ends: Vec::new(),
            elems: Vec::new(),
            rng: R::default(),
        }
    }
}

impl<T: HasPriority, R> RandomPriorityBag<T, R> {
    #[inline]
    #[must_use]
    pub const fn with_rng(rng: R) -> Self {
        Self {
            group_ends: Vec::new(),
            elems: Vec::new(),
            rng,
        }
    }
}

impl<T: HasPriority, R: Default> Default for RandomPriorityBag<T, R> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HasPriority<Priority: Clone> + Clone, R: Clone> Clone for RandomPriorityBag<T, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            group_ends: self.group_ends.clone(),
            elems: self.elems.clone(),
            rng: self.rng.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.group_ends.clone_from(&source.group_ends);
        self.elems.clone_from(&source.elems);
        self.rng.clone_from(&source.rng);
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

        let pos = self.rng.random_range(preceding_end..final_end);
        let best = self.elems.swap_remove(pos);

        self.group_ends.pop_if(|(_, position)| {
            *position -= 1;
            *position == preceding_end
        });

        Some(best)
    }

    pub fn insert(&mut self, new_elem: T) {
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

impl<T: HasPriority<Priority: Clone> + Clone, R: Clone> Clone for ElementsIter<T, R> {
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

impl<'a, T: HasPriority, R: Rng> IntoIterator for RandomPriorityBag<T, R> {
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
    rng: R,
}

impl<'a, T: HasPriority, R: Clone> Clone for ElementsIterRef<'a, T, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            current_group_elems: self.current_group_elems,
            current_group_ixs: self.current_group_ixs.clone(),
            remaining_elems: self.remaining_elems,
            remaining_group_ends: self.remaining_group_ends.clone(),
            rng: self.rng.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.current_group_elems = source.current_group_elems;
        self.current_group_ixs.clone_from(&source.current_group_ixs);
        self.remaining_elems = source.remaining_elems;
        self.remaining_group_ends
            .clone_from(&source.remaining_group_ends);
        self.rng.clone_from(&source.rng);
    }
}

impl<'a, T: HasPriority, R: Rng + SeedableRng> IntoIterator for &'a RandomPriorityBag<T, R> {
    type Item = &'a T;

    type IntoIter = ElementsIterRef<'a, T, R>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ElementsIterRef {
            current_group_elems: todo!(),
            current_group_ixs: todo!(),
            remaining_elems: todo!(),
            remaining_group_ends: todo!(),
            rng: self.rng.fork(),
        }
    }
}

impl<'a, T: HasPriority, R: ?Sized + Rng> Iterator for ElementsIterRef<'a, T, R> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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

#[cfg(test)]
mod tests {}
