use serde::de::{SeqAccess, Unexpected, Visitor};
use ::{Error, Value};
use std::str::FromStr;
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
    fn  visit_keyword<E>(self,s:&str) -> Result<<Self as Visitor<'de>>::Value, E>
    where E:serde::de::Error{
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
            D: EDNDeserializer<'de>;// + serde::Deserializer<'de>;
}

pub trait EDNDeserializeOwned: for<'de> EDNDeserialize<'de> {}

impl<T> EDNDeserializeOwned for T where T: for<'de> EDNDeserialize<'de> {}

pub trait EDNDeserializeSeed<'de>: Sized //+ serde::de::DeserializeSeed<'de>
{
    /// The type produced by using this seed.
    type Value;

    /// Equivalent to the more common `Deserialize::deserialize` method, except
    /// with some initial piece of data (the seed) passed in.
    fn deserialize<D>(self, deserializer: D) -> Result<<Self as EDNDeserializeSeed<'de>>::Value, <D as EDNDeserializer<'de>>::Error>
        where
            D: EDNDeserializer<'de> + serde::Deserializer<'de>;
}

impl<'de, T> EDNDeserializeSeed<'de> for PhantomData<T>
    where
        T: EDNDeserialize<'de>, //+ serde::Deserialize<'de>,
{
    type Value = T;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<T, <D as EDNDeserializer<'de>>::Error>
        where D: EDNDeserializer<'de>,
    //+ serde::Deserializer<'de>,
    {
        EDNDeserialize::deserialize(deserializer)
    }
}

pub trait EDNSeqAccess<'de> {
    //    type Error: Error;
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


#[test]
fn main() {
//    let x = Value::from_str(r#"(foo "bar")"#);
//    let x = Value::from_str(r#"(false (bar"baz"))"#);
//    let x = Value::from_str(r#"(println(println[[(true)]"hi"]))"#).unwrap();

    let x = Value::from_str(r#"(println(println[[:foo [(true)]]"hi"]))"#).unwrap();
//    let x = Value::from_str(r#"(println(println[(true)"hi"]))"#).unwrap();

    let k = Value::from_str(":foo");
    println!("x {:?}", &x);
    println!("x {}", &x);
    println!("one more again");
    println!("{}", format!("{}", &x));
    println!("k {:?}", k.unwrap());
    assert_eq!(false, true)
}
