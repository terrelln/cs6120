use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;

pub struct ContextRef<'a, T> {
    ptr: *const T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> ContextRef<'a, T> {
    fn new(ptr: *const T) -> Self {
        ContextRef {
            ptr: ptr,
            phantom: PhantomData,
        }
    }

    pub fn alias<U>(&self, alias: &'a U) -> ContextRef<'a, U> {
        ContextRef {
            ptr: alias,
            phantom: PhantomData,
        }
    }

    pub fn ptr(&self) -> *const T {
        self.ptr
    }
}

impl<'a, T> Deref for ContextRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a T {
        let ptr = unsafe { &*self.ptr };
        ptr
    }
}

impl<'a, T> Clone for ContextRef<'a, T> {
    fn clone(&self) -> Self {
        ContextRef {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Copy for ContextRef<'a, T> {}

impl<'a, T: PartialEq> PartialEq for ContextRef<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&*self.deref(), &*other.deref())
    }
}

impl<'a, T: Eq> Eq for ContextRef<'a, T> {}

impl<'a, T: Ord> Ord for ContextRef<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&*self.deref(), &*other.deref())
    }
}

impl<'a, T: PartialOrd> PartialOrd for ContextRef<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&*self.deref(), &*other.deref())
    }
}

impl<'a, T: Hash> Hash for ContextRef<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&*self.deref(), state)
    }
}

impl<'a, T: fmt::Display> fmt::Display for ContextRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.deref(), f)
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for ContextRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.deref(), f)
    }
}

impl<'a, T> fmt::Pointer for ContextRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}

pub struct Context {
    boxes: RefCell<Vec<Box<dyn Any>>>,
}

impl Context {
    pub fn create<'a, T: 'static>(&'a self, value: T) -> ContextRef<'a, T> {
        let mut boxed: Box<dyn Any> = Box::new(value);
        let ptr = &*boxed.downcast_mut::<T>().unwrap() as *const T;
        {
            let mut boxes = self.boxes.borrow_mut();
            boxes.push(boxed);
        }
        ContextRef::new(ptr)
    }

    pub fn coerce<'a, T: 'static>(&'a self, value: &'a T) -> ContextRef<'a, T> {
        let ptr = value as *const T;
        ContextRef::new(ptr)
    }

    pub fn new() -> Context {
        Context {
            boxes: RefCell::new(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let ctx = Context::new();
        let num0 = ctx.create(5);
        let num1 = ctx.create(5);
        assert_eq!(*num0.deref(), *num1.deref());
    }

    struct S {
        x: i32,
        y: Vec<i32>,
    }

    #[test]
    fn test_alias() {
        let ctx = Context::new();
        let s = S {
            x: 5,
            y: vec![0, 1, 2],
        };
        let s = ctx.create(s);
        let x = s.alias(&s.x);
        let y = s.alias(&s.y[0]);
        assert_eq!(*x, 5);
        assert_eq!(*y, 0);
    }

    #[test]
    fn test_coerce() {
        let ctx = Context::new();
        let s = S {
            x: 5,
            y: vec![0, 1, 2],
        };
        let s = ctx.create(s);
        let x = ctx.coerce(&s.x);
        let y = ctx.coerce(&s.y[0]);
        assert_eq!(*x, 5);
        assert_eq!(*y, 0);
    }
}
