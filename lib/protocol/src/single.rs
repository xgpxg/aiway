use std::cell::UnsafeCell;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};

/// 一个具有内部可变性的单值容器
#[derive(Debug)]
pub struct SingleValue<T> {
    value: UnsafeCell<T>,
    has_value: AtomicBool,
}

impl<T: Display> Display for SingleValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.get();
        if value.is_none() {
            return write!(f, "None");
        }
        write!(f, "{}", value.unwrap())
    }
}

unsafe impl<T: Send> Send for SingleValue<T> {}
unsafe impl<T: Send> Sync for SingleValue<T> {}

impl<T> SingleValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            has_value: AtomicBool::new(true),
        }
    }

    pub fn set(&self, value: T) {
        unsafe {
            *self.value.get() = value;
        }
        self.has_value.store(true, Ordering::Release);
    }

    // 目前bu需要take
    // pub fn take(&self) -> Option<T> {
    //     if self.has_value.swap(false, Ordering::Acquire) {
    //         unsafe { (*self.value.get()).take() }
    //     } else {
    //         None
    //     }
    // }

    pub fn get(&self) -> Option<&T> {
        if self.has_value.load(Ordering::Acquire) {
            unsafe { Some(&*self.value.get()) }
        } else {
            None
        }
    }

    /// 获取并
    pub fn get_flat<U>(&self) -> Option<&U>
    where
        T: AsRef<Option<U>>
    {
        self.get().and_then(|inner| inner.as_ref().as_ref())
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
