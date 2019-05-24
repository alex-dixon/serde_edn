// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::fmt;
use std::slice;
use std::str;
use std::vec;

use ::{serde, Keyword};
use serde::de::{
    Deserialize, DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess, SeqAccess,
    Unexpected, VariantAccess, Visitor,
};

use error::Error;
use map::Map;
use number::Number;
use value::Value;

use serde::de;

#[cfg(feature = "arbitrary_precision")]
use number::NumberFromString;
use keyword::KeywordFromString;
use symbol::SymbolFromString;
use edn_de::{EDNDeserialize, EDNDeserializer, EDNVisitor, EDNSeqAccess};


// note: This is serde's Deserialize!
impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid edn value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Number::from_f64(value).map_or(Value::Nil, Value::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
                where
                    E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where
                    V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = try!(visitor.next_element()) {
                    vec.push(elem);
                }
//                println!("visit seq for maybe a list {:?}",visitor);
//                match visitor {
//                    ListAccess => println!("OK"),
//                    _ => println!("Not ok")
//                }

                Ok(Value::Vector(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where
                    V: MapAccess<'de>,
            {
                match visitor.next_key_seed(KeyClassifier)? {
                    Some(KeyClass::KeywordHack) => {
                        let kw: KeywordFromString = visitor.next_value()?;
                        Ok(Value::Keyword(kw.value))
                    }
                    Some(KeyClass::SymbolHack) => {
                        let kw: SymbolFromString = visitor.next_value()?;
                        Ok(Value::Symbol(kw.value))
                    }
                    #[cfg(feature = "arbitrary_precision")]
                    Some(KeyClass::Number) => {
                        let number: NumberFromString = visitor.next_value()?;
                        Ok(Value::Number(number.value))
                    }
                    #[cfg(feature = "raw_value")]
                    Some(KeyClass::RawValue) => {
                        let value = visitor.next_value_seed(::raw::BoxedFromString)?;
                        ::from_str(value.get()).map_err(de::Error::custom)
                    }
                    Some(KeyClass::Map(first_key)) => {
                        let mut values = Map::new();

                        values.insert(first_key, try!(visitor.next_value()));
                        while let Some((key, value)) = try!(visitor.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(Value::Object(values))
                    }
                    None => Ok(Value::Object(Map::new())),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
//        EDNDeserializer::deserialize_any(deserializer,ValueVisitor)
    }
}

impl<'de> EDNDeserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as EDNDeserializer<'de>>::Error>
        where
            D: EDNDeserializer<'de> //+ serde::Deserializer<'de>
    {
        struct ValueVisitor;

        impl<'de> EDNVisitor<'de> for ValueVisitor {
            type EDNValue = Value;
//            type Value = Value;

            #[inline]
            fn visit_list<V>(self, mut visitor: V) -> Result<<Self as Visitor<'de>>::Value, V::Error>
                where
                    V: EDNSeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = try!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Value::List(vec))
            }
        }

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid edn value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Number::from_f64(value).map_or(Value::Nil, Value::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
                where
                    E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where
                    V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = try!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Value::Vector(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where
                    V: MapAccess<'de>,
            {
                match visitor.next_key_seed(KeyClassifier)? {
                    Some(KeyClass::KeywordHack) => {
                        let kw: KeywordFromString = visitor.next_value()?;
                        Ok(Value::Keyword(kw.value))
                    }
                    Some(KeyClass::SymbolHack) => {
                        let kw: SymbolFromString = visitor.next_value()?;
                        Ok(Value::Symbol(kw.value))
                    }
                    #[cfg(feature = "arbitrary_precision")]
                    Some(KeyClass::Number) => {
                        let number: NumberFromString = visitor.next_value()?;
                        Ok(Value::Number(number.value))
                    }
                    #[cfg(feature = "raw_value")]
                    Some(KeyClass::RawValue) => {
                        let value = visitor.next_value_seed(::raw::BoxedFromString)?;
                        ::from_str(value.get()).map_err(de::Error::custom)
                    }
                    Some(KeyClass::Map(first_key)) => {
                        let mut values = Map::new();

                        values.insert(first_key, try!(visitor.next_value()));
                        while let Some((key, value)) = try!(visitor.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(Value::Object(values))
                    }
                    None => Ok(Value::Object(Map::new())),
                }
            }
        }

        EDNDeserializer::deserialize_any(deserializer,ValueVisitor)
    }
}

impl str::FromStr for Value {
    type Err = Error;
    fn from_str(s: &str) -> Result<Value, Error> {
        super::super::de::from_str(s)
    }
}

macro_rules! deserialize_prim_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Value::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Value::Number(n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    }
}

fn visit_vector<'de, V>(vector: Vec<Value>, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
{
    let len = vector.len();
    let mut deserializer = SeqDeserializer::new(vector);
    let seq = try!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in vector",
        ))
    }
}

