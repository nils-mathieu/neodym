use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// A mutually exclusive lock protecting a value of type `T`.'
///
/// # Fairness
///
/// This mutex is *not* fair. This means that there is no guarantee that threads will acquire the
/// lock in the order they requested it.
pub struct Mutex<T> {
    /// The protected value.
    value: UnsafeCell<T>,
    /// the current state of the mutex.
    lock: AtomicBool,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new [`Mutex<T>`] with the given value.
    ///
    /// The mutex is initially unlocked.
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            lock: AtomicBool::new(false),
        }
    }

    /// Returns whether the mutex is currently locked.
    ///
    /// Note that this function can only be used as a hint, as the mutex may change state by the
    /// time this function returns.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Relaxed)
    }

    /// Locks the mutex and returns a guard that releases the lock when dropped.
    #[inline]
    pub fn lock(&self) -> MutexLock<T> {
        while self
            .lock
            .compare_exchange_weak(false, true, Acquire, Relaxed)
            .is_err()
        {
            // Wait until the lock seems released.
            while self.is_locked() {
                core::hint::spin_loop();
            }
        }

        MutexLock {
            value: unsafe { &mut *self.value.get() },
            lock: &self.lock,
        }
    }

    /// Returns the inner value without locking the mutex.
    ///
    /// This is safe because the mutex must be exclusively borrowed to call this function, which
    /// ensures that no lock exists for it.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

/// Holds a lock on a [`Mutex<T>`], ensuring exclusive access to the protected value.
pub struct MutexLock<'a, T> {
    value: &'a mut T,
    lock: &'a AtomicBool,
}

impl<'a, T> MutexLock<'a, T> {
    /// Maps the protected value to a new value.
    pub fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> MutexLock<'a, U> {
        let this = ManuallyDrop::new(this);

        unsafe {
            let value = core::ptr::read(&this.value);
            let lock = core::ptr::read(&this.lock);

            MutexLock {
                value: f(value),
                lock,
            }
        }
    }

    /// Leaks the mutex lock, returning a mutable reference to the protected value without
    /// releasing the lock.
    ///
    /// This effectively circumvents the mutex lock, locking the mutex forever.
    pub fn leak(this: Self) -> &'a mut T {
        let this = ManuallyDrop::new(this);
        unsafe { core::ptr::read(&this.value) }
    }
}

impl<'a, T> Deref for MutexLock<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> Drop for MutexLock<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        self.lock.store(false, Release);
    }
}
