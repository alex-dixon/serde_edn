use serde::de::{SeqAccess, Unexpected, Visitor};
use ::{Error, Value};
use std::str::FromStr;

pub trait EDNVisitor<'de>: Sized + Visitor<'de> {
    type EDNValue;

    fn visit_list<A>(self, seq: A) -> Result<<Self as Visitor<'de>>::Value, A::Error>
        where
            A: SeqAccess<'de>,
    {
        let _ = seq;
        unimplemented!()
//        Err(Error::invalid_type(Unexpected::Seq, &self))
    }
}

pub trait EDNDeserializer<'de>: Sized {
    type Error;

    fn deserialize_any<V>(self,visitor:V)->Result<V::Value,Self::Error>
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
            D: EDNDeserializer<'de> + serde::Deserializer<'de>;
}

pub trait EDNDeserializeOwned: for<'de> EDNDeserialize<'de> {}

impl<T> EDNDeserializeOwned for T where T: for<'de> EDNDeserialize<'de> {}


#[test]
fn main() {
//    let x = Value::from_str(r#"(foo "bar")"#);
    let x = Value::from_str(r#"(foo(bar"baz"))"#);
    let k = Value::from_str(":foo");
    println!("x {:?}",x.unwrap());
    println!("k {:?}",k.unwrap());
    assert_eq!(false,true)
}
