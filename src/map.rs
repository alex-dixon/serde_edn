// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A map of String to serde_edn::Value.
//!
//! By default the map is backed by a [`BTreeMap`]. Enable the `preserve_order`
//! feature of serde_edn to use [`IndexMap`] instead.
//!
//! [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
//! [`IndexMap`]: https://docs.rs/indexmap/*/indexmap/map/struct.IndexMap.html

use serde::{de, ser};
use std::borrow::Borrow;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops;
use value::Value;

#[cfg(not(feature = "preserve_order"))]
use std::collections::{btree_map, BTreeMap,hash_map,HashMap};

#[cfg(feature = "preserve_order")]
use indexmap::{self, IndexMap};


macro_rules! delegate_iterator {
    (($name:ident $($generics:tt)*) => $item:ty) => {
        impl $($generics)* Iterator for $name $($generics)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl $($generics)* DoubleEndedIterator for $name $($generics)* {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl $($generics)* ExactSizeIterator for $name $($generics)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }
    }
}

#[cfg(not(feature = "preserve_order"))]
type MapImpl<K, V> = HashMap<K, V>;
#[cfg(feature = "preserve_order")]
type MapImpl<K, V> = IndexMap<K, V>;

pub struct Map <K,V> {
    map: MapImpl<K,V>
}
impl Map<Value,Value> {
    #[inline]
    pub fn new() -> Self { Map { map: MapImpl::new(), } }

    #[inline]
    pub fn get(&self, key: &Value) -> Option<&Value>
    {
        self.map.get(key)
    }
    pub fn get_mut(&mut self, key: &Value) -> Option<&mut Value>
    {
        self.map.get_mut(key)
    }

    #[inline]
    pub fn insert(&mut self, k: Value, v: Value) -> Option<Value> {
        self.map.insert(k, v)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Gets an iterator over the entries of the map.
    #[inline]
    pub fn iter(&self) -> MapIter {
        MapIter {
            iter: self.map.iter(),
        }
    }

    pub fn entry<S>(&mut self, key: S) -> EDNEntry
        where
            S: Into<Value>,
    {
        #[cfg(feature = "preserve_order")]
        use indexmap::map::Entry as EntryImpl;
        #[cfg(not(feature = "preserve_order"))]
        use std::collections::hash_map::Entry as EntryImpl;


        match self.map.entry(key.into()) {
            EntryImpl::Vacant(vacant) => EDNEntry::Vacant(EDNVacantEntry { vacant: vacant }),
            EntryImpl::Occupied(occupied) => EDNEntry::Occupied(EDNOccupiedEntry { occupied: occupied }),
        }
    }
}

impl Hash for Map<Value, Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // todo. perf?
        for t in self.iter() {
            t.hash(state)
        }
    }
}


#[cfg(not(feature = "preserve_order"))]
type MapIterImpl<'a> = hash_map::Iter<'a, Value, Value>;
#[cfg(feature = "preserve_order")]
type MapIterImpl<'a> = indexmap::map::Iter<'a, Value, Value>;

pub struct MapIter<'a> {
    iter: MapIterImpl<'a>,
}

//delegate_iterator!((MapIter<'a>) => (&'a Value, &'a Value));
impl<'a> Iterator for MapIter<'a> {
    type Item = (&'a Value, &'a Value);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
//impl<'a> DoubleEndedIterator for MapIter<'a> {
//    #[inline]
//    fn next_back(&mut self) -> Option<Self::Item> {
//        self.iter.next_back()
//    }
//}
impl<'a> ExactSizeIterator for MapIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}


impl<'a> IntoIterator for &'a Map<Value, Value> {
    type Item = (&'a Value, &'a Value);
    type IntoIter = MapIter<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        MapIter {
            iter: self.map.iter(),
        }
    }

}
impl Clone for Map<Value, Value> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

impl PartialEq for Map<Value, Value> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if cfg!(feature = "preserve_order") {
            if self.len() != other.len() {
                return false;
            }

