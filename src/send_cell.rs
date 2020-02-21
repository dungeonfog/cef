use std::ops::{Deref, DerefMut};

pub struct SendCell<T: Send> {
    data: imp::SendCell<T>,
}

pub struct RefMut<'a, T: Send> {
    data: imp::RefMut<'a, T>,
}

impl<T: Send> SendCell<T> {
    pub fn new(t: T) -> Self {
        SendCell {
            data: imp::SendCell::new(t)
        }
    }
    pub unsafe fn get(&self) -> RefMut<'_, T> {
        RefMut {
            data: self.data.get(),
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


#[cfg(debug_assertions)]
mod imp {
    use parking_lot::{Mutex, MutexGuard};

    pub struct SendCell<T: Send> {
        data: Mutex<T>,
    }

    unsafe impl<T: Send> Sync for SendCell<T> {}

    pub type RefMut<'a, T> = MutexGuard<'a, T>;

    impl<T: Send> SendCell<T> {
        pub fn new(t: T) -> Self {
            SendCell {
                data: Mutex::new(t),
            }
        }
        pub unsafe fn get(&self) -> RefMut<T> {
            self.data.try_lock().expect("Tried to access data from multiple threads! Bug in cef wrapper - please report to https://github.com/anlumo/cef.")
        }
    }
}

#[cfg(not(debug_assertions))]
mod imp {
    use std::cell::UnsafeCell;

    pub struct SendCell<T: Send> {
        data: UnsafeCell<T>,
    }

    unsafe impl<T: Send> Sync for SendCell<T> {}

    pub type RefMut<'a, T> = &'a mut T;

    impl<T: Send> SendCell<T> {
        pub fn new(t: T) -> Self {
            SendCell {
                data: UnsafeCell::new(t),
            }
        }
        pub unsafe fn get(&self) -> RefMut<T> {
            &mut *self.data.get()
        }
    }
}