fn visit_list<'de, V>(vector: Vec<Value>, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
{
    let len = vector.len();
    let mut deserializer = SeqDeserializer::new(vector);
//    let mut deserializer = ListDeserializer::new(vector);
    let seq = try!(visitor.visit_seq(&mut deserializer));
    println!("visit_list");
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in list",
        ))
    }
}

fn visit_object<'de, V>(object: Map<String, Value>, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDeserializer::new(object);
    let map = try!(visitor.visit_map(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}



impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(n) => n.deserialize_any(visitor),
            Value::String(v) => visitor.visit_string(v),
            Value::Vector(v) => visit_vector(v, visitor),
            Value::List(v) => visit_list(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
            Value::Keyword(kw) => {
                println!("visit keyword to str...{:?}", kw.to_string());
                visitor.visit_string(kw.to_string())
            }
            Value::Symbol(v) => {
//                v.deserialize_any(visitor)
                visitor.visit_string(v.to_string())

            },
        }
    }

    deserialize_prim_number!(deserialize_i8);
    deserialize_prim_number!(deserialize_i16);
    deserialize_prim_number!(deserialize_i32);
    deserialize_prim_number!(deserialize_i64);
    deserialize_prim_number!(deserialize_u8);
    deserialize_prim_number!(deserialize_u16);
    deserialize_prim_number!(deserialize_u32);
    deserialize_prim_number!(deserialize_u64);
    deserialize_prim_number!(deserialize_f32);
    deserialize_prim_number!(deserialize_f64);

    serde_if_integer128! {
        deserialize_prim_number!(deserialize_i128);
        deserialize_prim_number!(deserialize_u128);
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in edn as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer {
            variant: variant,
            value: value,
        })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
            {
                if name == ::raw::TOKEN {
                    return visitor.visit_map(::raw::OwnedRawDeserializer {
                        raw_value: Some(self.to_string()),
                    });
                }
            }

        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::String(v) => visitor.visit_string(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::String(v) => visitor.visit_string(v),
            Value::Vector(v) => visit_vector(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Vector(v) => visit_vector(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Vector(v) => visit_vector(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
        where
            V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Vector(v)) => {
                serde::Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Object(v)) => {
                serde::Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = try!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(
                    len,
                    &"fewer elements in vector",
                ))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <Map<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: Map<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Owned(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

macro_rules! deserialize_value_ref_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match *self {
                Value::Number(ref n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match *self {
                Value::Number(ref n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    }
}

fn visit_vector_ref<'de, V>(vector: &'de [Value], visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
{
    let len = vector.len();
    let mut deserializer = SeqRefDeserializer::new(vector);
    let seq = try!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in vector",
        ))
    }
}

fn visit_list_ref<'de, V>(vector: &'de [Value], visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
{
    let len = vector.len();
    let mut deserializer = SeqRefDeserializer::new(vector);
    let seq = try!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in vector",
        ))
    }
}

