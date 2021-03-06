// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use super::Value;
use map::{Map};
use number::Number;

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(n.into())
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
}

#[cfg(feature = "arbitrary_precision")]
serde_if_integer128! {
    from_integer! {
        i128 u128
    }
}

impl From<f32> for Value {
    /// Convert 32-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let f: f32 = 13.37;
    /// let x: Value = f.into();
    /// # }
    /// ```
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    /// Convert 64-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let f: f64 = 13.37;
    /// let x: Value = f.into();
    /// # }
    /// ```
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Value::Nil, Value::Number)
    }
}

impl From<bool> for Value {
    /// Convert boolean to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let b = false;
    /// let x: Value = b.into();
    /// # }
    /// ```
    fn from(f: bool) -> Self {
        Value::Bool(f)
    }
}

impl From<String> for Value {
    /// Convert `String` to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let s: String = "lorem".to_string();
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: String) -> Self {
        Value::String(f)
    }
}

impl<'a> From<&'a str> for Value {
    /// Convert string slice to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let s: &str = "lorem";
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: &str) -> Self {
        Value::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    /// Convert copy-on-write string to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Value = s.into();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_string());
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        Value::String(f.into_owned())
    }
}

impl From<Map<Value, Value>> for Value {
    /// Convert map to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::{MapInternal, Value};
    ///
    /// let mut m = MapInternal::new();
    /// m.insert("Lorem".to_string(), "ipsum".into());
    /// let x: Value = m.into();
    /// # }
    /// ```
    fn from(f: Map<Value, Value>) -> Self {
        Value::Object(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Convert a `Vec` to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// # }
    /// ```
    fn from(f: Vec<T>) -> Self {
        Value::Vector(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Value>> From<&'a [T]> for Value {
    /// Convert a slice to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// # }
    /// ```
    fn from(f: &'a [T]) -> Self {
        Value::Vector(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Value>> ::std::iter::FromIterator<T> for Value {
    /// Convert an iteratable type to a `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Value = v.collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::Value;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into_iter().collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use std::iter::FromIterator;
    /// use serde_edn::Value;
    ///
    /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// # }
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::Vector(iter.into_iter().map(Into::into).collect())
    }
}
