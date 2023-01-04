//! AOSA represents Append-Only String Arena, it's convenient to hold plenty of temporary
//! strings inside the continuous memory and free them all at once.
use std::alloc::{alloc, Layout};
use std::mem::align_of;
use std::cell::{Cell, UnsafeCell};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("exceeds capacity with additional {0} bytes")]
    ExceedsCapacity(usize),
}

/// StringArena is a single-thread append-only string arena.
pub struct StringArena {
    arena: UnsafeCell<Box<[u8]>>,
    idx: Cell<usize>,
}

impl StringArena {
    /// Create a new string arena with given capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::from_size_align(cap, align_of::<u8>()).unwrap();
        let arena = unsafe {
            let ptr = alloc(layout);
            let vec = Vec::from_raw_parts(ptr, cap, cap);
            UnsafeCell::new(vec.into_boxed_slice())
        };
        StringArena{arena, idx: Cell::new(0)}
    }

    /// Returns bytes written of current arena.
    #[inline]
    pub fn len(&self) -> usize {
        self.idx.get()
    }

    /// Returns whether the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns capacity of current arena.
    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { (*self.arena.get()).len() }
    }

    /// Add a string into current arena.
    /// Returns the string ref if succeeds.
    /// The only reason of failure is that input string exceeds remained capacity.
    /// The additional bytes required to store it is returned if fails. 
    #[inline]
    pub fn add<T: AsRef<str>>(&self, s: T) -> Result<&str> {
        let s = s.as_ref();
        let len = s.len();
        let idx = self.len();
        let new_len = len + idx;
        if self.capacity() < new_len {
            return Err(Error::ExceedsCapacity(new_len - self.capacity()))
        }
        // SAFETY:
        // 
        // The mutable byte slice is guaranteed not to be modified concurrently.
        unsafe {
            let arena = &mut *self.arena.get();
            let bs = &mut arena[idx..new_len];
            bs.copy_from_slice(s.as_bytes());
            self.idx.set(new_len);
            Ok(std::str::from_utf8_unchecked(bs))
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_arena() {
        let sa = StringArena::with_capacity(12);
        assert_eq!(sa.len(), 0);
        assert!(sa.is_empty());
        assert_eq!(sa.capacity(), 12);
        let s1 = sa.add("hello").unwrap();
        assert_eq!(s1, "hello");
        let s2 = sa.add("world").unwrap();
        assert_eq!(s2, "world");
        assert!(sa.add("rust").is_err());
    }
}
