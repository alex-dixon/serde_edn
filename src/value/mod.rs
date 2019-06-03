// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The Value enum, a loosely typed way of representing any valid edn value.
//!
//! # Constructing edn
//!
//! Serde edn provides a [`edn!` macro][macro] to build `serde_edn::Value`
//! objects with very natural edn syntax. In order to use this macro,
//! `serde_edn` needs to be imported with the `#[macro_use]` attribute.
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_edn;
//!
//! fn main() {
//!     // The type of `john` is `serde_edn::Value`
//!     let john = edn!({
//!       "name": "John Doe",
//!       "age": 43,
//!       "phones": [
//!         "+44 1234567",
//!         "+44 2345678"
//!       ]
//!     });
//!
//!     println!("first phone number: {}", john["phones"][0]);
//!
//!     // Convert to a string of edn and print it out
//!     println!("{}", john.to_string());
//! }
//! ```
//!
//! The `Value::to_string()` function converts a `serde_edn::Value` into a
//! `String` of edn text.
//!
//! One neat thing about the `edn!` macro is that variables and expressions can
//! be interpolated directly into the edn value as you are building it. Serde
//! will check at compile time that the value you are interpolating is able to
//! be represented as edn.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate serde_edn;
//! #
//! # fn random_phone() -> u16 { 0 }
//! #
//! # fn main() {
//! let full_name = "John Doe";
//! let age_last_year = 42;
//!
//! // The type of `john` is `serde_edn::Value`
//! let john = edn!({
//!   "name": full_name,
//!   "age": age_last_year + 1,
//!   "phones": [
//!     format!("+44 {}", random_phone())
//!   ]
//! });
//! #     let _ = john;
//! # }
//! ```
//!
//! A string of edn data can be parsed into a `serde_edn::Value` by the
//! [`serde_edn::from_str`][from_str] function. There is also
//! [`from_slice`][from_slice] for parsing from a byte slice `&[u8]` and
//! [`from_reader`][from_reader] for parsing from any `io::Read` like a File or
//! a TCP stream.
//!
//! ```rust
//! extern crate serde_edn;
//!
//! use serde_edn::{Value, Error};
//!
//! fn untyped_example() -> Result<(), Error> {
//!     // Some edn input data as a &str. Maybe this comes from the user.
//!     let data = r#"{
//!                     "name" "John Doe",
//!                     "age" 43,
//!                     "phones" [
//!                       "+44 1234567",
//!                       "+44 2345678"
//!                     ]
//!                   }"#;
//!
//!     // Parse the string of data into serde_edn::Value.
//!     let v: Value = serde_edn::from_str(data)?;
//!
//!     // Access parts of the data by indexing with square brackets.
//!     println!("Please call {} at the number {}", v["name"], v["phones"][0]);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     untyped_example().unwrap();
//! # }
//! ```
//!
//! [macro]: https://docs.serde.rs/serde_edn/macro.edn.html
//! [from_str]: https://docs.serde.rs/serde_edn/de/fn.from_str.html
//! [from_slice]: https://docs.serde.rs/serde_edn/de/fn.from_slice.html
//! [from_reader]: https://docs.serde.rs/serde_edn/de/fn.from_reader.html

use std::fmt::{self, Debug};
use std::io;
use std::mem;
use std::str;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use error::Error;
pub use map::Map;
pub use number::Number;

#[cfg(feature = "raw_value")]
pub use raw::RawValue;

pub use self::index::Index;

use self::ser::Serializer;
pub use symbol::Symbol;
pub use keyword::Keyword;
use edn_ser::EDNSerialize;

/// Represents any valid edn value.
///
/// See the `serde_edn::value` module documentation for usage examples.
#[derive(Clone, PartialEq)]
pub enum Value {
    /// Represents a edn null value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!(nil);
    /// # }
    /// ```
    Nil,

    /// Represents a edn boolean.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!(true);
    /// # }
    /// ```
    Bool(bool),

    /// Represents a edn number, whether integer or floating point.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!(12.5);
    /// # }
    /// ```
    Number(Number),

