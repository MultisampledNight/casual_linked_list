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

use std::marker::PhantomData;

use crate::MaybePointer;

/// Immutable edition.
pub struct Cursor<'a, T> {
    _ele: MaybePointer<T>,
    _index: usize,
    _bound_to_list: PhantomData<&'a ()>,
}
