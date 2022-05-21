// Includes

use mlsys::*;

use std::ops::{Deref, DerefMut};

#[cfg(not(feature = "thread-safe"))]
use std::cell::UnsafeCell;

// Types

#[repr(C)]
#[derive(Debug)]
pub enum Bool {
    True,
    False,
}

pub(crate) type FNV = u64;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SyncAddress(Address);

#[repr(transparent)]
#[cfg(not(feature = "thread-safe"))]
pub(crate) struct Mutex<T>(UnsafeCell<T>);

#[repr(transparent)]
#[cfg(all(feature = "thread-safe", not(feature = "spinlock")))]
pub(crate) struct Mutex<T>(std::sync::Mutex<T>);

#[repr(transparent)]
#[cfg(feature = "spinlock")]
pub(crate) struct Mutex<T>(spin::Mutex<T>);

pub(crate) type NoHashMap<K, V> =
    std::collections::HashMap<K, V, nohash_hasher::BuildNoHashHasher<K>>;

// SyncAddress

impl SyncAddress {
    pub(crate) fn from(p: Address) -> Self {
        SyncAddress(p)
    }

    pub(crate) fn extract(&self) -> Address {
        self.0
    }
}

unsafe impl Send for SyncAddress {}
unsafe impl Sync for SyncAddress {}

impl nohash_hasher::IsEnabled for SyncAddress {}

// Mutex

#[cfg(not(feature = "thread-safe"))]
impl<T> Mutex<T> {
    pub(crate) fn new(value: T) -> Self {
        Mutex(UnsafeCell::from(value))
    }

    pub(crate) fn lock(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

#[cfg(feature = "thread-safe")]
impl<T> Mutex<T> {
    #[cfg(not(feature = "spinlock"))]
    pub(crate) fn new(value: T) -> Self {
        Mutex(std::sync::Mutex::new(value))
    }

    #[cfg(feature = "spinlock")]
    pub(crate) fn new(value: T) -> Self {
        Mutex(spin::Mutex::new(value))
    }

    #[cfg(not(feature = "spinlock"))]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<T> {
        self.0.lock().unwrap()
    }

    #[cfg(feature = "spinlock")]
    pub(crate) fn lock(&self) -> spin::MutexGuard<T> {
        self.0.lock()
    }
}

impl<T> Deref for Mutex<T> {
    #[cfg(not(feature = "thread-safe"))]
    type Target = UnsafeCell<T>;

    #[cfg(all(feature = "thread-safe", not(feature = "spinlock")))]
    type Target = std::sync::Mutex<T>;

    #[cfg(feature = "spinlock")]
    type Target = spin::Mutex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Mutex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(not(feature = "thread-safe"))]
unsafe impl<T> Send for Mutex<T> {}

#[cfg(not(feature = "thread-safe"))]
unsafe impl<T> Sync for Mutex<T> {}
