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

use crate::ElementPointer;

pub struct Iter<'a, T> {
    forward_node: ElementPointer<'a, T>,
    backward_node: ElementPointer<'a, T>,
    finished: bool,
}

enum Direction {
    Forward,
    Backward,
}

impl<'a, T: 'a> Iter<'a, T> {
    pub(crate) fn new(
        forward_start: ElementPointer<'a, T>,
        backward_start: ElementPointer<'a, T>,
    ) -> Self {
        Self {
            forward_node: forward_start,
            backward_node: backward_start,
            finished: false,
        }
    }

    fn next_in_dir(&mut self, direction: Direction) -> Option<&'a T> {
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
                old_node = unsafe { self.forward_node.as_ref() };
                self.forward_node = old_node.next()?;
            }
            Direction::Backward => {
                old_node = unsafe { self.backward_node.as_ref() };
                self.backward_node = old_node.prev()?;
            }
        };

        old_node.data()
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.next_in_dir(Direction::Forward)
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.next_in_dir(Direction::Backward)
    }
}
