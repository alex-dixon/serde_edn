use error::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess, IntoDeserializer};
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub struct Keyword {
    pub value: Option<String>,
}

pub const TOKEN: &'static str = "$serde_edn::private::KeywordHack";
pub const FIELD: &'static str = "$__serde_edn_private_keyword";
pub const NAME: &'static str = "$__serde_edn_private_Keyword";


impl Keyword {
    #[inline]
    pub fn from_str(s: &str) -> Result<Keyword, Error> {
        Ok(Keyword { value: Some(String::from(s)) })
    }
}

impl FromStr for Keyword {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Keyword { value: Some(String::from(s)) })
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref value)  = self.value {
            write!(formatter, ":{}", value)?;
        }
        Ok(())
    }
}

impl Debug for Keyword {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_tuple("Keyword").field(&self.value).finish()
    }
}

impl Serialize for Keyword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct(TOKEN, 1)?;
        s.serialize_field(TOKEN, &self.to_string())?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Keyword {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Keyword, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct KeywordVisitor;

        impl<'de> de::Visitor<'de> for KeywordVisitor {
            type Value = Keyword;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an edn keyword")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Keyword, V::Error>
                where
                    V: de::MapAccess<'de>,
            {
                let value = visitor.next_key::<KeywordKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("keyword key not found"));
                }
                let v: KeywordFromString = visitor.next_value()?;
                Ok(v.value)
            }
        }

        static FIELDS: [&'static str; 1] = [FIELD];
        deserializer.deserialize_struct(NAME, &FIELDS, KeywordVisitor)
    }
}

struct KeywordKey;

impl<'de> de::Deserialize<'de> for KeywordKey {
    fn deserialize<D>(deserializer: D) -> Result<KeywordKey, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid keyword field")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
                where
                    E: de::Error,
            {
                if s == FIELD {
                    Ok(())
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(KeywordKey)
    }
}


// Not public API. Should be pub(crate).
#[doc(hidden)]
pub struct KeywordDeserializer<'de> {
    pub value: &'de str,
//    pub value: Option<String>,
}


impl<'de> MapAccess<'de> for KeywordDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where
            K: de::DeserializeSeed<'de>,
    {
//        if self.value.is_none() {
//            return Ok(None);
//        }
        seed.deserialize(KeywordFieldDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where
            V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.into_deserializer())
    }
}

struct KeywordFieldDeserializer;

impl<'de> Deserializer<'de> for KeywordFieldDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(TOKEN)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct ignored_any
        unit_struct tuple_struct tuple enum identifier
    }
}



// does it ever end?
pub struct KeywordFromString {
    pub value: Keyword,
}

impl<'de> de::Deserialize<'de> for KeywordFromString {
    fn deserialize<D>(deserializer: D) -> Result<KeywordFromString, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = KeywordFromString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string representing a keyword")
            }

            fn visit_str<E>(self, s: &str) -> Result<KeywordFromString, E>
                where
                    E: de::Error,
            {
                match s.parse() {
                    Ok(x) => Ok(KeywordFromString { value: x }),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
