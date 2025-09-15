use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// 一个具有内部可变性的单值容器
#[derive(Debug)]
pub struct SingleValue<T> {
    value: UnsafeCell<Option<T>>,
    has_value: AtomicBool,
}

unsafe impl<T: Send> Send for SingleValue<T> {}
unsafe impl<T: Send> Sync for SingleValue<T> {}

impl<T> SingleValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(Some(value)),
            has_value: AtomicBool::new(true),
        }
    }

    pub fn set(&self, value: T) {
        unsafe {
            *self.value.get() = Some(value);
        }
        self.has_value.store(true, Ordering::Release);
    }

    pub fn take(&self) -> Option<T> {
        if self.has_value.swap(false, Ordering::Acquire) {
            unsafe { (*self.value.get()).take() }
        } else {
            None
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.has_value.load(Ordering::Acquire) {
            unsafe { (*self.value.get()).as_ref() }
        } else {
            None
        }
    }
}

impl<T: Default> Default for SingleValue<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl Into<SingleValue<String>> for String {
    fn into(self) -> SingleValue<String> {
        SingleValue::new(self)
    }
}

impl Into<SingleValue<Vec<u8>>> for Vec<u8> {
    fn into(self) -> SingleValue<Vec<u8>> {
        SingleValue::new(self)
    }
}
