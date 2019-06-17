use serde::de::{SeqAccess, Visitor};
use serde::export::PhantomData;

pub trait EDNVisitor<'de>: Sized + Visitor<'de> {
    type EDNValue;

    fn visit_list<A>(self, seq: A) -> Result<<Self as Visitor<'de>>::Value, A::Error>
        where
            A: EDNSeqAccess<'de>,
    {
        let _ = seq;
        unimplemented!()
//        Err(Error::invalid_type(Unexpected::Seq, &self))
    }
    fn visit_vector<A>(self, seq: A) -> Result<<Self as Visitor<'de>>::Value, A::Error>
        where
            A: EDNSeqAccess<'de>,
    {
        let _ = seq;
        unimplemented!()
//        Err(Error::invalid_type(Unexpected::Seq, &self))
    }

    fn visit_set<A>(self, seq: A) -> Result<<Self as Visitor<'de>>::Value, A::Error>
        where
            A: EDNSeqAccess<'de>,
    {
        let _ = seq;
        unimplemented!()
//        Err(Error::invalid_type(Unexpected::Seq, &self))
    }

    // note: not borrowed so lifetime implicitly 'a (not 'de)
    fn visit_symbol<E>(self, s: &str) -> Result<<Self as Visitor<'de>>::Value, E>;

    fn visit_borrowed_symbol<E>(self, s: &'de str) -> Result<<Self as Visitor<'de>>::Value, E>;

    fn visit_keyword<E>(self, s: &str) -> Result<<Self as Visitor<'de>>::Value, E>
        where E: serde::de::Error;

    fn visit_borrowed_keyword<E>(self, s: &'de str) -> Result<<Self as Visitor<'de>>::Value, E>
        where E: serde::de::Error;

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: EDNMapAccess<'de>,
    {
        let _ = map;
//        Err(Error::invalid_type(Unexpected::Map, &self))
        unimplemented!()
    }
}

pub trait EDNDeserializer<'de>: Sized {
    type Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: EDNVisitor<'de>;

    fn deserialize_list<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: EDNVisitor<'de>;
}

pub trait EDNDeserialize<'de>: Sized {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as EDNDeserializer<'de>>::Error>
        where
            D: EDNDeserializer<'de>;
}

pub trait EDNDeserializeOwned: for<'de> EDNDeserialize<'de> {}

impl<T> EDNDeserializeOwned for T where T: for<'de> EDNDeserialize<'de> {}

pub trait EDNDeserializeSeed<'de>: Sized
{
    type Value;

    fn deserialize<D>(self, deserializer: D) -> Result<<Self as EDNDeserializeSeed<'de>>::Value, <D as EDNDeserializer<'de>>::Error>
        where
            D: EDNDeserializer<'de>;
}

impl<'de, T> EDNDeserializeSeed<'de> for PhantomData<T>
    where
        T: EDNDeserialize<'de>,
{
    type Value = T;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<T, <D as EDNDeserializer<'de>>::Error>
        where D: EDNDeserializer<'de>,
    {
        EDNDeserialize::deserialize(deserializer)
    }
}

pub trait EDNSeqAccess<'de> {
    type Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<<T as EDNDeserializeSeed<'de>>::Value>, Self::Error>
        where
            T: EDNDeserializeSeed<'de>;

    #[inline]
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
        where
            T: EDNDeserialize<'de>,
    {
        self.next_element_seed(PhantomData)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

impl<'de, 'a, A> EDNSeqAccess<'de> for &'a mut A
    where
        A: EDNSeqAccess<'de>,
{
    type Error = A::Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<<T as EDNDeserializeSeed<'de>>::Value>, Self::Error>
        where
            T: EDNDeserializeSeed<'de>,
    {
        (**self).next_element_seed(seed)
    }

    #[inline]
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
        where
            T: EDNDeserialize<'de>,
    {
        (**self).next_element()
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        (**self).size_hint()
    }
}

pub trait EDNMapAccess<'de> {
    type Error: serde::de::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: EDNDeserializeSeed<'de>;

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: EDNDeserializeSeed<'de>;

    #[inline]
    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where
            K: EDNDeserializeSeed<'de>,
            V: EDNDeserializeSeed<'de>,
    {
        match try!(self.next_key_seed(kseed)) {
            Some(key) => {
                let value = try!(self.next_value_seed(vseed));
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where
            K: EDNDeserialize<'de>,
    {
        self.next_key_seed(PhantomData)
    }

    #[inline]
    fn next_value<V>(&mut self) -> Result<V, Self::Error>
        where
            V: EDNDeserialize<'de>,
    {
        self.next_value_seed(PhantomData)
    }

    #[inline]
    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where
            K: EDNDeserialize<'de>,
            V: EDNDeserialize<'de>,
    {
        self.next_entry_seed(PhantomData, PhantomData)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

impl<'de, 'a, A> EDNMapAccess<'de> for &'a mut A
    where
        A: EDNMapAccess<'de>,
{
    type Error = A::Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: EDNDeserializeSeed<'de>,
    {
        (**self).next_key_seed(seed)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: EDNDeserializeSeed<'de>,
    {
        (**self).next_value_seed(seed)
    }

    #[inline]
    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where
            K: EDNDeserializeSeed<'de>,
            V: EDNDeserializeSeed<'de>,
    {
        (**self).next_entry_seed(kseed, vseed)
    }

    #[inline]
    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where
            K: EDNDeserialize<'de>,
            V: EDNDeserialize<'de>,
    {
        (**self).next_entry()
    }

    #[inline]
    fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where
            K: EDNDeserialize<'de>,
    {
        (**self).next_key()
    }

    #[inline]
    fn next_value<V>(&mut self) -> Result<V, Self::Error>
        where
            V: EDNDeserialize<'de>,
    {
        (**self).next_value()
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        (**self).size_hint()
    }
}

pub trait EDNVariantAccess<'de>: Sized {
    type Error: serde::de::Error;

    fn unit_variant(self) -> Result<(), Self::Error>;

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where
            T: EDNDeserializeSeed<'de>;

    #[inline]
    fn newtype_variant<T>(self) -> Result<T, Self::Error>
        where
            T: EDNDeserialize<'de>,
    {
        self.newtype_variant_seed(PhantomData)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: EDNVisitor<'de>;

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: EDNVisitor<'de>;
}