            self.iter()
                .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
        } else {
            self.map.eq(&other.map)
        }
    }
}
impl Debug for Map<Value, Value> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.map.fmt(formatter)
    }
}

#[cfg(not(feature = "preserve_order"))]
type MapIntoIterImpl = hash_map::IntoIter<Value, Value>;
#[cfg(feature = "preserve_order")]
type MapIntoIterImpl = indexmap::map::IntoIter<Value, Value>;

pub struct MapIntoIter {
    iter: MapIntoIterImpl,
}

//delegate_iterator!((MapIntoIter) => (Value, Value));
impl Iterator for MapIntoIter {
    type Item = (Value, Value);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
//impl DoubleEndedIterator for MapIntoIter {
//    #[inline]
//    fn next_back(&mut self) -> Option<Self::Item> {
//        self.iter.next_back()
//    }
//}
impl ExactSizeIterator for MapIntoIter {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}



// entry
pub enum EDNEntry<'a> {
    Vacant(EDNVacantEntry<'a>),
    Occupied(EDNOccupiedEntry<'a>),
}

pub struct EDNVacantEntry<'a> {
    vacant: EDNVacantEntryImpl<'a>,
}

pub struct EDNOccupiedEntry<'a> {
    occupied: EDNOccupiedEntryImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type EDNVacantEntryImpl<'a> = hash_map::VacantEntry<'a, Value, Value>;
#[cfg(feature = "preserve_order")]
type EDNVacantEntryImpl<'a> = indexmap::map::VacantEntry<'a, Value, Value>;

#[cfg(not(feature = "preserve_order"))]
type EDNOccupiedEntryImpl<'a> = hash_map::OccupiedEntry<'a, Value, Value>;
#[cfg(feature = "preserve_order")]
type EDNOccupiedEntryImpl<'a> = indexmap::map::OccupiedEntry<'a, Value, Value>;

impl<'a> EDNEntry<'a> {
    pub fn key(&self) -> &Value {
        match *self {
            EDNEntry::Vacant(ref e) => e.key(),
            EDNEntry::Occupied(ref e) => e.key(),
        }
    }

    pub fn or_insert(self, default: Value) -> &'a mut Value {
        match self {
            EDNEntry::Vacant(entry) => entry.insert(default),
            EDNEntry::Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn or_insert_with<F>(self, default: F) -> &'a mut Value
        where
            F: FnOnce() -> Value,
    {
        match self {
            EDNEntry::Vacant(entry) => entry.insert(default()),
            EDNEntry::Occupied(entry) => entry.into_mut(),
        }
    }
}

impl<'a> EDNVacantEntry<'a> {
    #[inline]
    pub fn key(&self) -> &Value {
        self.vacant.key()
    }

    #[inline]
    pub fn insert(self, value: Value) -> &'a mut Value {
        self.vacant.insert(value)
    }
}

impl<'a> EDNOccupiedEntry<'a> {
    #[inline]
    pub fn key(&self) -> &Value {
        self.occupied.key()
    }

    #[inline]
    pub fn get(&self) -> &Value {
        self.occupied.get()
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut Value {
        self.occupied.get_mut()
    }

    #[inline]
    pub fn into_mut(self) -> &'a mut Value {
        self.occupied.into_mut()
    }

    #[inline]
    pub fn insert(&mut self, value: Value) -> Value {
        self.occupied.insert(value)
    }

    #[inline]
    pub fn remove(self) -> Value {
        self.occupied.remove()
    }
}



/////////////////////////////////////////////////////////////////////////////
/// Represents a edn key/value type.
/// called by lib itself for hacks so leaving for now
pub struct MapInternal<K, V> {
    map: MapInternalImpl<K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type MapInternalImpl<K, V> = BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
type MapInternalImpl<K, V> = IndexMap<K, V>;

impl MapInternal<String, Value> {
    /// Makes a new empty Map.
    #[inline]
    pub fn new() -> Self {
        MapInternal {
            map: MapInternalImpl::new(),
        }
    }

