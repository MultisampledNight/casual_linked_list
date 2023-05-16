#[cfg(test)]
mod tests;

pub mod cursor;
pub mod iter;

use std::{cmp, fmt, ptr::NonNull, hash::{Hash, Hasher}};

type Pointer<T> = NonNull<Node<T>>;
type MaybePointer<T> = Option<Pointer<T>>;

pub struct ReversibleList<T> {
    start: MaybePointer<T>,
    end: MaybePointer<T>,
    len: usize,
}

struct Node<T> {
    data: T,
    prev: MaybePointer<T>,
    next: MaybePointer<T>,
}

impl<T> ReversibleList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            len: 0,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator through this list.
    pub fn iter(&self) -> iter::Iter<'_, T> {
        // SAFETY: '_ is the lifetime of this list reference
        //         and `Iter` is bound by it --- will not ever be leaked
        //         pointers are only mutated through `Self::insert_in_dir` and
        //         `Self::pop`, which both preserve a valid linked list
        unsafe { iter::Iter::new(self.start, self.end) }
    }

    /// Creates a cursor pointing at the **first** node in the list.
    pub fn cursor_front(&self) -> cursor::Cursor<'_, T> {
        // SAFETY: Same as `Self::iter`.
        unsafe { cursor::Cursor::new_front(self) }
    }

    /// Creates a cursor pointing at the **last** node in the list.
    pub fn cursor_back(&self) -> cursor::Cursor<'_, T> {
        // SAFETY: Same as `Self::iter`.
        unsafe { cursor::Cursor::new_back(self) }
    }

    /// Creates a cursor pointing at node with the given index in the list.
    pub fn cursor_at(&self, idx: usize) -> cursor::Cursor<'_, T> {
        // SAFETY: Same as `Self::iter`.
        let mut cursor = unsafe { cursor::Cursor::new_back(self) };
        cursor.move_to(idx);
        cursor
    }

    pub fn cursor_mut_front(&mut self) -> cursor::CursorMut<'_, T> {
        // SAFETY: Same as `Self::iter`.
        unsafe { cursor::CursorMut::new_front(self) }
    }

    pub fn cursor_mut_back(&mut self) -> cursor::CursorMut<'_, T> {
        // SAFETY: Same as `Self::iter`.
        unsafe { cursor::CursorMut::new_back(self) }
    }

    /// Appends the given item to the end of the list, should complete in _O_(1).
    pub fn push_front(&mut self, item: T) {
        // SAFETY: `self.start` is only mutated by `Self::insert_in_dir` or `Self::pop`,
        // which both preserve the validity of it.
        unsafe {
            self.insert_in_dir(self.start, Direction::Before, item);
        }
    }

    /// Inserts the given item before the first element of the list, should complete in _O_(1).
    pub fn push_back(&mut self, item: T) {
        // SAFETY: `self.end` is only mutated by `Self::insert_in_dir` or `Self::pop`,
        // which both preserve the validity of it.
        unsafe {
            self.insert_in_dir(self.end, Direction::After, item);
        }
    }

    /// Inserts the given element in the given direction of the anchor element, or as the
    /// sole element of this list, if `anchor` is `None`. Ensures that `self.start` and
    /// `self.end` stay updated accordingly, if there is no node in `direction`.
    ///
    /// # Safety
    ///
    /// If `anchor` is `Some`, it must be a valid, well-aligned pointer to a list element owned by this list, as well as the node in the given direction (if any).
    ///
    /// # Panics
    ///
    /// Panics if `anchor` is the sentinel tail or head element, and `direction` points
    /// away from the rest of the list.
    unsafe fn insert_in_dir(&mut self, anchor: MaybePointer<T>, direction: Direction, item: T) {
        let (before_new, after_new) = match anchor {
            Some(anchor) => retrieve_paired_elements(anchor, Pair::AnchorAnd(direction)),
            None => (None, None),
        };

        let new_node = allocate(Node {
            data: item,
            prev: before_new,
            next: after_new,
        });

        // SAFETY: Delegated to the caller.
        unsafe {
            match before_new {
                Some(before_new) => (*before_new.as_ptr()).next = Some(new_node),
                None => self.start = Some(new_node),
            }
            match after_new {
                Some(after_new) => (*after_new.as_ptr()).prev = Some(new_node),
                None => self.end = Some(new_node),
            }
        }

        self.len += 1;
    }

    /// Removes the element at the beginning of the list, should complete in _O_(1).
    pub fn pop_front(&mut self) -> Option<T> {
        let first = self.start?;
        // SAFETY: Same as `Self::push_front`,
        unsafe { Some(self.remove(first)) }
    }

    /// Removes the element at the end of the list, should complete in _O_(1).
    pub fn pop_back(&mut self) -> Option<T> {
        let last = self.end?;
        // SAFETY: Same as `Self::push_back`.
        unsafe { Some(self.remove(last)) }
    }

    /// Removes the given element by first deallocating the node, then unlinking it.
    ///
    /// # Safety
    ///
    /// `node` must be a valid, well-aligned pointer to a list element owned by this list.
    unsafe fn remove(&mut self, node: Pointer<T>) -> T {
        let (before_ele, after_ele) = retrieve_paired_elements(node, Pair::Surrounding);

        // unlink it from the previous elements
        // there's 3 cases:
        match (before_ele, after_ele) {
            // 1. ele is at _both_ ends of the list (the only element)
            (None, None) => {
                self.start = None;
                self.end = None;
            }
            // 2. ele is at _one_ end of the list
            //    => readjustment of self.start/end necessary
            (Some(before_ele), None) => {
                (*before_ele.as_ptr()).next = None;
                self.end = Some(before_ele);
            }
            (None, Some(after_ele)) => {
                (*after_ele.as_ptr()).prev = None;
                self.start = Some(after_ele);
            }
            // 3. ele is somewhere _inside_ of the list
            (Some(before_ele), Some(after_ele)) => {
                (*before_ele.as_ptr()).next = Some(after_ele);
                (*after_ele.as_ptr()).prev = Some(before_ele);
            }
        }

        self.len -= 1;

        // reboxed will be dropped at the end of the scope -- and deallocate the Node
        let reboxed = Box::from_raw(node.as_ptr());
        reboxed.data
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Before,
    After,
}

