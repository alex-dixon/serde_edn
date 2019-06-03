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

    type SerializeL: SerializeList<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;
    type SerializeV: SerializeVector<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;
    type SerializeS: SerializeSet<Ok = Self::Ok, Error = <Self as serde::Serializer>::Error>;

    fn serialize_list(self, len: Option<usize>) -> Result<Self::SerializeL, <Self as serde::Serializer>::Error>;
    fn serialize_set(self, len: Option<usize>) -> Result<Self::SerializeS, <Self as serde::Serializer>::Error>;
    fn serialize_vector(self, len: Option<usize>) -> Result<Self::SerializeV, <Self as serde::Serializer>::Error>;
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
