use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

/// A handle is a sort of index into a vector of elements of a specific kind.
/// It is useful when we do not want to keep a reference to an element,
/// while taking advantage of strong typing to avoid using integers.
#[derive(Eq, PartialEq, Debug)]
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

/// A `Pack` is a powerful structure which contains a vector of contiguous elements
/// and a list of indices to those elements. `Handle`s are used to work with `Pack`s.
pub struct Pack<T> {
    /// List of contiguous elements
    vec: Vec<T>,
    /// List of indices to elements
    indices: Vec<usize>,
    /// List of positions to free indices
    free: Vec<usize>,
}

impl<T> Pack<T> {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            indices: vec![],
            free: vec![],
        }
    }

    pub fn push(&mut self, elem: T) -> Handle<T> {
        let index = self.vec.len();
        self.vec.push(elem);

        if !self.free.is_empty() {
            let id = self.free.pop().unwrap();
            self.indices[id] = index;
            Handle::new(id)
        } else {
            let id = self.indices.len();
            self.indices.push(index);
            Handle::new(id)
        }
    }

    fn get_vec_index(&self, handle: Handle<T>) -> usize {
        assert!(handle.id < self.indices.len());
        let vec_index = self.indices[handle.id];
        assert!(vec_index < self.vec.len());
        vec_index
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if !handle.valid() {
            return None;
        }
        self.vec.get(self.get_vec_index(handle))
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if !handle.valid() {
            return None;
        }
        let vec_index = self.get_vec_index(handle);
        self.vec.get_mut(vec_index)
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        let vec_index = self.get_vec_index(handle);
        let last_vec_index = self.vec.len() - 1;
        self.vec.swap(vec_index, last_vec_index);
        self.vec.pop();

        // Update index that was pointing to last element
        // We do not know where it is, therefore let us find it
        for index in &mut self.indices {
            if *index == last_vec_index {
                *index = vec_index;
            }
        }

        // Index of the removed element can be added to free list
        self.free.push(handle.id);
    }
}

impl<T> From<Vec<T>> for Pack<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut ret = Self::new();

        for elem in vec {
            ret.push(elem);
        }

        ret
    }
}

impl<T> FromIterator<T> for Pack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut ret = Self::new();

        for elem in iter {
            ret.push(elem);
        }

        ret
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
        assert_eq!(pack.get(thing).unwrap().val, 2);
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
            assert_eq!(pack.get(handles[i as usize]).unwrap().val, i);
        }
    }

    #[test]
    fn add_remove_add() {
        let mut pack = Pack::new();
        let handle = pack.push(Thing { val: 0 });
        assert_eq!(handle.id, 0);

        pack.remove(handle);
        assert_eq!(pack.len(), 0);

        let handle = pack.push(Thing { val: 1 });
        assert_eq!(handle.id, 0);
        assert_eq!(pack.get(handle).unwrap().val, 1);
    }
}

/// Useful timer to get delta time, and previous time for ImGui
pub struct Timer {
    prev: Instant,
    curr: Instant,
}

impl Timer {
    pub fn new() -> Self {
        let prev = Instant::now();
        let curr = Instant::now();
        Self { prev, curr }
    }

    /// Returns delta time in seconds
    pub fn get_delta(&mut self) -> Duration {
        self.curr = Instant::now();
        let delta = self.curr - self.prev;
        self.prev = self.curr;
        delta
    }

    /// Returns the time of last `get_delta()`
    pub fn get_prev(&self) -> Instant {
        self.prev
    }
}

pub struct ScopedTimer<'a> {
    timer: Timer,
    message: &'a str,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(msg: &'a str) -> Self {
        Self {
            timer: Timer::new(),
            message: msg,
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        println!(
            "{} ({}s)",
            self.message,
            self.timer.get_delta().as_secs_f32()
        );
    }
}
