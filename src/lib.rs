#[cfg(test)]
mod tests;

pub mod iter;

use std::{fmt, mem, ptr::NonNull};

type ElementPointer<'a, T> = NonNull<dyn Element<'a, T> + 'a>;

pub struct ReversibleList<'a, T> {
    start: NonNull<Head<'a, T>>,
    end: NonNull<Tail<'a, T>>,
    len: usize,
}

impl<'a, T: 'a> ReversibleList<'a, T> {
    pub fn new() -> Self {
        // SAFETY: `Box::into_raw` from `allocate` guarantees non-nullness
        let (start, end) = unsafe {
            let start = allocate(Head {
                next: NonNull::<Tail<T>>::dangling(),
            });

            let end = allocate(Tail { prev: start });

            (*start.as_ptr()).next = end;
            (start, end)
        };

        Self { start, end, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_front(&mut self, item: T) {
        // SAFETY: `self.start` is never invalidated and initialized only in `Self::new`,
        // `head.next` is only set by `Element::set_next`, where its unsafe contract must be upheld
        unsafe {
            self.insert_in_dir(self.start.as_ref().next, Direction::Before, item);
        }
    }

    pub fn push_back(&mut self, item: T) {
        // SAFETY: `self.end` is never invalidated and initialized only in `Self::new`,
        // `tail.prev` is only set by `Element::set_prev`, where its unsafe contract must be upheld
        unsafe {
            self.insert_in_dir(self.end.as_ref().prev, Direction::After, item);
        }
    }

    /// # Safety
    ///
    /// `anchor` must be a valid, well-aligned pointer.
    unsafe fn insert_in_dir(
        &mut self,
        anchor: ElementPointer<'a, T>,
        direction: Direction,
        item: T,
    ) {
        let (mut prev_for_new, mut next_for_new) = match direction {
            Direction::Before => {
                let ele_before_anchor = anchor
                    .as_ref()
                    .prev()
                    .expect("tried to insert node before head");
                (ele_before_anchor, anchor)
            }
            Direction::After => {
                let ele_after_anchor = anchor
                    .as_ref()
                    .next()
                    .expect("tried to insert node after tail");
                (anchor, ele_after_anchor)
            }
        };

        let new_next = allocate(Node {
            data: item,
            prev: prev_for_new,
            next: next_for_new,
        });

        // SAFETY: Node.next and Node.prev are only mutated by `Element::set_next` and
        //         `Element::set_prev`, where the caller has to uphold the safety contract.
        //         `new_next` was just created from `Box::into_raw` in `allocate`,
        //         guaranteeing validity.
        unsafe {
            prev_for_new.as_mut().set_next(new_next);
            next_for_new.as_mut().set_prev(new_next);
        }

        self.len += 1;
    }

    pub fn iter(&self) -> iter::Iter<'a, T> {
        // SAFETY: 'a is the lifetime of the whole list,
        //         and `Iter` is bound by it --- will not ever be leaked
        let start = unsafe { self.start.as_ref().next };
        let end = unsafe { self.end.as_ref().prev };
        iter::Iter::new(start, end)
    }
}

enum Direction {
    Before,
    After,
}

impl<'a, T: fmt::Debug + 'a> fmt::Debug for ReversibleList<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T: 'a> Default for ReversibleList<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: 'a> Drop for ReversibleList<'a, T> {
    fn drop(&mut self) {
        // SAFETY: boxes are only allocated in either `Self::push_*` or in
        //         `Self::new`, but either are never exposed
        //         nodes from `Self::push_*` _are_ deallocated when removing, but
        //         won't show up in iteration
        let mut element = self.start.as_ptr() as *mut dyn Element<'a, T>;
        unsafe {
            while let Some(next) = (*element).next() {
                let old = mem::replace(&mut element, next.as_ptr());
                drop(Box::from_raw(old));
            }
            drop(Box::from_raw(element));
        }
    }
}

/// Rust is not suited at all for the composite pattern in such a low-level collection. But for fun
/// and profit I'll use it here anyway.
trait Element<'a, T: 'a> {
    fn data(&self) -> Option<&T>;
    fn data_mut(&mut self) -> Option<&mut T>;
    fn prev(&self) -> Option<ElementPointer<'a, T>>;
    fn next(&self) -> Option<ElementPointer<'a, T>>;

    /// Sets the pointer to the previous element of this element to the given pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is a pointer to a valid node, and that the
    /// previous node is appropiately dropped.
    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>);
    unsafe fn set_next(&mut self, ptr: ElementPointer<'a, T>);
}

struct Head<'a, T> {
    next: ElementPointer<'a, T>,
}

impl<'a, T: 'a> Element<'a, T> for Head<'a, T> {
    fn data(&self) -> Option<&T> {
        None
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        None
    }

    fn prev(&self) -> Option<ElementPointer<'a, T>> {
        None
    }

    fn next(&self) -> Option<ElementPointer<'a, T>> {
        Some(self.next)
    }

    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>) {
        panic!("head does not have any previous element, but tried to set {ptr:?}");
    }

    unsafe fn set_next(&mut self, ptr: ElementPointer<'a, T>) {
        self.next = ptr;
    }
}

struct Tail<'a, T> {
    prev: ElementPointer<'a, T>,
}

impl<'a, T: 'a> Element<'a, T> for Tail<'a, T> {
    fn data(&self) -> Option<&T> {
        None
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        None
    }

    fn prev(&self) -> Option<ElementPointer<'a, T>> {
        Some(self.prev)
    }

    fn next(&self) -> Option<ElementPointer<'a, T>> {
        None
    }

    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>) {
        self.prev = ptr;
    }

    unsafe fn set_next(&mut self, ptr: ElementPointer<'a, T>) {
        panic!("tail does not have any next element, but tried to set {ptr:?}");
    }
}

struct Node<'a, T> {
    data: T,
    prev: ElementPointer<'a, T>,
    next: ElementPointer<'a, T>,
}

impl<'a, T: 'a> Element<'a, T> for Node<'a, T> {
    fn data(&self) -> Option<&T> {
        Some(&self.data)
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        Some(&mut self.data)
    }

    fn prev(&self) -> Option<ElementPointer<'a, T>> {
        Some(self.prev)
    }

    fn next(&self) -> Option<ElementPointer<'a, T>> {
        Some(self.next)
    }

    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>) {
        self.prev = ptr;
    }

    unsafe fn set_next(&mut self, ptr: ElementPointer<'a, T>) {
        self.next = ptr;
    }
}

fn allocate<T>(item: T) -> NonNull<T> {
    let ptr = Box::into_raw(Box::new(item));
    // SAFETY: `Box::into_raw` always returns a non-null pointer according to the docs
    unsafe { NonNull::new_unchecked(ptr) }
}
