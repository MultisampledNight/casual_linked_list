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

    pub fn iter(&self) -> iter::Iter<'a, T> {
        // SAFETY: 'a is the lifetime of the whole list,
        //         and `Iter` is bound by it --- will not ever be leaked
        let start = unsafe { self.start.as_ref().next };
        let end = unsafe { self.end.as_ref().prev };
        iter::Iter::new(start, end)
    }

    /// Appends the given item to the end of the list, should complete in _O_(_1_).
    pub fn push_front(&mut self, item: T) {
        // SAFETY: `self.start` is never invalidated and initialized only in `Self::new`
        unsafe {
            self.insert_in_dir(self.start, Direction::After, item);
        }
    }

    /// Inserts the given item before the first element of the list, should complete in _O_(_1_).
    pub fn push_back(&mut self, item: T) {
        // SAFETY: `self.end` is never invalidated and initialized only in `Self::new`
        unsafe {
            self.insert_in_dir(self.end, Direction::Before, item);
        }
    }

    /// Inserts the given element in the given direction of the anchor element.
    ///
    /// # Safety
    ///
    /// `anchor` must be a valid, well-aligned pointer to a list element owned by this list.
    ///
    /// # Panics
    ///
    /// Panics if `anchor` is the sentinel tail or head element, and `direction` points
    /// away from the rest of the list.
    unsafe fn insert_in_dir(
        &mut self,
        anchor: ElementPointer<'a, T>,
        direction: Direction,
        item: T,
    ) {
        let (Some(mut prev_for_new), Some(mut next_for_new)) =
            retrieve_paired_elements(anchor, Pair::AnchorAnd(direction))
        else {
            panic!("tried to insert element in impossible relation to sentinel element");
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

    /// Removes the element at the beginning of the list, should complete in _O_(_1_).
    pub fn pop_front(&mut self) -> Option<T> {
        // SAFETY: Same as `Self::push_front`,
        //         additionally `Head.next` is only changed by `Element::set_next`,
        //         where the caller has to uphold its unsafe contract.
        if self.is_empty() {
            return None;
        }

        unsafe {
            let first = self.start.as_ref().next;
            Some(self.pop(first))
        }
    }

    /// Removes the element at the end of the list, should complete in _O_(_1_).
    pub fn pop_back(&mut self) -> Option<T> {
        // SAFETY: Same as `Self::push_back`.
        //         additionally `Tail.prev` is only changed by `Element::set_prev`,
        //         where the caller has to uphold its unsafe contract.
        if self.is_empty() {
            return None;
        }

        unsafe {
            let last = self.end.as_ref().prev;
            Some(self.pop(last))
        }
    }

    /// Removes the given element by first deallocating the node, then unlinking it.
    ///
    /// # Safety
    ///
    /// `ele` must be a valid, well-aligned pointer to a list element owned by this list.
    ///
    /// # Panics
    ///
    /// Panics if `ele` is the sentinel head or tail element.
    unsafe fn pop(&mut self, ele: ElementPointer<'a, T>) -> T {
        let (Some(mut before_ele), Some(mut after_ele)) = retrieve_paired_elements(ele, Pair::Surrounding)
        else {
            panic!("tried to pop sentinel head or tail");
        };

        before_ele.as_mut().set_next(after_ele);
        after_ele.as_mut().set_prev(before_ele);

        self.len -= 1;
        let reboxed = Box::from_raw(ele.as_ptr());
        reboxed.into_data().unwrap()
    }
}

enum Direction {
    Before,
    After,
}

enum Pair {
    AnchorAnd(Direction),
    Surrounding,
}

/// Retrieves the given pair in relation to the given anchor list element. The returned tuple
/// refers to a pair of `(left, right)`, in terms where "next" is "right-hand". If the relative
/// element is inaccessible due to the anchor being the last/first element, it'll be `None`.
///
/// # Safety
///
/// `anchor` must be a valid, well-aligned pointer to a list element.
unsafe fn retrieve_paired_elements<'a, T: 'a>(
    anchor: ElementPointer<'a, T>,
    which: Pair,
) -> (Option<ElementPointer<'a, T>>, Option<ElementPointer<'a, T>>) {
    match which {
        Pair::AnchorAnd(Direction::Before) => {
            let ele_before_anchor = anchor.as_ref().prev();
            (ele_before_anchor, Some(anchor))
        }
        Pair::AnchorAnd(Direction::After) => {
            let ele_after_anchor = anchor.as_ref().next();
            (Some(anchor), ele_after_anchor)
        }
        Pair::Surrounding => {
            let ele_before_anchor = anchor.as_ref().prev();
            let ele_after_anchor = anchor.as_ref().next();
            (ele_before_anchor, ele_after_anchor)
        }
    }
}

fn allocate<T>(item: T) -> NonNull<T> {
    let ptr = Box::into_raw(Box::new(item));
    // SAFETY: `Box::into_raw` always returns a non-null pointer according to the docs
    unsafe { NonNull::new_unchecked(ptr) }
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
        // SAFETY: boxes are only allocated in either `Self::insert_in_dir` or in `Self::new`,
        //         but either are never exposed
        //         nodes from `Self::push_*` _are_ deallocated with `Self::pop`,
        //         but are also unlinked and made inaccessible
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
    fn into_data(self: Box<Self>) -> Option<T>;
    fn prev(&self) -> Option<ElementPointer<'a, T>>;
    fn next(&self) -> Option<ElementPointer<'a, T>>;

    /// Sets the pointer to the previous element to the given pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is a pointer to a valid node, and that the
    /// previous node is appropiately dropped if not used otherwise.
    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>);

    /// Sets the pointer to the next element to the given pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is a pointer to a valid node, and that the
    /// previous node is appropiately dropped if not used otherwise.
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

    fn into_data(self: Box<Self>) -> Option<T> {
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

    fn into_data(self: Box<Self>) -> Option<T> {
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

    fn into_data(self: Box<Self>) -> Option<T> {
        Some(self.data)
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
