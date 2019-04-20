// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::ops;

use super::Value;
use map::Map;

/// A type that can be used to index into a `serde_edn::Value`.
///
/// The [`get`] and [`get_mut`] methods of `Value` accept any type that
/// implements `Index`, as does the [square-bracket indexing operator]. This
/// trait is implemented for strings which are used as the index into a edn
/// map, and for `usize` which is used as the index into a edn vector.
///
/// [`get`]: ../enum.Value.html#method.get
/// [`get_mut`]: ../enum.Value.html#method.get_mut
/// [square-bracket indexing operator]: ../enum.Value.html#impl-Index%3CI%3E
///
/// This trait is sealed and cannot be implemented for types outside of
/// `serde_edn`.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_edn;
/// #
/// # fn main() {
/// let data = edn!({ "inner": [1, 2, 3] });
///
/// // Data is a edn map so it can be indexed with a string.
/// let inner = &data["inner"];
///
/// // Inner is a edn vector so it can be indexed with an integer.
/// let first = &inner[0];
///
/// assert_eq!(first, 1);
/// # }
/// ```
pub trait Index: private::Sealed {
    /// Return None if the key is not already in the vector or object.
    #[doc(hidden)]
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value>;

    /// Return None if the key is not already in the vector or object.
    #[doc(hidden)]
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value>;

    /// Panic if vector index out of bounds. If key is not already in the object,
    /// insert it with a value of null. Panic if Value is a type that cannot be
    /// indexed into, except if Value is null then it can be treated as an empty
    /// object.
    #[doc(hidden)]
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value;
}

impl Index for usize {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match *v {
            Value::Vector(ref vec) => vec.get(*self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match *v {
            Value::Vector(ref mut vec) => vec.get_mut(*self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        match *v {
            Value::Vector(ref mut vec) => {
                let len = vec.len();
                vec.get_mut(*self).unwrap_or_else(|| {
                    panic!(
                        "cannot access index {} of edn vector of length {}",
                        self, len
                    )
                })
            }
            _ => panic!("cannot access index {} of edn {}", self, Type(v)),
        }
    }
}

impl Index for str {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match *v {
            Value::Object(ref map) => map.get(self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match *v {
            Value::Object(ref mut map) => map.get_mut(self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        if let Value::Nil = *v {
            *v = Value::Object(Map::new());
        }
        match *v {
            Value::Object(ref mut map) => map.entry(self.to_owned()).or_insert(Value::Nil),
            _ => panic!("cannot access key {:?} in edn {}", self, Type(v)),
        }
    }
}

impl Index for String {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        self[..].index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        self[..].index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        self[..].index_or_insert(v)
    }
}

impl<'a, T: ?Sized> Index for &'a T
where
    T: Index,
{
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        (**self).index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        (**self).index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        (**self).index_or_insert(v)
    }
}

// Prevent users from implementing the Index trait.
mod private {
    pub trait Sealed {}
    impl Sealed for usize {}
    impl Sealed for str {}
    impl Sealed for String {}
    impl<'a, T: ?Sized> Sealed for &'a T where T: Sealed {}
}

/// Used in panic messages.
struct Type<'a>(&'a Value);

impl<'a> fmt::Display for Type<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            Value::Nil => formatter.write_str("null"),
            Value::Bool(_) => formatter.write_str("boolean"),
            Value::Number(_) => formatter.write_str("number"),
            Value::String(_) => formatter.write_str("string"),
            Value::Vector(_) => formatter.write_str("vector"),
            Value::Object(_) => formatter.write_str("object"),
        }
    }
}

// The usual semantics of Index is to panic on invalid indexing.
//
// That said, the usual semantics are for things like Vec and BTreeMap which
// have different use cases than Value. If you are working with a Vec, you know
// that you are working with a Vec and you can get the len of the Vec and make
// sure your indices are within bounds. The Value use cases are more
// loosey-goosey. You got some edn from an endpoint and you want to pull values
// out of it. Outside of this Index impl, you already have the option of using
// value.as_vector() and working with the Vec directly, or matching on
// Value::Vector and getting the Vec directly. The Index impl means you can skip
// that and index directly into the thing using a concise syntax. You don't have
// to check the type, you don't have to check the len, it is all about what you
// expect the Value to look like.
//
// Basically the use cases that would be well served by panicking here are
// better served by using one of the other approaches: get and get_mut,
// as_vector, or match. The value of this impl is that it adds a way of working
// with Value that is not well served by the existing approaches: concise and
// careless and sometimes that is exactly what you want.
impl<I> ops::Index<I> for Value
where
    I: Index,
{
    type Output = Value;

    /// Index into a `serde_edn::Value` using the syntax `value[0]` or
    /// `value["k"]`.
    ///
    /// Returns `Value::Nil` if the type of `self` does not match the type of
    /// the index, for example if the index is a string and `self` is a vector
    /// or a number. Also returns `Value::Nil` if the given key does not exist
    /// in the map or the given index is not within the bounds of the vector.
    ///
    /// For retrieving deeply nested values, you should have a look at the
    /// `Value::pointer` method.
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
    /// assert_eq!(data["x"]["y"], edn!(["z", "zz"]));
    /// assert_eq!(data["x"]["y"][0], edn!("z"));
    ///
    /// assert_eq!(data["a"], edn!(nil)); // returns null for undefined values
    /// assert_eq!(data["a"]["b"], edn!(nil)); // does not panic
    /// # }
    /// ```
    fn index(&self, index: I) -> &Value {
        static NULL: Value = Value::Nil;
        index.index_into(self).unwrap_or(&NULL)
    }
}

impl<I> ops::IndexMut<I> for Value
where
    I: Index,
{
    /// Write into a `serde_edn::Value` using the syntax `value[0] = ...` or
    /// `value["k"] = ...`.
    ///
    /// If the index is a number, the value must be a vector of length bigger
    /// than the index. Indexing into a value that is not a vector or a vector
    /// that is too small will panic.
    ///
    /// If the index is a string, the value must be an object or null which is
    /// treated like an empty object. If the key is not already present in the
    /// object, it will be inserted with a value of null. Indexing into a value
    /// that is neither an object nor null will panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut data = edn!({ "x": 0 });
    ///
    /// // replace an existing key
    /// data["x"] = edn!(1);
    ///
    /// // insert a new key
    /// data["y"] = edn!([false, false, false]);
    ///
    /// // replace a vector value
    /// data["y"][0] = edn!(true);
    ///
    /// // inserted a deeply nested key
    /// data["a"]["b"]["c"]["d"] = edn!(true);
    ///
    /// println!("{}", data);
    /// # }
    /// ```
    fn index_mut(&mut self, index: I) -> &mut Value {
        index.index_or_insert(self)
    }
}
