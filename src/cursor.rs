//! Cursors into the linked list, movable back and forth.
//!
//! This mimics the cursor design of [`std`]'s [`LinkedList`], even though its documentation is
//! out-of-sync with the implementation at the time of writing.
//!
//! A cursor is a pointer to one element in a linked list. This pointer can move one element forward
//! with [`move_front`] and one element backward with [`move_back`]. Similarly, the node the
//! pointer currently points to (the "pointee") can be extracted in expected _O_(1) using
//! [`current`].
//!
//! If you move the pointer past the list (by using [`move_back`] while on the last element) or
//! before the list (by using [`move_front`] while on the first element), the pointer is placed on
//! the opposite end of the list.
//!
//! One last thing, there are multiple cursor types:
//!
//! - [`UndistortedCursor`] and [`Cursor`] are _immutable_, and cannot change the list, only observe
//!   it. There can exist **multiple** of these at any given timepoint, given that no mutable
//!   cursor/reference exists.
//! - [`UndistortedCursorMut`] and [`CursorMut`] are _mutable_, allowing modification of the list.
//!   However, there can exist only **one** these at any given timepoint, thus they conflict with
//!   the immutable ones.
//! - The "normal" ones **without** prefix respect the calls made to [`ReversibleList::reverse`],
//!   while the ones **with** `UndistortedCursor` ignore them completely (which might cause
//!   elements to end up at unexpected positions).
//!
//! [`LinkedList`]: std::collections::LinkedList
//! [`move_back`]: Cursor::move_back
//! [`move_front`]: Cursor::move_front
//! [`current`]: Cursor::current
//! [`index`]: Cursor::index
//! [`None`]: Option::None

use std::cmp::{
    self,
    Ordering::{Equal, Greater, Less},
};

use crate::{MaybePointer, ReversibleList};

/// Immutable edition. **Ignores** any past calls to [`ReversibleList::reverse`], like
/// [`ReversibleList::undistorted_iter`], see its documentation for details.
pub struct UndistortedCursor<'a, T> {
    node: MaybePointer<T>,
    index: usize,
    list: &'a ReversibleList<T>,
}

macro_rules! impl_common_cursor {
    ($name:ident $($mut:ident)?) => {
        impl<'a, T: 'a> $name<'a, T> {
            /// # Safety
            ///
            /// `list.start` must be a valid pointer to the first list element.
            pub(crate) unsafe fn new_front(list: &'a $($mut)? ReversibleList<T>) -> Self {
                Self {
                    node: list.start,
                    index: 0,
                    list,
                }
            }

            /// # Safety
            ///
            /// `list.end` must be a valid pointer to the last list element.
            pub(crate) unsafe fn new_back(list: &'a $($mut)? ReversibleList<T>) -> Self {
                Self {
                    node: list.end,
                    index: list.len.saturating_sub(1),
                    list,
                }
            }

            /// Returns the data stored on the current node, or `None` if the list is empty. There is no
            pub fn current(&self) -> Option<&T> {
                // SAFETY: Delegated to the unsafe contract of `new_front`/`new_back`.
                self.node.map(|node| unsafe { &(*node.as_ptr()).data })
            }

            /// Makes this cursor look at the **previous** node in the list. If there is none, the cursor will
            /// point at the **end** of the list. Does nothing if the list is empty.
            pub fn move_prev(&mut self) {
                let Some(current) = self.node else {
                    return;
                };

                if self.index == 0 {
                    // currently at the start, wrap to the end
                    self.node = self.list.end;
                    self.index = self.list.len.saturating_sub(1);
                } else {
                    // somewhere in mid of the list
                    // SAFETY: Delegated to the unsafe contract of `new_front`/`new_back`.
                    self.node = unsafe { (*current.as_ptr()).prev };
                    self.index -= 1;
                }
            }

            /// Makes this cursor look at the **next** node in the list. If there is none, the cursor will
            /// point at the **beginning** of the list. Does nothing if the list is empty.
            pub fn move_next(&mut self) {
                let Some(current) = self.node else {
                    return;
                };

                if self.index == self.list.len.saturating_sub(1) {
                    // currently at the end, wrap to the start
                    self.node = self.list.start;
                    self.index = 0;
                } else {
                    // somewhere in mid of the list
                    // SAFETY: Delegated to the unsafe contract of `new_front`/`new_back`.
                    self.node = unsafe { (*current.as_ptr()).next };
                    self.index += 1;
                }
            }

            /// Moves this cursor `n` nodes backward. Note that wrapping behavior still applies.
            pub fn move_prev_n(&mut self, n: usize) {
                // filter out how many times we we really need to move
                let n = n % self.list.len;
                for _ in 0..n {
                    self.move_prev();
                }
            }

            /// Moves this cursor `n` nodes forward. Note that wrapping behavior still applies.
            pub fn move_next_n(&mut self, n: usize) {
                let n = n % self.list.len;
                for _ in 0..n {
                    self.move_next();
                }
            }

            /// Moves this cursor to the given absolute list index.
            pub fn move_to(&mut self, target_idx: usize) {
                // check if wrapping or going straight through the list is shorter
                let direct_distance = self.index.abs_diff(target_idx);
                let wrapping_distance = cmp::min(self.index, target_idx)
                    + cmp::max(self.index, target_idx).abs_diff(self.list.len);

                match (
                    self.index.cmp(&target_idx),
                    direct_distance.cmp(&wrapping_distance),
                ) {
                    (Less, Less | Equal) => self.move_next_n(direct_distance),
                    (Less, Greater) => self.move_prev_n(wrapping_distance),
                    (Greater, Less | Equal) => self.move_prev_n(direct_distance),
                    (Greater, Greater) => self.move_next_n(wrapping_distance),
                    (Equal, _) => (),
                }
            }
        }
    };
}

impl_common_cursor!(UndistortedCursor);

/// Mutable edition. **Ignores** any past calls to [`ReversibleList::reverse`], like
/// [`ReversibleList::undistorted_iter`], see its documentation for details.
pub struct UndistortedCursorMut<'a, T> {
    node: MaybePointer<T>,
    index: usize,
    list: &'a mut ReversibleList<T>,
}

impl_common_cursor!(UndistortedCursorMut mut);