    /// Represents a edn string.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!("a string");
    /// # }
    /// ```
    String(String),

    /// Represents an edn vector.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!(["a", "vector"]);
    /// # }
    /// ```
    Vector(Vec<Value>),
    List(Vec<Value>),
    Set(Vec<Value>),

    /// Represents an edn map.
    ///
    /// By default the map is backed by a BTreeMap. Enable the `preserve_order`
    /// feature of serde_edn to use IndexMap instead, which preserves
    /// entries in the order they are inserted into the map. In particular, this
    /// allows edn data to be deserialized into a Value and serialized to a
    /// string while retaining the order of map keys in the input.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "an": "object" });
    /// # }
    /// ```
    Object(Map<String, Value>),
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!(":somekeyword");
    /// # }
    /// ```
    Keyword(Keyword),
    //    Keyword2(String),
    Symbol(Symbol),
}

impl PartialEq<&Value> for Value {
    fn eq(&self, &other: &&Value) -> bool {
        false
//        PartialEq::eq(&self, *&other)
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Nil => formatter.debug_tuple("Nil").finish(),
            Value::Bool(v) => formatter.debug_tuple("Bool").field(&v).finish(),
            Value::Number(ref v) => Debug::fmt(v, formatter),
            Value::String(ref v) => formatter.debug_tuple("String").field(v).finish(),
            Value::Vector(ref v) => formatter.debug_tuple("Vector").field(v).finish(),
            Value::List(ref v) => formatter.debug_tuple("List").field(v).finish(),
            Value::Set(ref v) => formatter.debug_tuple("Set").field(v).finish(),
            Value::Object(ref v) => formatter.debug_tuple("Object").field(v).finish(),
            Value::Keyword(ref v) => Debug::fmt(v, formatter),
            Value::Symbol(ref v) => Debug::fmt(v, formatter),
        }
    }
}

struct WriterFormatter<'a, 'b: 'a> {
    inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> io::Write for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        fn io_error<E>(_: E) -> io::Error {
            // Error value does not matter because fmt::Display impl below just
            // maps it to fmt::Error
            io::Error::new(io::ErrorKind::Other, "fmt error")
        }
        let s = try!(str::from_utf8(buf).map_err(io_error));
        try!(self.inner.write_str(s).map_err(io_error));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for Value {
    /// Display a edn value as a string.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let edn = edn!({ "city": "London", "street": "10 Downing Street" });
    ///
    /// // Compact format:
    /// //
    /// // {"city" "London" "street" "10 Downing Street"}
    /// let compact = format!("{}", edn);
    /// assert_eq!(compact,
    ///     "{\"city\" \"London\" \"street\" \"10 Downing Street\"}");
    ///
    /// // Pretty format:
    /// //
    /// // {
    /// //   "city" "London",
    /// //   "street" "10 Downing Street"
    /// // }
    /// let pretty = format!("{:#}", edn);
    /// assert_eq!(pretty,
    ///     "{\n  \"city\" \"London\"\n  \"street\" \"10 Downing Street\"\n}");
    /// # }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alternate = f.alternate();
        let mut wr = WriterFormatter { inner: f };
        if alternate {
            // {:#}
            super::ser::to_writer_pretty(&mut wr, self).map_err(|_| fmt::Error)
        } else {
            // {}
            super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
        }
    }
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}

