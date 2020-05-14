use std::ops::{Deref, DerefMut};

pub struct SendProtectorMut<T: Send> {
    data: imp::SendProtectorMut<T>,
}

pub struct RefMut<'a, T: Send> {
    data: imp::RefMut<'a, T>,
}

impl<T: Send> SendProtectorMut<T> {
    pub fn new(t: T) -> Self {
        SendProtectorMut {
            data: imp::SendProtectorMut::new(t)
        }
    }
    pub unsafe fn get_mut(&self) -> RefMut<'_, T> {
        RefMut {
            data: self.data.get_mut(),
        }
    }
}

impl<T: Send> Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<T: Send> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

pub struct SendProtector<T: Send> {
    data: imp::SendProtector<T>,
}

pub struct Ref<'a, T: Send> {
    data: imp::Ref<'a, T>,
}

impl<T: Send> SendProtector<T> {
    pub fn new(t: T) -> Self {
        SendProtector {
            data: imp::SendProtector::new(t)
        }
    }
    pub unsafe fn get(&self) -> Ref<'_, T> {
        Ref {
            data: self.data.get(),
        }
    }
}

impl<T: Send> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

#[cfg(debug_assertions)]
mod imp {
    use parking_lot::{Mutex, MutexGuard, ReentrantMutex, ReentrantMutexGuard};

    pub struct SendProtectorMut<T: Send> {
        data: Mutex<T>,
    }

    unsafe impl<T: Send> Sync for SendProtectorMut<T> {}

    pub type RefMut<'a, T> = MutexGuard<'a, T>;

    impl<T: Send> SendProtectorMut<T> {
        pub fn new(t: T) -> Self {
            SendProtectorMut {
                data: Mutex::new(t),
            }
        }
        pub unsafe fn get_mut(&self) -> RefMut<T> {
            self.data.try_lock().expect("Tried to either access data from multiple threads or do a re-entrant data access! Bug in cef wrapper - please report to https://github.com/dungeonfog/cef.")
        }
    }

    pub struct SendProtector<T: Send> {
        data: ReentrantMutex<T>,
    }

    unsafe impl<T: Send> Sync for SendProtector<T> {}

    pub type Ref<'a, T> = ReentrantMutexGuard<'a, T>;

    impl<T: Send> SendProtector<T> {
        pub fn new(t: T) -> Self {
            SendProtector {
                data: ReentrantMutex::new(t),
            }
        }
        pub unsafe fn get(&self) -> Ref<T> {
            self.data.try_lock().expect("Tried to access data from multiple threads! Bug in cef wrapper - please report to https://github.com/dungeonfog/cef.")
        }
    }
}

#[cfg(not(debug_assertions))]
mod imp {
    use std::cell::UnsafeCell;

    pub struct SendProtectorMut<T: Send> {
        data: UnsafeCell<T>,
    }

    unsafe impl<T: Send> Sync for SendProtectorMut<T> {}

    pub type RefMut<'a, T> = &'a mut T;

    impl<T: Send> SendProtectorMut<T> {
        pub fn new(t: T) -> Self {
            SendProtectorMut {
                data: UnsafeCell::new(t),
            }
        }
        pub unsafe fn get_mut(&self) -> RefMut<T> {
            &mut *self.data.get()
        }
    }

    pub struct SendProtector<T: Send> {
        data: T,
    }

    unsafe impl<T: Send> Sync for SendProtector<T> {}

    pub type RefMut<'a, T> = &'a T;

    impl<T: Send> SendProtector<T> {
        pub fn new(t: T) -> Self {
            SendProtector {
                data: t,
            }
        }
        pub unsafe fn get(&self) -> RefMut<T> {
            &self.data
        }
    }
}

