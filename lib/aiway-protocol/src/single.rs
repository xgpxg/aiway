use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::UnsafeCell;
use std::fmt::Display;

/// 一个具有内部可变性的单值容器
#[derive(Debug)]
pub struct SingleValue<T> {
    value: UnsafeCell<Option<T>>,
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
            value: UnsafeCell::new(Some(value)),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: UnsafeCell::new(None),
        }
    }

    pub fn set(&self, value: T) {
        unsafe {
            *self.value.get() = Some(value);
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        unsafe { (*self.value.get()).as_ref() }
    }

    #[inline]
    #[allow(unused)]
    pub fn take(&self) -> Option<T> {
        unsafe {
            let option_ref = &mut *self.value.get();
            option_ref.take()
        }
    }

    // #[inline]
    // pub fn get_flat<U>(&self) -> Option<&U>
    // where
    //     T: AsRef<Option<U>>,
    // {
    //     self.get().and_then(|inner| inner.as_ref().as_ref())
    // }
}

impl<T: Default> Default for SingleValue<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl From<String> for SingleValue<String> {
    #[inline]
    fn from(value: String) -> Self {
        SingleValue::new(value)
    }
}

impl From<&str> for SingleValue<String> {
    #[inline]
    fn from(value: &str) -> Self {
        SingleValue::new(value.to_string())
    }
}

impl<T: Serialize> Serialize for SingleValue<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.get() {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}
impl<'de, T> Deserialize<'de> for SingleValue<T>
where
    T: Deserialize<'de> + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(SingleValue::new(value))
    }
}
