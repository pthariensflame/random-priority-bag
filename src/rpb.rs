use crate::has_priority::HasPriority;
use parking_lot::Mutex;
use rand::prelude::*;
use std::iter::FusedIterator;

/// Time complexities are stated in the variables:
/// - *e*: number of elements
/// - *p*: number of distinct priorities
///
/// `drop`` complexity: always O(*e*+*p*)
pub struct RandomPriorityBag<T: HasPriority, R: ?Sized> {
    pub(crate) group_ends: Vec<(T::Priority, usize)>,
    pub(crate) elems: Vec<T>,
    pub(crate) rng: Mutex<R>,
}

impl<T: HasPriority, R> RandomPriorityBag<T, R> {
    /// Complexity: always O(1)
    #[inline]
    #[must_use]
    pub const fn new(rng: R) -> Self {
        Self {
            group_ends: Vec::new(),
            elems: Vec::new(),
            rng: Mutex::new(rng),
        }
    }

    // Complexity: worst-case O(*e* log(*e*))
    #[must_use]
    fn reconstruct_from_elems(mut elems: Vec<T>, rng: Mutex<R>) -> Self {
        let mut group_ends = Vec::with_capacity(T::estimate_distinct_priorities(elems.len()));
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
            rng,
        }
    }

    /// Complexity: worst-case O(*e* log(*e*))
    #[inline]
    #[must_use]
    pub fn from_vec<V: Into<Vec<T>>>(vec: V, rng: R) -> Self {
        Self::reconstruct_from_elems(vec.into(), Mutex::new(rng))
    }
}

impl<T: HasPriority, R: Default> Default for RandomPriorityBag<T, R> {
    /// Complexity: always O(1)
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
    /// Complexity: always O(1) [assuming rng forking is O(1)]
    #[inline]
    fn clone(&self) -> Self {
        Self {
            group_ends: self.group_ends.clone(),
            elems: self.elems.clone(),
            rng: Mutex::new(self.rng.lock().fork()),
        }
    }

    /// Complexity: always O(1) [assuming rng forking is O(1)]
    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.group_ends.clone_from(&source.group_ends);
        self.elems.clone_from(&source.elems);
        *self.rng.get_mut() = source.rng.lock().fork();
    }
}

impl<T: HasPriority, R: ?Sized> RandomPriorityBag<T, R> {
    /// Complexity: always O(1)
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        let res = self.elems.is_empty();
        assert!(self.group_ends.is_empty() == res);
        res
    }

    /// Complexity: dependent on memory allocator
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.elems.shrink_to_fit();
        self.group_ends.shrink_to_fit();
    }

    /// Complexity: dependent on memory allocator
    #[inline]
    pub fn shrink_to(&mut self, elements: usize, priorities: usize) {
        self.elems.shrink_to(elements);
        self.group_ends.shrink_to(priorities);
    }

    /// Complexity: dependent on memory allocator
    #[inline]
    pub fn reserve(&mut self, additional_elements: usize, additional_priorities: usize) {
        self.elems.reserve(additional_elements);
        self.group_ends.reserve(additional_priorities);
    }

    /// Complexity: dependent on memory allocator
    #[inline]
    pub fn reserve_exact(&mut self, additional_elements: usize, additional_priorities: usize) {
        self.elems.reserve_exact(additional_elements);
        self.group_ends.reserve_exact(additional_priorities);
    }

    /// Complexity: always O(1)
    #[inline]
    pub fn clear(&mut self) {
        self.elems.clear();
        self.group_ends.clear();
    }

    /// Complexity: always O(1)
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.elems.len()
    }

    /// Complexity: always O(1)
    #[inline]
    #[must_use]
    pub const fn priorities_len(&self) -> usize {
        self.group_ends.len()
    }

    /// Complexity: always O(1)
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.elems.capacity()
    }

    /// Complexity: always O(1)
    #[inline]
    #[must_use]
    pub const fn priorities_capacity(&self) -> usize {
        self.group_ends.capacity()
    }

    /// Initial complexity: always O(1)
    /// Complexity per iteration: always O(1)
    #[inline]
    #[must_use]
    pub fn priorities(
        &self,
    ) -> impl ExactSizeIterator<Item = &T::Priority> + DoubleEndedIterator + FusedIterator {
        self.group_ends.iter().map(|(prio, _)| prio)
    }
}