fn visit_object_ref<'de, V>(object: &'de Map<String, Value>, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapRefDeserializer::new(object);
    let map = try!(visitor.visit_map(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de> serde::Deserializer<'de> for &'de Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Nil => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(ref n) => n.deserialize_any(visitor),
            Value::String(ref v) => visitor.visit_borrowed_str(v),
            Value::Vector(ref v) => visit_vector_ref(v, visitor),
            Value::List(ref v) => visit_list_ref(v, visitor),
            Value::Object(ref v) => visit_object_ref(v, visitor),
            Value::Keyword(ref kw) => {
                //todo. keyword ref  deserializer?
                match kw.clone().value {
                    None => Err(serde::de::Error::custom("keyword error")),
                    Some(v) => {
                        println!("keyword ref visit str");
                        visitor.visit_str(&v)
                    }
                }

//                kw.deserialize_any(visitor)
//                visitor.visit_borrowed_str(&kw.value)
            }
            Value::Symbol(ref sym) => {
                match sym.clone().value {
                    None => Err(serde::de::Error::custom("symbol error")),
                    Some(v) => {
                        println!("symbol ref visit str");
                        visitor.visit_str(&v)
                    }
                }
//                visitor.visit_borrowed_str(&sym.value)
            }
        }
    }

    deserialize_value_ref_number!(deserialize_i8);
    deserialize_value_ref_number!(deserialize_i16);
    deserialize_value_ref_number!(deserialize_i32);
    deserialize_value_ref_number!(deserialize_i64);
    deserialize_value_ref_number!(deserialize_u8);
    deserialize_value_ref_number!(deserialize_u16);
    deserialize_value_ref_number!(deserialize_u32);
    deserialize_value_ref_number!(deserialize_u64);
    deserialize_value_ref_number!(deserialize_f32);
    deserialize_value_ref_number!(deserialize_f64);

    serde_if_integer128! {
        deserialize_prim_number!(deserialize_i128);
        deserialize_prim_number!(deserialize_u128);
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        let (variant, value) = match *self {
            Value::Object(ref value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in edn as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            Value::String(ref variant) => (variant, None),
            ref other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumRefDeserializer {
            variant: variant,
            value: value,
        })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
            {
                if name == ::raw::TOKEN {
                    return visitor.visit_map(::raw::OwnedRawDeserializer {
                        raw_value: Some(self.to_string()),
                    });
                }
            }

        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::String(ref v) => visitor.visit_borrowed_str(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::String(ref v) => visitor.visit_borrowed_str(v),
            Value::Vector(ref v) => visit_vector_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Nil => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Vector(ref v) => visit_vector_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Object(ref v) => visit_object_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match *self {
            Value::Vector(ref v) => visit_vector_ref(v, visitor),
            Value::Object(ref v) => visit_object_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct EnumRefDeserializer<'de> {
    variant: &'de str,
    value: Option<&'de Value>,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de> {
    type Error = Error;
    type Variant = VariantRefDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
        where
            V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantRefDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantRefDeserializer<'de> {
    value: Option<&'de Value>,
}

impl<'de> VariantAccess<'de> for VariantRefDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self.value {
            Some(&Value::Vector(ref v)) => {
                serde::Deserializer::deserialize_any(SeqRefDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self.value {
            Some(&Value::Object(ref v)) => {
                serde::Deserializer::deserialize_any(MapRefDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqRefDeserializer<'de> {
    iter: slice::Iter<'de, Value>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [Value]) -> Self {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> serde::Deserializer<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = try!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(
                    len,
                    &"fewer elements in vector",
                ))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapRefDeserializer<'de> {
    iter: <&'de Map<String, Value> as IntoIterator>::IntoIter,
    value: Option<&'de Value>,
}

impl<'de> MapRefDeserializer<'de> {
    fn new(map: &'de Map<String, Value>) -> Self {
        MapRefDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapRefDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Borrowed(&**key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapRefDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MapKeyDeserializer<'de> {
    key: Cow<'de, str>,
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match (self.key.parse(), self.key) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
                (Err(_), Cow::Owned(s)) => visitor.visit_string(s),
            }
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        BorrowedCowStrDeserializer::new(self.key).deserialize_any(visitor)
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);

    serde_if_integer128! {
        deserialize_integer_key!(deserialize_i128 => visit_i128);
        deserialize_integer_key!(deserialize_u128 => visit_u128);
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        self.key
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct KeyClassifier;

enum KeyClass {
    Map(String),
    #[cfg(feature = "arbitrary_precision")]
    Number,
    #[cfg(feature = "raw_value")]
    RawValue,
    KeywordHack,
    SymbolHack,
}

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        match s {
            ::symbol::TOKEN => Ok(KeyClass::SymbolHack),
            ::keyword::TOKEN => Ok(KeyClass::KeywordHack),
            #[cfg(feature = "arbitrary_precision")]
            ::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            ::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s.to_owned())),
        }
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        match s.as_str() {
            ::symbol::TOKEN => Ok(KeyClass::SymbolHack),
            ::keyword::TOKEN => Ok(KeyClass::KeywordHack),
            #[cfg(feature = "arbitrary_precision")]
            ::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            ::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s)),
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UnexpectedEDN<'a> {
    Symbol(&'a str),
    Keyword(&'a str),
}

impl Value {
    #[cold]
    fn invalid_type<E>(&self, exp: &Expected) -> E
        where
            E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected {
        match *self {
            Value::Nil => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(b),
            Value::Number(ref n) => n.unexpected(),
            Value::String(ref s) => Unexpected::Str(s),
            Value::Vector(_) => Unexpected::Seq,
            Value::List(_) => Unexpected::Seq,
            Value::Object(_) => Unexpected::Map,
            Value::Keyword(ref s) => Unexpected::Other("keyword"),
            Value::Symbol(ref s) => Unexpected::Other("symbol")
//            Value::Keyword(ref s) => UnexpectedEDN::Keyword(s),
//            Value::Symbol(ref s) => UnexpectedEDN::Symbol(s)
        }
    }
}

struct BorrowedCowStrDeserializer<'de> {
    value: Cow<'de, str>,
}

impl<'de> BorrowedCowStrDeserializer<'de> {
    fn new(value: Cow<'de, str>) -> Self {
        BorrowedCowStrDeserializer { value: value }
    }
}

impl<'de> de::Deserializer<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> de::EnumAccess<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = Error;
    type Variant = UnitOnly;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
        where
            T: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self)?;
        Ok((value, UnitOnly))
    }
}

struct UnitOnly;

impl<'de> de::VariantAccess<'de> for UnitOnly {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}
