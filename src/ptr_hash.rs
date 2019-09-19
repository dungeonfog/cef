use std::{
    ptr::hash,
    hash::Hasher,
    collections::hash_map::DefaultHasher,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Hashed(u64);

impl Hashed {
    pub fn from<C>(object: *const C) -> Self {
        let mut hasher = DefaultHasher::new();
        hash(object, &mut hasher);
        Self(hasher.finish())
    }
}

impl Into<u64> for Hashed {
    fn into(self) -> u64 {
        self.0
    }
}

unsafe impl Send for Hashed {}
unsafe impl Sync for Hashed {}