#[derive(Clone, Copy)]
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
unsafe fn retrieve_paired_elements<T>(
    anchor: Pointer<T>,
    which: Pair,
) -> (MaybePointer<T>, MaybePointer<T>) {
    match which {
        Pair::AnchorAnd(Direction::Before) => {
            let ele_before_anchor = anchor.as_ref().prev;
            (ele_before_anchor, Some(anchor))
        }
        Pair::AnchorAnd(Direction::After) => {
            let ele_after_anchor = anchor.as_ref().next;
            (Some(anchor), ele_after_anchor)
        }
        Pair::Surrounding => {
            let ele_before_anchor = anchor.as_ref().prev;
            let ele_after_anchor = anchor.as_ref().next;
            (ele_before_anchor, ele_after_anchor)
        }
    }
}

fn allocate<T>(item: T) -> NonNull<T> {
    let ptr = Box::into_raw(Box::new(item));
    // SAFETY: `Box::into_raw` always returns a non-null pointer according to the docs
    unsafe { NonNull::new_unchecked(ptr) }
}

impl<T: Clone> Clone for ReversibleList<T> {
    fn clone(&self) -> Self {
        self.iter().map(Clone::clone).collect()
    }

    // TODO: optimized clone_from, someday...
}

impl<T: fmt::Debug> fmt::Debug for ReversibleList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Default for ReversibleList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for ReversibleList<T> {
    fn drop(&mut self) {
        // just create a cursor and remove elements until it is empty
        // the cursor advances to the next element automatically
        let mut cursor = self.cursor_mut_front();
        while cursor.remove_current().is_some() {}
    }
}

impl<T> Extend<T> for ReversibleList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // distortions caused by Self::reverse are only applicable on a finite range
        // so extending a ReversibleList *always* ends up at the absolute end, either way
        let mut back_cursor = self.cursor_mut_back();

        for item in iter {
            back_cursor.insert_after(item);
            back_cursor.move_next();
        }
    }
}

impl<T> FromIterator<T> for ReversibleList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T> From<Vec<T>> for ReversibleList<T> {
    fn from(value: Vec<T>) -> Self {
        value.into_iter().collect()
    }
}

impl<T, const N: usize> From<[T; N]> for ReversibleList<T> {
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T: Hash> Hash for ReversibleList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        for item in self.iter() {
            item.hash(state);
        }
    }
}

impl<T: PartialEq> PartialEq for ReversibleList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for ReversibleList<T> {}

impl<T: PartialOrd> PartialOrd for ReversibleList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord> Ord for ReversibleList<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}
