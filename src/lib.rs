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
use rand::{Rng, RngExt as _};

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

impl<T: HasPriority + Clone, R: Clone> Clone for RandomPriorityBag<T, R>
where
    T::Priority: Clone,
{
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

    pub fn shrink_to_fit(&mut self) {
        self.elems.shrink_to_fit();
        self.group_ends.shrink_to_fit();
    }

    pub fn reserve(&mut self, additional_elements: usize, additional_priorities: usize) {
        self.elems.reserve(additional_elements);
        self.group_ends.reserve(additional_priorities);
    }

    pub fn reserve_exact(&mut self, additional_elements: usize, additional_priorities: usize) {
        self.elems.reserve_exact(additional_elements);
        self.group_ends.reserve_exact(additional_priorities);
    }

    pub const fn len(&self) -> usize {
        self.elems.len()
    }

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

#[cfg(test)]
mod tests {}