    #[cfg(not(feature = "preserve_order"))]
    /// Makes a new empty Map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        // does not support with_capacity
        let _ = capacity;
        MapInternal {
            map: BTreeMap::new(),
        }
    }

    #[cfg(feature = "preserve_order")]
    /// Makes a new empty Map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        MapInternal {
            map: IndexMap::with_capacity(capacity),
        }
    }

    /// Clears the map, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear()
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&Value>
    where
        String: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.get(key)
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.contains_key(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Value>
    where
        String: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.get_mut(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    #[inline]
    pub fn insert(&mut self, k: String, v: Value) -> Option<Value> {
        self.map.insert(k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Value>
    where
        String: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.remove(key)
    }

    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    pub fn entry<S>(&mut self, key: S) -> Entry
    where
        S: Into<String>,
    {
        #[cfg(feature = "preserve_order")]
        use indexmap::map::Entry as EntryImpl;
        #[cfg(not(feature = "preserve_order"))]
        use std::collections::btree_map::Entry as EntryImpl;

        match self.map.entry(key.into()) {
            EntryImpl::Vacant(vacant) => Entry::Vacant(VacantEntry { vacant: vacant }),
            EntryImpl::Occupied(occupied) => Entry::Occupied(OccupiedEntry { occupied: occupied }),
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Gets an iterator over the entries of the map.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            iter: self.map.iter(),
        }
    }

    /// Gets a mutable iterator over the entries of the map.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }

    /// Gets an iterator over the keys of the map.
    #[inline]
    pub fn keys(&self) -> Keys {
        Keys {
            iter: self.map.keys(),
        }
    }

    /// Gets an iterator over the values of the map.
    #[inline]
    pub fn values(&self) -> Values {
        Values {
            iter: self.map.values(),
        }
    }

    /// Gets an iterator over mutable values of the map.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut {
        ValuesMut {
            iter: self.map.values_mut(),
        }
    }
}

impl Default for MapInternal<String, Value> {
    #[inline]
    fn default() -> Self {
        MapInternal {
            map: MapInternalImpl::new(),
        }
    }
}

impl Clone for MapInternal<String, Value> {
    #[inline]
    fn clone(&self) -> Self {
        MapInternal {
            map: self.map.clone(),
        }
    }
}

impl PartialEq for MapInternal<String, Value> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if cfg!(feature = "preserve_order") {
            if self.len() != other.len() {
                return false;
            }

            self.iter()
                .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
        } else {
            self.map.eq(&other.map)
        }
    }
}
impl IntoIterator for Map<Value, Value> {
    type Item = (Value, Value);
    type IntoIter = MapIntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        MapIntoIter {
            iter: self.map.into_iter(),
        }
    }
}




/// Access an element of this map. Panics if the given key is not present in the
/// map.
///
/// ```rust
/// # use serde_edn::Value;
/// #
/// # let val = &Value::String("".to_owned());
/// # let _ =
/// match *val {
///     Value::String(ref s) => Some(s.as_str()),
///     Value::Vector(ref xs) => xs[0].as_str(),
///     Value::Object(ref map) => map["type"].as_str(),
///     _ => None,
/// }
/// # ;
/// ```
impl<'a, Q: ?Sized> ops::Index<&'a Q> for MapInternal<String, Value>
where
    String: Borrow<Q>,
    Q: Ord + Eq + Hash,
{
    type Output = Value;

    fn index(&self, index: &Q) -> &Value {
        self.map.index(index)
    }
}

/// Mutably access an element of this map. Panics if the given key is not
/// present in the map.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_edn;
/// #
/// # fn main() {
/// #     let mut map = serde_edn::MapInternal::new();
/// #     map.insert("key".to_owned(), serde_edn::Value::Nil);
/// #
/// map["key"] = edn!("value");
/// # }
/// ```
impl<'a, Q: ?Sized> ops::IndexMut<&'a Q> for MapInternal<String, Value>
where
    String: Borrow<Q>,
    Q: Ord + Eq + Hash,
{
    fn index_mut(&mut self, index: &Q) -> &mut Value {
        self.map.get_mut(index).expect("no entry found for key")
    }
}