impl Value {
    /// Index into a edn vector or map. A string index can be used to access a
    /// value in a map, and a usize index can be used to access an element of a
    /// vector.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is a vector or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the vector.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let object = edn!({ "A": 65, "B": 66, "C": 67 });
    /// assert_eq!(*object.get("A").unwrap(), edn!(65));
    ///
    /// let vector = edn!([ "A", "B", "C" ]);
    /// assert_eq!(*vector.get(2).unwrap(), edn!("C"));
    ///
    /// assert_eq!(vector.get("A"), None);
    /// # }
    /// ```
    ///
    /// Square brackets can also be used to index into a value in a more concise
    /// way. This returns `Value::Nil` in cases where `get` would have returned
    /// `None`.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let object = edn!({
    ///     "A": ["a", "á", "à"],
    ///     "B": ["b", "b́"],
    ///     "C": ["c", "ć", "ć̣", "ḉ"],
    /// });
    /// assert_eq!(object["B"][0], edn!("b"));
    ///
    /// assert_eq!(object["D"], edn!(nil));
    /// assert_eq!(object[0]["x"]["y"]["z"], edn!(nil));
    /// # }
    /// ```
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }

    /// Mutably index into a edn vector or map. A string index can be used to
    /// access a value in a map, and a usize index can be used to access an
    /// element of an vector.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is an vector or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the vector.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut object = edn!({ "A": 65, "B": 66, "C": 67 });
    /// *object.get_mut("A").unwrap() = edn!(69);
    ///
    /// let mut vector = edn!([ "A", "B", "C" ]);
    /// *vector.get_mut(2).unwrap() = edn!("D");
    /// # }
    /// ```
    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut Value> {
        index.index_into_mut(self)
    }

    /// Returns true if the `Value` is an Object. Returns false otherwise.
    ///
    /// For any Value on which `is_object` returns true, `as_object` and
    /// `as_object_mut` are guaranteed to return the map representation of the
    /// object.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let obj = edn!({ "a": { "nested": true }, "b": ["a", "vector"] });
    ///
    /// assert!(obj.is_object());
    /// assert!(obj["a"].is_object());
    ///
    /// // vector, not an object
    /// assert!(!obj["b"].is_object());
    /// # }
    /// ```
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// If the `Value` is an Object, returns the associated Map. Returns None
    /// otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": { "nested": true }, "b": ["a", "vector"] });
    ///
    /// // The length of `{"nested": true}` is 1 entry.
    /// assert_eq!(v["a"].as_object().unwrap().len(), 1);
    ///
    /// // The vector `["a", "vector"]` is not an object.
    /// assert_eq!(v["b"].as_object(), None);
    /// # }
    /// ```
    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match *self {
            Value::Object(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut v = edn!({ "a": { "nested": true } });
    ///
    /// v["a"].as_object_mut().unwrap().clear();
    /// assert_eq!(v, edn!({ "a": {} }));
    /// # }
    ///
    /// ```
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match *self {
            Value::Object(ref mut map) => Some(map),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Vector. Returns false otherwise.
    ///
    /// For any Value on which `is_vector` returns true, `as_vector` and
    /// `as_vector_mut` are guaranteed to return the vector representing the
    /// vector.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let obj = edn!({ "a": ["a", "vector"], "b": { "an": "object" } });
    ///
    /// assert!(obj["a"].is_vector());
    ///
    /// // an object, not a vector
    /// assert!(!obj["b"].is_vector());
    /// # }
    /// ```
    pub fn is_vector(&self) -> bool {
        self.as_vector().is_some()
    }

    /// If the `Value` is a Vector, returns the associated vector. Returns None
    /// otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": ["a", "vector"], "b": { "an": "object" } });
    ///
    /// // The length of `["a", "vector"]` is 2 elements.
    /// assert_eq!(v["a"].as_vector().unwrap().len(), 2);
    ///
    /// // The object `{"an": "object"}` is not an vector.
    /// assert_eq!(v["b"].as_vector(), None);
    /// # }
    /// ```
    pub fn as_vector(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Vector(ref v) => Some(&*v),
            _ => None,
        }
    }

    /// If the `Value` is a Vector, returns the associated mutable vector.
    /// Returns None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut v = edn!({ "a": ["a", "vector"] });
    ///
    /// v["a"].as_vector_mut().unwrap().clear();
    /// assert_eq!(v, edn!({ "a": [] }));
    /// # }
    /// ```
    pub fn as_vector_mut(&mut self) -> Option<&mut Vec<Value>> {
        match *self {
            Value::Vector(ref mut list) => Some(list),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// For any Value on which `is_string` returns true, `as_str` is guaranteed
    /// to return the string slice.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": "some string", "b": false });
    ///
    /// assert!(v["a"].is_string());
    ///
    /// // The boolean `false` is not a string.
    /// assert!(!v["b"].is_string());
    /// # }
    /// ```
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// If the `Value` is a String, returns the associated str. Returns None
    /// otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": "some string", "b": false });
    ///
    /// assert_eq!(v["a"].as_str(), Some("some string"));
    ///
    /// // The boolean `false` is not a string.
    /// assert_eq!(v["b"].as_str(), None);
    ///
    /// // edn values are printed in edn representation, so strings are in quotes.
    /// //
    /// //    The value is: "some string"
    /// println!("The value is: {}", v["a"]);
    ///
    /// // Rust strings are printed without quotes.
    /// //
    /// //    The value is: some string
    /// println!("The value is: {}", v["a"].as_str().unwrap());
    /// # }
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn is_keyword(&self) -> bool {
        self.as_keyword().is_some()
    }

    pub fn as_keyword(&self) -> Option<&Keyword> {
        match *self {
            Value::Keyword(ref s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": 1, "b": "2" });
    ///
    /// assert!(v["a"].is_number());
    ///
    /// // The string `"2"` is a string, not a number.
    /// assert!(!v["b"].is_number());
    /// # }
    /// ```
    pub fn is_number(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Value on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let big = i64::max_value() as u64 + 10;
    /// let v = edn!({ "a": 64, "b": big, "c": 256.0 });
    ///
    /// assert!(v["a"].is_i64());
    ///
    /// // Greater than i64::MAX.
    /// assert!(!v["b"].is_i64());
    ///
    /// // Numbers with a decimal point are not considered integers.
    /// assert!(!v["c"].is_i64());
    /// # }
    /// ```
    pub fn is_i64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_i64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between zero and `u64::MAX`.
    ///
    /// For any Value on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": 64, "b": -64, "c": 256.0 });
    ///
    /// assert!(v["a"].is_u64());
    ///
    /// // Negative integer.
    /// assert!(!v["b"].is_u64());
    ///
    /// // Numbers with a decimal point are not considered integers.
    /// assert!(!v["c"].is_u64());
    /// # }
    /// ```
    pub fn is_u64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_u64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number that can be represented by f64.
    ///
    /// For any Value on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    ///
    /// Currently this function returns true if and only if both `is_i64` and
    /// `is_u64` return false but this is not a guarantee in the future.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": 256.0, "b": 64, "c": -64 });
    ///
    /// assert!(v["a"].is_f64());
    ///
    /// // Integers.
    /// assert!(!v["b"].is_f64());
    /// assert!(!v["c"].is_f64());
    /// # }
    /// ```
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_f64(),
            _ => false,
        }
    }

    /// If the `Value` is an integer, represent it as i64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let big = i64::max_value() as u64 + 10;
    /// let v = edn!({ "a": 64, "b": big, "c": 256.0 });
    ///
    /// assert_eq!(v["a"].as_i64(), Some(64));
    /// assert_eq!(v["b"].as_i64(), None);
    /// assert_eq!(v["c"].as_i64(), None);
    /// # }
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Number(ref n) => n.as_i64(),
            _ => None,
        }
    }

    /// If the `Value` is an integer, represent it as u64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": 64, "b": -64, "c": 256.0 });
    ///
    /// assert_eq!(v["a"].as_u64(), Some(64));
    /// assert_eq!(v["b"].as_u64(), None);
    /// assert_eq!(v["c"].as_u64(), None);
    /// # }
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Number(ref n) => n.as_u64(),
            _ => None,
        }
    }

    /// If the `Value` is a number, represent it as f64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": 256.0, "b": 64, "c": -64 });
    ///
    /// assert_eq!(v["a"].as_f64(), Some(256.0));
    /// assert_eq!(v["b"].as_f64(), Some(64.0));
    /// assert_eq!(v["c"].as_f64(), Some(-64.0));
    /// # }
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Number(ref n) => n.as_f64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// For any Value on which `is_boolean` returns true, `as_bool` is
    /// guaranteed to return the boolean value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": false, "b": "false" });
    ///
    /// assert!(v["a"].is_boolean());
    ///
    /// // The string `"false"` is a string, not a boolean.
    /// assert!(!v["b"].is_boolean());
    /// # }
    /// ```
    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool. Returns None
    /// otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": false, "b": "false" });
    ///
    /// assert_eq!(v["a"].as_bool(), Some(false));
    ///
    /// // The string `"false"` is a string, not a boolean.
    /// assert_eq!(v["b"].as_bool(), None);
    /// # }
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Nil. Returns false otherwise.
    ///
    /// For any Value on which `is_null` returns true, `as_null` is guaranteed
    /// to return `Some(())`.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": nil, "b": false });
    ///
    /// assert!(v["a"].is_null());
    ///
    /// // The boolean `false` is not null.
    /// assert!(!v["b"].is_null());
    /// # }
    /// ```
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the `Value` is a Nil, returns (). Returns None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let v = edn!({ "a": nil, "b": false });
    ///
    /// assert_eq!(v["a"].as_null(), Some(()));
    ///
    /// // The boolean `false` is not null.
    /// assert_eq!(v["b"].as_null(), None);
    /// # }
    /// ```
    pub fn as_null(&self) -> Option<()> {
        match *self {
            Value::Nil => Some(()),
            _ => None,
        }
    }

    /// Looks up a value by a edn Pointer.
    ///
    /// edn Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (edn) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let data = edn!({
    ///     "x": {
    ///         "y": ["z", "zz"]
    ///     }
    /// });
    ///
    /// assert_eq!(data.pointer("/x/y/1").unwrap(), &edn!("zz"));
    /// assert_eq!(data.pointer("/a/b/c"), None);
    /// # }
    /// ```
    pub fn pointer<'a>(&'a self, pointer: &str) -> Option<&'a Value> {
        if pointer == "" {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let tokens = pointer
            .split('/')
            .skip(1)
            .map(|x| x.replace("~1", "/").replace("~0", "~"));
        let mut target = self;

        for token in tokens {
            let target_opt = match *target {
                Value::Object(ref map) => map.get(&token),
                Value::Vector(ref list) => parse_index(&token).and_then(|x| list.get(x)),
                _ => return None,
            };
            if let Some(t) = target_opt {
                target = t;
            } else {
                return None;
            }
        }
        Some(target)
    }

    /// Looks up a value by a edn Pointer and returns a mutable reference to
    /// that value.
    ///
    /// edn Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (edn) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
    ///
    /// # Example of Use
    ///
    /// ```rust
    /// extern crate serde_edn;
    ///
    /// use serde_edn::Value;
    ///
    /// fn main() {
    ///     let s = r#"{"x" 1.0, "y" 2.0}"#;
    ///     let mut value: Value = serde_edn::from_str(s).unwrap();
    ///
    ///     // Check value using read-only pointer
    ///     assert_eq!(value.pointer("/x"), Some(&1.0.into()));
    ///     // Change value with direct assignment
    ///     *value.pointer_mut("/x").unwrap() = 1.5.into();
    ///     // Check that new value was written
    ///     assert_eq!(value.pointer("/x"), Some(&1.5.into()));
    ///
    ///     // "Steal" ownership of a value. Can replace with any valid Value.
    ///     let old_x = value.pointer_mut("/x").map(Value::take).unwrap();
    ///     assert_eq!(old_x, 1.5);
    ///     assert_eq!(value.pointer("/x").unwrap(), &Value::Nil);
    /// }
    /// ```
    pub fn pointer_mut<'a>(&'a mut self, pointer: &str) -> Option<&'a mut Value> {
        if pointer == "" {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let tokens = pointer
            .split('/')
            .skip(1)
            .map(|x| x.replace("~1", "/").replace("~0", "~"));
        let mut target = self;

        for token in tokens {
            // borrow checker gets confused about `target` being mutably borrowed too many times because of the loop
            // this once-per-loop binding makes the scope clearer and circumvents the error
            let target_once = target;
            let target_opt = match *target_once {
                Value::Object(ref mut map) => map.get_mut(&token),
                Value::Vector(ref mut list) => {
                    parse_index(&token).and_then(move |x| list.get_mut(x))
                }
                _ => return None,
            };
            if let Some(t) = target_opt {
                target = t;
            } else {
                return None;
            }
        }
        Some(target)
    }

    /// Takes the value out of the `Value`, leaving a `Nil` in its place.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut v = edn!({ "x": "y" });
    /// assert_eq!(v["x"].take(), edn!("y"));
    /// assert_eq!(v, edn!({ "x": nil }));
    /// # }
    /// ```
    pub fn take(&mut self) -> Value {
        mem::replace(self, Value::Nil)
    }
}

