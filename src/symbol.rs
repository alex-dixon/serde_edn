use error::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess, IntoDeserializer};
use std::fmt::{self, Debug};
use std::str::FromStr;


pub const TOKEN: &'static str = "$serde_edn::private::SymbolHack";
pub const FIELD: &'static str = "$__serde_edn_private_symbol";
pub const NAME: &'static str = "$__serde_edn_private_Symbol";


#[derive(Clone, PartialEq,Hash)]
pub struct Symbol {
    pub value: Option<String>,
}

impl Symbol {
    #[inline]
    pub fn from_str(s: &str) -> Result<Symbol, Error> {
        Ok(Symbol { value: Some(String::from(s)) })
    }
}

impl FromStr for Symbol {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Symbol { value: Some(String::from(s)) })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref value)  = self.value {
            write!(formatter, "{}", value)?;
        }
        Ok(())
    }
}

impl Debug for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_tuple("Symbol").field(&self.value).finish()
    }
}

impl Serialize for Symbol {
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

impl<'de> Deserialize<'de> for Symbol {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Symbol, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct SymbolVisitor;

        impl<'de> de::Visitor<'de> for SymbolVisitor {
            type Value = Symbol;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an edn symbol")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Symbol, V::Error>
                where
                    V: de::MapAccess<'de>,
            {
                let value = visitor.next_key::<SymbolKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("symbol key not found"));
                }
                let v: SymbolFromString = visitor.next_value()?;
                Ok(v.value)
            }
        }

        static FIELDS: [&'static str; 1] = [FIELD];
        deserializer.deserialize_struct(NAME, &FIELDS, SymbolVisitor)
    }
}

struct SymbolKey;

impl<'de> de::Deserialize<'de> for SymbolKey {
    fn deserialize<D>(deserializer: D) -> Result<SymbolKey, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid symbol field")
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
        Ok(SymbolKey)
    }
}


// Not public API. Should be pub(crate).
#[doc(hidden)]
pub struct SymbolDeserializer<'de> {
    pub value: &'de str,
//    pub value: Option<String>,
}


impl<'de> MapAccess<'de> for SymbolDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where
            K: de::DeserializeSeed<'de>,
    {
//        if self.value.is_none() {
//            return Ok(None);
//        }
        seed.deserialize(SymbolFieldDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where
            V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.into_deserializer())
    }
}

struct SymbolFieldDeserializer;

impl<'de> Deserializer<'de> for SymbolFieldDeserializer {
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

pub struct SymbolFromString {
    pub value: Symbol,
}

impl<'de> de::Deserialize<'de> for SymbolFromString {
    fn deserialize<D>(deserializer: D) -> Result<SymbolFromString, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SymbolFromString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string representing a symbol")
            }

            fn visit_str<E>(self, s: &str) -> Result<SymbolFromString, E>
                where
                    E: de::Error,
            {
                match s.parse() {
                    Ok(x) => Ok(SymbolFromString { value: x }),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
