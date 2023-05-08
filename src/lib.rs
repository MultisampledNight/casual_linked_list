use std::{marker::PhantomData, ptr::NonNull};

type ElementPointer<'a, T> = NonNull<dyn Element<T> + 'a>;

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
            let start = Box::into_raw(Box::new(Head {
                next: NonNull::<Tail<T>>::dangling(),
                _owns_by_value: PhantomData,
            }));
            let start = NonNull::new_unchecked(start);

            let end = Box::into_raw(Box::new(Tail {
                prev: start,
                _owns_by_value: PhantomData,
            }));
            let end = NonNull::new_unchecked(end);

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
        let last_element = unsafe { (*self.end.as_ptr()).prev };

        todo!()
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
    T: 'a
{
    fn default() -> Self {
        Self::new()
    }
}

/// Rust is not suited at all for the composite pattern in such a low-level collection. But for fun
/// and profit I'll use it here anyway.
trait Element<T> {
    fn data(&self) -> Option<&T>;
    fn data_mut(&mut self) -> Option<&mut T>;
}

struct Head<'a, T> {
    next: ElementPointer<'a, T>,
    _owns_by_value: PhantomData<T>,
}

impl<'a, T: 'a> Element<T> for Head<'a, T> {
    fn data(&self) -> Option<&T> {
        None
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        None
    }
}

struct Tail<'a, T> {
    prev: ElementPointer<'a, T>,
    _owns_by_value: PhantomData<T>,
}

impl<'a, T: 'a> Element<T> for Tail<'a, T> {
    fn data(&self) -> Option<&T> {
        None
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        None
    }
}

struct Node<'a, T> {
    data: T,
    prev: ElementPointer<'a, T>,
    next: ElementPointer<'a, T>,
}

impl<T> Element<T> for Node<'_, T> {
    fn data(&self) -> Option<&T> {
        Some(&self.data)
    }

    fn data_mut(&mut self) -> Option<&mut T> {
        Some(&mut self.data)
    }
}