impl<T: HasPriority, R: ?Sized + Rng> RandomPriorityBag<T, R> {
    /// Complexity: always O(*1*)
    pub fn pop_best(&mut self) -> Option<T> {
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

    /// Complexity: always O(*p*)
    pub fn pop_worst(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let first_end = self.group_ends.first().unwrap().1;

        let pos = self.rng.lock().random_range(0..first_end);
        self.elems.swap(pos, first_end - 1);

        if self.group_ends.len() >= 2 {
            for i in 1..self.group_ends.len() {
                self.elems
                    .swap(self.group_ends[i - 1].1 - 1, self.group_ends[i].1 - 1);
                self.group_ends[i - 1].1 -= 1
            }
        }
        self.group_ends.last_mut().unwrap().1 -= 1;

        if self.group_ends.first().unwrap().1 == 0 {
            self.group_ends.remove(0);
        }

        self.elems.pop()
    }

    /// Complexity: worst-case O(*p*)
    pub fn push(&mut self, new_elem: T) {
        let new_elem_priority = new_elem.get_priority();

        let selected_group_pos = self
            .group_ends
            .partition_point(|(group_priority, _)| new_elem_priority <= *group_priority);

        self.elems.push(new_elem);

        if let Some((group_priority, _)) = self.group_ends.get(selected_group_pos)
            && *group_priority == new_elem_priority
        {
            // group already exists, insert at end
            // this loop will only run any iterations if there are at least 2 existing groups
            for i in (selected_group_pos + 1..self.group_ends.len()).rev() {
                self.elems
                    .swap(self.group_ends[i - 1].1, self.group_ends[i].1);
                self.group_ends[i].1 += 1;
            }
            self.group_ends[selected_group_pos].1 += 1;
        } else {
            // new group needed
            let mut new_group_end = self.elems.len();

            // this loop will only run any iterations if there was at least 1 previously existing group
            for i in (selected_group_pos..self.group_ends.len()).rev() {
                self.elems
                    .swap(self.group_ends[i - 1].1, self.group_ends[i].1);
                self.group_ends[i].1 += 1;
                new_group_end = self.group_ends[i - 1].1 + 1;
            }

            self.group_ends
                .insert(selected_group_pos, (new_elem_priority, new_group_end));
        }
    }

    /// Complexity: always O(*e*)
    pub fn reshuffle(&mut self) {
        let mut rng = self.rng.lock();
        let mut prev_end = 0;
        self.group_ends.iter().for_each(|&(_, curr_end)| {
            self.elems[prev_end..curr_end].shuffle(&mut rng);
            prev_end = curr_end;
        });
    }

    /// Initial complexity: always O(1)
    /// Complexity per iteration: amortized O(1), worst-case O(*e*÷*p*)
    #[inline]
    pub fn iter(&self) -> crate::iter::ElementsIterRef<'_, T, R> {
        self.into_iter()
    }

    /// Initial complexity: always O(1)
    /// Complexity per iteration: amortized O(1), worst-case O(*e*÷*p*)
    #[inline]
    pub fn iter_rev(&self) -> crate::iter::ElementsIterRefRev<'_, T, R> {
        let rng = &self.rng;
        let remaining_group_ends = self
            .group_ends
            .split_last()
            .map(|(_, remaining)| remaining)
            .unwrap_or(&[]);
        crate::iter::ElementsIterRefRev {
            current_group_elems: &[],
            current_group_ixs: rand::seq::index::sample(&mut rng.lock(), 0, 0).into_iter(),
            remaining_elems: &self.elems,
            remaining_group_ends,
            exhausted_groups_len: 0,
            rng,
        }
    }

    /// Initial complexity: always O(1)
    /// Complexity per iteration: amortized O(1), worst-case O(*e*÷*p*)
    #[inline]
    pub fn iter_mut(&mut self) -> crate::iter::ElementsIterMut<'_, T, R> {
        self.into_iter()
    }

    /// Initial complexity: always O(1)
    /// Complexity per iteration: amortized O(1), worst-case O(*e*÷*p*)
    #[inline]
    pub fn iter_mut_rev(&mut self) -> crate::iter::ElementsIterMutRev<'_, T, R> {
        let rng = self.rng.get_mut();
        let remaining_group_ends = self
            .group_ends
            .split_last()
            .map(|(_, remaining)| remaining)
            .unwrap_or(&[]);
        crate::iter::ElementsIterMutRev {
            current_group_elems: &mut [],
            remaining_elems: &mut self.elems,
            remaining_group_ends,
            exhausted_groups_len: 0,
            rng,
        }
    }
}

impl<T: HasPriority, R: Rng> RandomPriorityBag<T, R> {
    /// Complexity: always O(*e*)
    #[must_use]
    pub fn into_vec<V>(self) -> Vec<T> {
        let Self {
            group_ends,
            mut elems,
            rng,
        } = self;
        let mut prior_end = 0;
        group_ends.iter().for_each(|&(_, group_end)| {
            elems[prior_end..group_end].shuffle(&mut rng.lock());
            prior_end = group_end;
        });
        elems
    }

    /// Complexity: worst-case O(*e* log(*e*))
    #[must_use]
    pub fn map<U, F>(self, f: F) -> RandomPriorityBag<U, R>
    where
        U: HasPriority,
        F: FnMut(T) -> U,
    {
        let Self {
            group_ends: _,
            elems,
            rng,
        } = self;
        let mapped_elems = elems.into_iter().map(f).collect();
        RandomPriorityBag::reconstruct_from_elems(mapped_elems, rng)
    }
}

#[cfg(test)]
mod tests {}
