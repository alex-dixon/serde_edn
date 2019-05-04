//use Value;
//use error::Error;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Unimplemented<'a> {
    /// The input contained a boolean value that was not expected.
    Keyword(&'a str),
    Symbol(&'a str),

}

//pub trait EDNVisitor<'de>: Sized {
//    type OutputValue;
//
//    fn visit_keyword_str<E>(self, v: &str) -> Result<Self::OutputValue, E>
//        where E: de::Error,
//    {
//        Err(Error::custom(Unimplemented::Keyword(&v), &self))
//    }
//
//    fn visit_keyword_string<E>(self, v: String) -> Result<Self::OutputValue, E>
//        where E: de::Error,
//    {
//        Err(Error::custom(Unimplemented::Keyword(&v), &self))
//    }
//
//    fn visit_symbol_str<E>(self, v: &str) -> Result<Self::OutputValue, E>
//        where E: de::Error,
//    {
//        Err(Error::custom(Unimplemented::Symbol(&v), &self))
//    }
//
//    fn visit_symbol_string<E>(self, v: String) -> Result<Self::OutputValue, E>
//        where E: de::Error,
//    {
//        Err(Error::custom(Unimplemented::Symbol(&v), &self))
//    }
////}
//
//#[inline]
//pub fn visit_keyword_str<'de, V>(v: &str, visitor:V) -> Result<V::Value, Error> {
//    visit_keyword_string(visitor,String::from(v))
//}
//
//#[inline]
//pub fn visit_keyword_string<'de, V>(v: String, visitor: V) -> Result<V::Value, Error>{
////pub fn visit_keyword_string<E>(v: String) -> Result<Value, E> {
//    Ok(Value::Keyword(v))
//}
//
//#[inline]
//pub fn visit_symbol_str<E>(v: &str) -> Result<Value, E> {
//    visit_symbol_string(String::from(v))
//}
//
//#[inline]
//pub fn visit_symbol_string<E>(v: String) -> Result<Value, E> {
//    Ok(Value::Symbol(v))
//}
