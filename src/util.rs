use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A handle is just an index into a vector of a specific kind.
/// It is useful when we do not want to keep a reference to an element,
/// while taking advantage of strong typing to avoid using integers.
#[derive(Eq, PartialEq, Hash)]
pub struct Handle<T> {
    pub id: usize,
    phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub fn none() -> Self {
        Self {
            id: std::usize::MAX,
            phantom: PhantomData,
        }
    }

    pub fn valid(&self) -> bool {
        self.id != std::usize::MAX
    }
}

impl<'a, T> Handle<T> {
    pub fn get(&self, pack: &'a Pack<T>) -> Option<&'a T> {
        pack.vec.get(self.id)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

/// A pack is just a vector with some more methods to work with handles
pub struct Pack<T> {
    vec: Vec<T>,
}

impl<T> Pack<T> {
    pub fn new() -> Self {
        Self { vec: vec![] }
    }

    pub fn push(&mut self, elem: T) -> Handle<T> {
        let id = self.vec.len();
        self.vec.push(elem);
        Handle::new(id)
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        if !handle.valid() {
            return None;
        }
        self.vec.get(handle.id)
    }

    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        if !handle.valid() {
            return None;
        }
        self.vec.get_mut(handle.id)
    }
}

impl<T> Deref for Pack<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for Pack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Thing {
        val: u32,
    }

    #[test]
    fn simple() {
        let mut pack = Pack::new();
        let thing = pack.push(Thing { val: 2 });
        assert_eq!(thing.get(&pack).unwrap().val, 2);
        assert_eq!(pack.get(&thing).unwrap().val, 2);
    }

    #[test]
    fn multiple() {
        let mut pack = Pack::new();
        let mut handles = vec![];

        for i in 0..4 {
            let handle = pack.push(Thing { val: i });
            handles.push(handle);
        }

        for i in 0..4u32 {
            assert_eq!(handles[i as usize].get(&pack).unwrap().val, i);
            assert_eq!(pack.get(&handles[i as usize]).unwrap().val, i);
        }
    }
}
