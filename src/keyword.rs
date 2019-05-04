use error::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess, IntoDeserializer};
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub struct Keyword {
    pub value: String,
}
pub const TOKEN: &'static str = "$serde_edn::private::KeywordHack";


impl Keyword {
    #[inline]
    pub fn from_str(s: &str) -> Option<Keyword> {
        Some(Keyword { value: String::from(s) })
    }
//    fn visit<'de, V>(self, visitor: V) -> Result<V::Value>
//        where
//            V: de::Visitor<'de>,
//    {
//        match self {
//            Keyword()
//        }
//    }
}

impl Serialize for Keyword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
//        serializer.serialize_str(&self.value)
        let mut kw = String::with_capacity(1 + self.value.len());
        kw.push_str(":");
        kw.push_str(&self.value);
        serializer.serialize_str(&kw)
    }
}

impl<'de> Deserialize<'de> for Keyword {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Keyword, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct KeywordVisitor;

        impl<'de> Visitor<'de> for KeywordVisitor {
            type Value = Keyword;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a keyword")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Keyword, E>
                where
                    E: de::Error,
            {
                println!("KeywordVisitor: visit-str");
                Keyword::from_str(value).ok_or_else(|| de::Error::custom("not a keyword"))
            }

            #[inline]
            #[cfg(any(feature = "std", feature = "alloc"))]
            fn visit_string<E>(self, value: String) -> Result<Keyword, E>
                where
                    E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(KeywordVisitor)
    }
}

impl<'de> Deserializer<'de> for Keyword {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        println!("deser any kw");
        visitor.visit_string(self.value)
    }
    forward_to_deserialize_any! {
        bool char str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
        i8 i16 i32 i64
        u8 u16 u32 u64
        f32 f64
    }
}

//impl<'de, 'a> Deserializer<'de> for &'a Keyword {
//    type Error = Error;
//
//    #[inline]
//    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
//        where
//            V: Visitor<'de>,
//    {
//        visitor.visit_borrowed_str(&self.value.clone())
//    }
//    forward_to_deserialize_any! {
//        bool char str string bytes byte_buf option unit unit_struct
//        newtype_struct seq tuple tuple_struct map struct enum identifier
//        ignored_any
//        i8 i16 i32 i64
//        u8 u16 u32 u64
//        f32 f64
//    }
//}


impl fmt::Display for Keyword {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//        Display::fmt(&self.value, formatter)
        write!(formatter, "{}", &self.value)
    }
}

impl Debug for Keyword {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_tuple("Keyword").field(&self.value).finish()
    }
}

//fn visit()0
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

impl FromStr for Keyword {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Keyword{value:String::from(s)})
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

//impl From<ParserNumber> for Number {
//    fn from(value: ParserNumber) -> Self {
//        let n = match value {
//            ParserNumber::F64(f) => {
//                #[cfg(not(feature = "arbitrary_precision"))]
//                    {
//                        N::Float(f)
//                    }
//                #[cfg(feature = "arbitrary_precision")]
//                    {
//                        f.to_string()
//                    }
//            }
//            ParserNumber::U64(u) => {
//                #[cfg(not(feature = "arbitrary_precision"))]
//                    {
//                        N::PosInt(u)
//                    }
//                #[cfg(feature = "arbitrary_precision")]
//                    {
//                        u.to_string()
//                    }
//            }
//            ParserNumber::I64(i) => {
//                #[cfg(not(feature = "arbitrary_precision"))]
//                    {
//                        N::NegInt(i)
//                    }
//                #[cfg(feature = "arbitrary_precision")]
//                    {
//                        i.to_string()
//                    }
//            }
//            #[cfg(feature = "arbitrary_precision")]
//            ParserNumber::String(s) => s,
//        };
//        Number { n: n }
//    }
//}