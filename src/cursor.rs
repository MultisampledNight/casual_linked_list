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
//! before the list (by using [`move_front`] while on the first element), the pointer is placed
//! on a _ghost_ element. While on that ghost element,
//!
//! - [`current`] and [`index`] return [`None`].
//! - When moving **forward** while on that ghost element, the cursor is placed on the **start** of
//!   the list.
//! - When moving **backward** while on that ghost element, the the cursor is placed on the **end**
//!   of the list.
//!
//! One last thing, there are multiple cursor types:
//!
//! [`LinkedList`]: std::collections::LinkedList
//! [`move_back`]: Cursor::move_back
//! [`move_front`]: Cursor::move_front
//! [`current`]: Cursor::current
//! [`index`]: Cursor::index
//! [`None`]: Option::None

use crate::{MaybePointer, ReversibleList, Direction};

/// Immutable edition. **Ignores** any past calls to [`ReversibleList::reverse`], like
/// [`ReversibleList::undistorted_iter`], see its documentation for details.
pub struct UndistortedCursor<'a, T> {
    node: MaybePointer<T>,
    index: usize,
    list: &'a ReversibleList<T>,
}

impl<'a, T: 'a> UndistortedCursor<'a, T> {
    /// # Safety
    ///
    /// `list.start` must be a valid pointer to the first list element.
    pub(crate) unsafe fn new_front(list: &'a ReversibleList<T>) -> Self {
        Self {
            node: list.start,
            index: 0,
            list,
        }
    }

    /// # Safety
    ///
    /// `list.end` must be a valid pointer to the last list element.
    pub(crate) unsafe fn new_back(list: &'a ReversibleList<T>) -> Self {
        Self {
            node: list.end,
            index: list.len.saturating_sub(1),
            list,
        }
    }

    pub fn current(&self) -> Option<&T> {
        // SAFETY: Delegated to the unsafe contract of `new_front`/`new_back`.
        self.node.map(|node| unsafe { &(*node.as_ptr()).data })
    }

    /// Makes this cursor look at the **previous** node in the list. If there is none, the cursor
    /// will point at the _ghost_ node. If the current node is the _ghost_, the cursor will
    /// be point at the **end** of the list.
    pub fn move_prev(&mut self) {
        match self.node {
            None => {
                // currently at the ghost node => wrap to the end
                self.node = self.list.end;
                self.index = self.list.len.saturating_sub(1);
            },
            Some(current) => {
                // SAFETY: Delegated to the unsafe contract of `new_front`/`new_back`.
                self.node = unsafe { (*current.as_ptr()).prev };
            },
        }
    }

    /// Makes this cursor look at the **next** node in the list. If there is none, the cursor
    /// will point at the _ghost_ node. If the current node is the _ghost_, the cursor will
    /// be point at the **beginning** of the list.
    pub fn move_next(&mut self) {
        match self.node {
            None => {
                // currently at the ghost node => wrap to the end
                self.node = self.list.start;
                self.index = 0;
            },
            Some(current) => {
                // SAFETY: Delegated to the unsafe contract of `new_front`/`new_back`.
                self.node = unsafe { (*current.as_ptr()).next };
            },
        }
    }
}
