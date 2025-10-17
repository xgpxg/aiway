use rbatis::rbdc::DateTime;
use serde::{Deserializer, Serializer, de};

#[allow(unused)]
pub(crate) mod pool;

/// 序列化时间
///
/// 用于将rbatis映射的时间`DateTime`在传给前端时，序列化为字符串
pub fn serialize_datetime<S: Serializer>(
    time: &Option<DateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match time {
        None => serializer.serialize_none(),
        Some(dt) => serializer.serialize_str(&dt.format("YYYY-MM-DD hh:mm:ss")),
    }
}

/// 反序列化varchar或text为Option<String>
/// 支持sqlite和mysql
/// 用于解决rbatis在反序列化时的问题
pub fn deserialize_to_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> de::Visitor<'de> for StringVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter
                .write_str("a string, byte array, or other value that can be converted to string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match str::from_utf8(value) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(de::Error::invalid_value(
                    de::Unexpected::Bytes(value),
                    &self,
                )),
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        // sqlite 需要
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut json_map = serde_json::Map::new();

            while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                json_map.insert(key, value);
            }

            let json_string =
                serde_json::to_string(&json_map).map_err(|e| de::Error::custom(e.to_string()))?;

            Ok(Some(json_string))
        }
    }

    deserializer.deserialize_any(StringVisitor)
}

/// 反序列化varchar或text字段为为Vec<String>
/// 这个可以兼容sqlite和mysql
/// 原生的rbatis在处理mysql时有问题
#[allow(unused)]
pub fn deserialize_to_string_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecStringVisitor;

    impl<'de> de::Visitor<'de> for VecStringVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or byte array representing Vec<String>")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(Some(vec![]))
            } else {
                // 尝试解析为 JSON 数组
                match serde_json::from_str::<Vec<String>>(value) {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => {
                        // 如果不是有效的JSON数组，按逗号分割处理
                        Ok(Some(value.split(',').map(|s| s.to_string()).collect()))
                    }
                }
            }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }

        fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match str::from_utf8(value) {
                Ok(s) => self.visit_str(s),
                Err(_) => Err(de::Error::invalid_value(
                    de::Unexpected::Bytes(value),
                    &self,
                )),
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        // mysql需要
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();

            while let Some(elem) = seq.next_element::<String>()? {
                vec.push(elem);
            }

            Ok(Some(vec))
        }
    }

    deserializer.deserialize_any(VecStringVisitor)
}
