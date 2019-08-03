use Keyword;
use symbol::Symbol;

pub trait EDNSerialize : serde::Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, <S as serde::Serializer>::Error>
        where
            S: EDNSerializer + serde::Serializer;
}


pub trait EDNSerializer: Sized + serde::Serializer
{
//    type Ok;
    type Error;// = <Self as serde::Serializer>::Error;

    type SerializeList: SerializeList<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;
    type SerializeVector: SerializeVector<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;
    type SerializeSet: SerializeSet<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;
    type SerializeMap: SerializeMap<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;

    fn serialize_list(self, len: Option<usize>) -> Result<Self::SerializeList, <Self as serde::Serializer>::Error>;
    fn serialize_set(self, len: Option<usize>) -> Result<Self::SerializeSet, <Self as serde::Serializer>::Error>;
    fn serialize_vector(self, len: Option<usize>) -> Result<Self::SerializeVector, <Self as serde::Serializer>::Error>;
    fn serialize_map(self, len:Option<usize>) -> Result<<Self  as EDNSerializer>::SerializeMap, <Self as serde::Serializer>::Error>;
    fn serialize_keyword(self, value: &Keyword) -> Result<<Self as serde::Serializer>::Ok, <Self as serde::Serializer>::Error>;
    fn serialize_symbol(self, value: &Symbol) -> Result<<Self as serde::Serializer>::Ok, <Self as serde::Serializer>::Error>;
}

pub trait SerializeVector {
    type Ok;
    type Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: EDNSerialize;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait SerializeList {
    type Ok;
    type Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: EDNSerialize;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait SerializeSet {
    type Ok;
    type Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: EDNSerialize;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait SerializeMap {
    type Ok;
    type Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: EDNSerialize;

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: EDNSerialize;

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
        where
            K: EDNSerialize,
            V: EDNSerialize,
    {
        try!(self.serialize_key(key));
        self.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>;
}
