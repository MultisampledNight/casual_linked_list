//! All iterator structs. You'll rarely want to use these directly.
//!
//! An ASCII diagram showing the initial situation for fun and profit:
//!
//! ```text
//!  forward_node               backward_node
//!            |                 |
//!            v                 v
//!   head <-> node <-> node <-> node <-> tail
//! ```
//!
//! The core idea is to, in one step of iteration (where "current" depends on `next` or `next_back`
//! being called):
//!
//! 1. If we're already finished, `None`
//! 2. If forward and backward running pointers are equal, mark this iterator as finished,
//!    but still yield *this* node
//! 3. Return the data of the current node
//! 4. Set the current node to the next node depending on the direction

use std::marker::PhantomData;

use crate::MaybePointer;

pub struct Iter<'list, T: 'list> {
    forward_node: MaybePointer<T>,
    backward_node: MaybePointer<T>,
    finished: bool,
    _bound_to_list: PhantomData<&'list ()>,
}

enum Direction {
    Forward,
    Backward,
}

impl<'list, T: 'list> Iter<'list, T> {
    pub(crate) unsafe fn new(
        forward_start: MaybePointer<T>,
        backward_start: MaybePointer<T>,
    ) -> Self {
        Self {
            forward_node: forward_start,
            backward_node: backward_start,
            finished: false,
            _bound_to_list: PhantomData,
        }
    }

    fn next_in_dir(&mut self, direction: Direction) -> Option<&'list T> {
        if self.finished {
            return None;
        }

        if self.forward_node == self.backward_node {
            // only return this node, then stop
            self.finished = true;
        }

        let old_node;

        match direction {
            Direction::Forward => {
                old_node = unsafe { self.forward_node?.as_ref() };
                self.forward_node = old_node.next;
            }
            Direction::Backward => {
                old_node = unsafe { self.backward_node?.as_ref() };
                self.backward_node = old_node.prev;
            }
        };

        Some(&old_node.data)
    }
}

impl<'list, T: 'list> Iterator for Iter<'list, T> {
    type Item = &'list T;

    fn next(&mut self) -> Option<&'list T> {
        self.next_in_dir(Direction::Forward)
    }
}

impl<'list, T: 'list> DoubleEndedIterator for Iter<'list, T> {
    fn next_back(&mut self) -> Option<&'list T> {
        self.next_in_dir(Direction::Backward)
    }
}
