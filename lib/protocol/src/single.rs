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

impl Into<SingleValue<String>> for String {
    #[inline]
    fn into(self) -> SingleValue<String> {
        SingleValue::new(self)
    }
}

impl Into<SingleValue<Vec<u8>>> for Vec<u8> {
    #[inline]
    fn into(self) -> SingleValue<Vec<u8>> {
        SingleValue::new(self)
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
