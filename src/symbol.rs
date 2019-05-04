use error::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{self, Visitor};
use std::fmt::{self, Debug, Display};

#[derive(Clone, PartialEq)]
pub struct Symbol {
    pub value: String,
}

impl Symbol {
    #[inline]
    pub fn from_str(s: &str) -> Option<Symbol> {
        Some(Symbol { value: String::from(s) })
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for Symbol {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Symbol, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct SymbolVisitor;

        impl<'de> Visitor<'de> for SymbolVisitor {
            type Value = Symbol;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a symbol")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Symbol, E>
                where
                    E: de::Error,
            {
                Symbol::from_str(value).ok_or_else(|| de::Error::custom("not a symbol"))
            }

            #[inline]
            #[cfg(any(feature = "std", feature = "alloc"))]
            fn visit_string<E>(self, value: String) -> Result<Symbol, E>
                where
                    E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(SymbolVisitor)
    }
}

impl<'de> Deserializer<'de> for Symbol {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
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

impl fmt::Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.value, formatter)
    }
}

impl Debug for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_tuple("Symbol").field(&self.value).finish()
    }
}
//
//
//fn parse_keyword<'s, T: ?Sized, F>(
//    &'s mut self,
//    scratch: &'s mut Vec<u8>,
//    result: F,
//) -> Result<Reference<'a, 's, T>>
//    where
//        T: 's,
//        F: for<'f> FnOnce(&'s Self, &'f [u8]) -> Result<&'f T>,
//{
//    match try!(self.next()) {
//        // start sequence :: | :/ | :[0-9] is invalid
//        Some(b':') | Some(b'/') | Some(b'0'...b'9') => {
//            return Err(self.peek_error(ErrorCode::InvalidKeyword))
//        },
//        c1 @ Some(b'-') | c1@ Some(b'+') | c1 @ Some(b'.') => {
//            // second char after - | + | .
//            match try!(self.peek()) {
//                Some(b'0'...b'9') => return Err(self.peek_error(ErrorCode::InvalidKeyword)),
//                // TODO. if whitespace then c1
//                Some(b' ') |  Some(b'\n') |  Some(b'\r') | Some(b'\t') => Ok(()),
//                Some(_)=> {
//                    Ok(self.parse_symbol(&mut self.scratch))
//                }
//
//                None => {}
//            }
//            return result(self,).map(Reference::Borrowed)
//
//        }
//        Some(c) => Ok(())
//    }
//}