impl Debug for MapInternal<String, Value> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.map.fmt(formatter)
    }
}

impl ser::Serialize for MapInternal<String, Value> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = try!(serializer.serialize_map(Some(self.len())));
        for (k, v) in self {
            try!(map.serialize_key(k));
            try!(map.serialize_value(v));
        }
        map.end()
    }
}

impl<'de> de::Deserialize<'de> for MapInternal<String, Value> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = MapInternal<String, Value>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MapInternal::new())
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut values = MapInternal::new();

                while let Some((key, value)) = try!(visitor.next_entry()) {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl FromIterator<(String, Value)> for MapInternal<String, Value> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, Value)>,
    {
        MapInternal {
            map: FromIterator::from_iter(iter),
        }
    }
}

impl Extend<(String, Value)> for MapInternal<String, Value> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (String, Value)>,
    {
        self.map.extend(iter);
    }
}



//////////////////////////////////////////////////////////////////////////////

/// A view into a single entry in a map, which may either be vacant or occupied.
/// This enum is constructed from the [`entry`] method on [`Map`].
///
/// [`entry`]: struct.Map.html#method.entry
/// [`Map`]: struct.Map.html
pub enum Entry<'a> {
    /// A vacant Entry.
    Vacant(VacantEntry<'a>),
    /// An occupied Entry.
    Occupied(OccupiedEntry<'a>),
}

/// A vacant Entry. It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a> {
    vacant: VacantEntryImpl<'a>,
}

/// An occupied Entry. It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a> {
    occupied: OccupiedEntryImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type VacantEntryImpl<'a> = btree_map::VacantEntry<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type VacantEntryImpl<'a> = indexmap::map::VacantEntry<'a, String, Value>;

#[cfg(not(feature = "preserve_order"))]
type OccupiedEntryImpl<'a> = btree_map::OccupiedEntry<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type OccupiedEntryImpl<'a> = indexmap::map::OccupiedEntry<'a, String, Value>;

impl<'a> Entry<'a> {
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut map = serde_edn::MapInternal::new();
    /// assert_eq!(map.entry("serde").key(), &"serde");
    /// ```
    pub fn key(&self) -> &String {
        match *self {
            Entry::Vacant(ref e) => e.key(),
            Entry::Occupied(ref e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut map = serde_edn::MapInternal::new();
    /// map.entry("serde").or_insert(edn!(12));
    ///
    /// assert_eq!(map["serde"], 12);
    /// # }
    /// ```
    pub fn or_insert(self, default: Value) -> &'a mut Value {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// let mut map = serde_edn::MapInternal::new();
    /// map.entry("serde").or_insert_with(|| edn!("hoho"));
    ///
    /// assert_eq!(map["serde"], "hoho".to_owned());
    /// # }
    /// ```
    pub fn or_insert_with<F>(self, default: F) -> &'a mut Value
    where
        F: FnOnce() -> Value,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }
}

impl<'a> VacantEntry<'a> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    ///
    /// match map.entry("serde") {
    ///     Entry::Vacant(vacant) => {
    ///         assert_eq!(vacant.key(), &"serde");
    ///     }
    ///     Entry::Occupied(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn key(&self) -> &String {
        self.vacant.key()
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a
    /// mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    ///
    /// match map.entry("serde") {
    ///     Entry::Vacant(vacant) => {
    ///         vacant.insert(edn!("hoho"));
    ///     }
    ///     Entry::Occupied(_) => unimplemented!(),
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn insert(self, value: Value) -> &'a mut Value {
        self.vacant.insert(value)
    }
}

