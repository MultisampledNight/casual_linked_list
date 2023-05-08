use std::{marker::PhantomData, ptr::NonNull};

type ElementPointer<'a, T> = NonNull<dyn Element<'a, T> + 'a>;

#[derive(Debug)]
pub struct ReversibleList<'a, T> {
    start: NonNull<Head<'a, T>>,
    end: NonNull<Tail<'a, T>>,
    len: usize,
    _owns_by_value: PhantomData<T>,
}

impl<'a, T: 'a> ReversibleList<'a, T> {
    pub fn new() -> Self {
        // SAFETY: Box::into_raw guarantees non-nullness
        //         0x1 is trivially not NULL, alignment isn't of matter since it's
        //         never read anyway
        let (start, end) = unsafe {
            let start = allocate(Head {
                next: NonNull::<Tail<T>>::dangling(),
                _owns_by_value: PhantomData,
            });

            let end = allocate(Tail {
                prev: start,
                _owns_by_value: PhantomData,
            });

            (*start.as_ptr()).next = end;
            (start, end)
        };

        Self {
            start,
            end,
            len: 0,
            _owns_by_value: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_back(&mut self, item: T) {
        // SAFETY:
        // 1. lifetimes
        //    Tail and Nodes are never leaked to the outside world and only created for the
        //    list -- which lives definitely long enough, else this method would be
        //    incallable
        // 2. pointer validity
        //    `self.end` is never invalidated and initialized only in `Self::new`,
        //    `tail.prev` is only set by `Element::set_prev`, where its unsafe contract
        //    must be upheld
        let last_element = unsafe {
            let tail = self.end.as_mut();
            tail.prev.as_mut()
        };
        last_element.insert_after(item);
    }
}

impl<T> Drop for ReversibleList<'_, T> {
    fn drop(&mut self) {
        // SAFETY: boxes have been previously allocated in `Self::new` and can only be
        //         dropped here -- since this is the Drop impl
        //         additionally, `start` and `end` aren't exposed
        unsafe {
            drop(Box::from_raw(self.start.as_ptr()));
            drop(Box::from_raw(self.end.as_ptr()));
        }
    }
}

impl<'a, T: 'a> Default for ReversibleList<'a, T>
where
    T: 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Rust is not suited at all for the composite pattern in such a low-level collection. But for fun
/// and profit I'll use it here anyway.
trait Element<'a, T: 'a> {
    fn data(&self) -> Option<&T>;
    fn data_mut(&mut self) -> Option<&mut T>;

    fn insert_after(&mut self, item: T);
    /// Sets the pointer to the previous element of this element to the given pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is a pointer to a valid node, and that the
    /// previous node is appropiately dropped.
    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>);
}

struct Head<'a, T> {
    next: ElementPointer<'a, T>,
    _owns_by_value: PhantomData<T>,
}

impl<'a, T: 'a> Element<'a, T> for Head<'a, T> {
    fn data(&self) -> Option<&T> {
        None
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        None
    }

    fn insert_after(&mut self, item: T) {
        let new_next = Node {
            data: item,
            // SAFETY: if &mut self would be null this method call would not work anyway
            prev: unsafe { NonNull::new_unchecked(self) },
            next: self.next,
        };

        let new_next = allocate(new_next);

        // SAFETY: self.next and self.prev are *always* valid, since they're never
        //         invalidated, only swapped with new pointers
        unsafe {
            (*self.next.as_ptr()).set_prev(new_next)
        }
    }

    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>) {
        panic!("head does not have any previous element, but tried to set {ptr:?}");
    }
}

struct Tail<'a, T> {
    prev: ElementPointer<'a, T>,
    _owns_by_value: PhantomData<T>,
}

impl<'a, T: 'a> Element<'a, T> for Tail<'a, T> {
    fn data(&self) -> Option<&T> {
        None
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        None
    }

    fn insert_after(&mut self, _item: T) {
        panic!("no elements can be inserted *after* the tail");
    }

    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>) {
        self.prev = ptr;
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

    fn insert_after(&mut self, item: T) {
        let new_next = Node {
            data: item,
            // SAFETY: if &mut self would be null this method call would not work anyway
            prev: unsafe { NonNull::new_unchecked(self) },
            next: self.next,
        };

        let new_next = allocate(new_next);

        // SAFETY: self.next and self.prev are *always* valid, since they're never
        //         invalidated, only swapped with new pointers
        unsafe {
            (*self.next.as_ptr()).set_prev(new_next)
        }
    }

    unsafe fn set_prev(&mut self, ptr: ElementPointer<'a, T>) {
        self.prev = ptr;
    }
}

fn allocate<T>(item: T) -> NonNull<T> {
    let ptr = Box::into_raw(Box::new(item));
    // SAFETY: `Box::into_raw` always returns a non-null pointer according to the docs
    unsafe { NonNull::new_unchecked(ptr) }
}
