use std::{
    sync::Mutex,
    collections::HashMap,
    sync::{Arc, Weak},
};
use lazy_static::lazy_static;
use typemap::{Key, TypeMap, SendMap};

use crate::ptr_hash::Hashed;

lazy_static! {
    static ref LINKS: Mutex<SendMap> = Mutex::new(TypeMap::custom());
}

struct ReferenceWrapper<W>(Weak<W>) where W: 'static + Send + Sync;

impl<W: 'static + Send + Sync> Key for ReferenceWrapper<W> {
    type Value = HashMap<u64, Self>;
}

pub(crate) fn register<R>(object: Hashed, wrapper: &Arc<R>) -> bool where R: 'static + Send + Sync {
    if let Ok(ref mut links) = LINKS.lock() {
        let map: &mut HashMap<u64, ReferenceWrapper<R>> = links.entry::<ReferenceWrapper<R>>().or_insert_with(HashMap::new);
        map.insert(object.into(), ReferenceWrapper(Arc::downgrade(wrapper)));
        true
    } else {
        false
    }
}

pub(crate) fn unregister<R>(object: Hashed) where R: 'static + Send + Sync {
    if let Ok(ref mut links) = LINKS.lock() {
        if let Some(map) = links.get_mut::<ReferenceWrapper<R>>() {
            map.remove(&object.into());
        }
    }
}

pub(crate) fn get<R>(object: Hashed) -> Option<Arc<R>> where R: 'static + Send + Sync {
    if let Ok(ref mut links) = LINKS.lock() {
        if let Some(map) = links.get::<ReferenceWrapper<R>>() {
            return map.get(&object.into()).and_then(|reference| Weak::upgrade(&reference.0));
        }
    }
    None
}