impl<'a> OccupiedEntry<'a> {
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    /// map.insert("serde".to_owned(), edn!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         assert_eq!(occupied.key(), &"serde");
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn key(&self) -> &String {
        self.occupied.key()
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    /// map.insert("serde".to_owned(), edn!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         assert_eq!(occupied.get(), 12);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn get(&self) -> &Value {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    /// map.insert("serde".to_owned(), edn!([1, 2, 3]));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(mut occupied) => {
    ///         occupied.get_mut().as_vector_mut().unwrap().push(edn!(4));
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    ///
    /// assert_eq!(map["serde"].as_vector().unwrap().len(), 4);
    /// # }
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut Value {
        self.occupied.get_mut()
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    /// map.insert("serde".to_owned(), edn!([1, 2, 3]));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(mut occupied) => {
    ///         occupied.into_mut().as_vector_mut().unwrap().push(edn!(4));
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    ///
    /// assert_eq!(map["serde"].as_vector().unwrap().len(), 4);
    /// # }
    /// ```
    #[inline]
    pub fn into_mut(self) -> &'a mut Value {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key, and returns
    /// the entry's old value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    /// map.insert("serde".to_owned(), edn!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(mut occupied) => {
    ///         assert_eq!(occupied.insert(edn!(13)), 12);
    ///         assert_eq!(occupied.get(), 13);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn insert(&mut self, value: Value) -> Value {
        self.occupied.insert(value)
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_edn;
    /// #
    /// # fn main() {
    /// use serde_edn::map::Entry;
    ///
    /// let mut map = serde_edn::MapInternal::new();
    /// map.insert("serde".to_owned(), edn!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         assert_eq!(occupied.remove(), 12);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn remove(self) -> Value {
        self.occupied.remove()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> IntoIterator for &'a MapInternal<String, Value> {
    type Item = (&'a String, &'a Value);
    type IntoIter = Iter<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.map.iter(),
        }
    }
}

/// An iterator over a serde_edn::Map's entries.
pub struct Iter<'a> {
    iter: IterImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type IterImpl<'a> = btree_map::Iter<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type IterImpl<'a> = indexmap::map::Iter<'a, String, Value>;

delegate_iterator!((Iter<'a>) => (&'a String, &'a Value));

//////////////////////////////////////////////////////////////////////////////

impl<'a> IntoIterator for &'a mut MapInternal<String, Value> {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = IterMut<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

/// A mutable iterator over a serde_edn::Map's entries.
pub struct IterMut<'a> {
    iter: IterMutImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type IterMutImpl<'a> = btree_map::IterMut<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type IterMutImpl<'a> = indexmap::map::IterMut<'a, String, Value>;

delegate_iterator!((IterMut<'a>) => (&'a String, &'a mut Value));

//////////////////////////////////////////////////////////////////////////////

impl IntoIterator for MapInternal<String, Value> {
    type Item = (String, Value);
    type IntoIter = IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

/// An owning iterator over a serde_edn::Map's entries.
pub struct IntoIter {
    iter: IntoIterImpl,
}

#[cfg(not(feature = "preserve_order"))]
type IntoIterImpl = btree_map::IntoIter<String, Value>;
#[cfg(feature = "preserve_order")]
type IntoIterImpl = indexmap::map::IntoIter<String, Value>;

delegate_iterator!((IntoIter) => (String, Value));

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a serde_edn::Map's keys.
pub struct Keys<'a> {
    iter: KeysImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type KeysImpl<'a> = btree_map::Keys<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type KeysImpl<'a> = indexmap::map::Keys<'a, String, Value>;

delegate_iterator!((Keys<'a>) => &'a String);

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a serde_edn::Map's values.
pub struct Values<'a> {
    iter: ValuesImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesImpl<'a> = btree_map::Values<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type ValuesImpl<'a> = indexmap::map::Values<'a, String, Value>;

delegate_iterator!((Values<'a>) => &'a Value);

//////////////////////////////////////////////////////////////////////////////

/// A mutable iterator over a serde_edn::Map's values.
pub struct ValuesMut<'a> {
    iter: ValuesMutImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesMutImpl<'a> = btree_map::ValuesMut<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type ValuesMutImpl<'a> = indexmap::map::ValuesMut<'a, String, Value>;

delegate_iterator!((ValuesMut<'a>) => &'a mut Value);