/// The default value is `Value::Nil`.
///
/// This is useful for handling omitted `Value` fields when deserializing.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # extern crate serde_edn;
/// #
/// use serde_edn::Value;
///
/// #[derive(Deserialize)]
/// struct Settings {
///     level: i32,
///     #[serde(default)]
///     extras: Value,
/// }
///
/// # fn try_main() -> Result<(), serde_edn::Error> {
/// let data = r#" { "level" 42 } "#;
/// let s: Settings = serde_edn::from_str(data)?;
///
/// assert_eq!(s.level, 42);
/// assert_eq!(s.extras, Value::Nil);
/// #
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap()
/// # }
/// ```
impl Default for Value {
    fn default() -> Value {
        Value::Nil
    }
}

mod de;
mod from;
mod index;
mod partial_eq;
mod ser;

/// Convert a `T` into `serde_edn::Value` which is an enum that can represent
/// any valid edn data.
///
/// # Example
///
/// ```rust
/// extern crate serde;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate serde_edn;
///
/// use std::error::Error;
///
/// #[derive(Serialize)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn compare_edn_values() -> Result<(), Box<Error>> {
///     let u = User {
///         fingerprint: "0xF9BA143B95FF6D82".to_owned(),
///         location: "Menlo Park, CA".to_owned(),
///     };
///
///     // The type of `expected` is `serde_edn::Value`
///     let expected = edn!({
///                            "fingerprint": "0xF9BA143B95FF6D82",
///                            "location": "Menlo Park, CA",
///                          });
///
///     let v = serde_edn::to_value(u).unwrap();
///     assert_eq!(v, expected);
///
///     Ok(())
/// }
/// #
/// # fn main() {
/// #     compare_edn_values().unwrap();
/// # }
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
///
/// ```rust
/// extern crate serde_edn;
///
/// use std::collections::BTreeMap;
///
/// fn main() {
///     // The keys in this map are vectors, not strings.
///     let mut map = BTreeMap::new();
///     map.insert(vec![32, 64], "x86");
///
///     println!("{}", serde_edn::to_value(map).unwrap_err());
/// }
/// ```
// Taking by value is more friendly to iterator adapters, option and result
// consumers, etc. See https://github.com/serde-rs/edn/pull/149.
pub fn to_value<T>(value: T) -> Result<Value, Error>
    where
        T: Serialize,
{
    value.serialize(Serializer)
}

/// Interpret a `serde_edn::Value` as an instance of type `T`.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate serde_edn;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// extern crate serde;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn main() {
///     // The type of `j` is `serde_edn::Value`
///     let j = edn!({
///                     "fingerprint": "0xF9BA143B95FF6D82",
///                     "location": "Menlo Park, CA"
///                   });
///
///     let u: User = serde_edn::from_value(j).unwrap();
///     println!("{:#?}", u);
/// }
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a edn map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the edn map or some number is too big to fit in the expected primitive
/// type.
pub fn from_value<T>(value: Value) -> Result<T, Error>
    where
        T: DeserializeOwned,
{
    T::deserialize(value)
}
