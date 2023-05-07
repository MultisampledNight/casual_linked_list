use std::{marker::PhantomData, ptr::NonNull};

type Pointer<T> = Option<NonNull<T>>;

pub struct ReversibleList<T> {
    start: Pointer<Head<T>>,
    end: Pointer<Tail<T>>,
    len: usize,
    _owns_by_value: PhantomData<T>,
}

impl<T> ReversibleList<T> {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
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
}

impl<T> Default for ReversibleList<T> {
    fn default() -> Self {
        Self::new()
    }
}

trait Element<T> {}

struct Head<T> {
    next: Pointer<dyn Element<T>>,
    _owns_by_value: PhantomData<T>,
}

impl<T> Element<T> for Head<T> {}

struct Tail<T> {
    prev: Pointer<dyn Element<T>>,
    _owns_by_value: PhantomData<T>,
}

impl<T> Element<T> for Tail<T> {}

struct Node<T> {
    data: T,
    prev: Pointer<dyn Element<T>>,
    next: Pointer<dyn Element<T>>,
}

impl<T> Element<T> for Node<T> {}
