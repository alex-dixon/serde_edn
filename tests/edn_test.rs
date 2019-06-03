#![cfg(not(feature = "preserve_order"))]
#![cfg_attr(feature = "cargo-clippy", allow(float_cmp, unreadable_literal))]
#![cfg_attr(feature = "trace-macros", feature(trace_macros))]
#[cfg(feature = "trace-macros")]
trace_macros!(true);

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_edn;
extern crate compiletest_rs;

#[macro_use]
mod macros;

use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug};
use std::io;
use std::iter;
use std::marker::PhantomData;
use std::str::FromStr;
use std::string::ToString;
use std::{f32, f64};
use std::{i16, i32, i64, i8};
use std::{u16, u32, u64, u8};

use serde::de::{self, Deserialize, IgnoredAny};
use serde::ser::{self, Serialize, Serializer};

use serde_bytes::{ByteBuf, Bytes};

use serde_edn::{
    from_reader, from_slice, from_str, from_value, to_string, to_string_pretty, to_value, to_vec,
    to_writer, Deserializer, Number, Value, Keyword,
};
use serde_edn::value::Symbol;
use serde_edn::edn_ser::EDNSerialize;
use compiletest_rs::common::Mode::CompileFail;

#[derive(Clone)]
struct SimpleTypes {
    int: Value,
    float: Value,
    string: Value,
    keyword: Value,
    symbol: Value,
    boolean: Value,
}

impl SimpleTypes {
    fn values(self) -> Vec<Value> {
        vec!(
            self.symbol,
            self.keyword,
            self.string,
            self.int,
            self.float,
            self.boolean
        )
    }
}

impl Default for SimpleTypes {
    fn default() -> Self {
        SimpleTypes {
            int: Value::Number(Number::from_str("42").unwrap()),
            float: Value::Number(Number::from_str("42.3").unwrap()),
            string: Value::String(String::from_str("foo").unwrap()),
            keyword: Value::Keyword(Keyword::from_str("foo").unwrap()),
            symbol: Value::Symbol(Symbol::from_str("println").unwrap()),
            boolean: Value::Bool(true),
        }
    }
}

struct ComplexTypes {
    vector: Value,
    list: Value,
}

impl Default for ComplexTypes {
    fn default() -> Self {
        let st = SimpleTypes::default();
        ComplexTypes {
            vector: Value::Vector(st.clone().values()),
            list: Value::List(st.values()),
        }
    }
}

fn round_trip(s: &str, v: Value) {
    let a = Value::from_str(s).unwrap();
    assert_eq!(a, v);

    let b = to_string(&v).unwrap();
    assert_eq!(b, s)
}

#[test]
fn parse_list() {
    let st = SimpleTypes::default();

    let empty = Value::from_str(r#"()"#).unwrap();
    assert_eq!(empty, Value::List(vec!()));

    let flat = Value::from_str(r#"(println :foo "foo" 42 42.3 true)"#).unwrap();
    assert_eq!(flat, Value::List(SimpleTypes::default().values()));

    let lol5 = Value::from_str(r#"((((()))))"#).unwrap();
    assert_eq!(lol5, Value::List(vec!(Value::List(vec!(Value::List(vec!(Value::List(vec!(Value::List(vec!()))))))))));

    let inside_vector = Value::from_str(r#"(println [:foo (println)])"#).unwrap();
    assert_eq!(
        inside_vector,
        Value::List(vec![st.symbol.clone(),
                         Value::Vector(vec![st.keyword,
                                            Value::List(vec![st.symbol])])])
    )
}

#[test]
fn parse_set() {
    let st = SimpleTypes::default();

    let empty = Value::from_str(r#"#{}"#).unwrap();
    assert_eq!(empty, Value::Set(vec!()));

    let flat = Value::from_str(r#"#{println :foo "foo" 42 42.3 true}"#).unwrap();
    assert_eq!(flat, Value::Set(SimpleTypes::default().values()));

//    let lol5 = Value::from_str(r#"((((()))))"#).unwrap();
//    assert_eq!(lol5, Value::List(vec!(Value::List(vec!(Value::List(vec!(Value::List(vec!(Value::List(vec!()))))))))));
//
    let inside_vector = Value::from_str(r#"#{println [:foo #{println}]}"#).unwrap();
    assert_eq!(
        inside_vector,
        Value::Set(vec![st.symbol.clone(),
                         Value::Vector(vec![st.keyword,
                                            Value::Set(vec![st.symbol])])])
    )
}

const STR: SimpleStrings<'static> = SimpleStrings {
    int: "42",
    float: "42.3",
    string: "\"foo\"",
    keyword: ":foo",
    symbol: "foo",
};

//#[derive(Clone)]
struct SimpleStrings<'a> {
    int: &'a str,
    float: &'a str,
    string: &'a str,
    keyword: &'a str,
    symbol: &'a str,
}

impl SimpleStrings<'static> {
    fn values(self) -> Vec<&'static str> {
        vec!(
            self.symbol,
            self.keyword,
            self.string,
            self.int,
            self.float,
        )
    }
}


#[test]
fn serialize_list() {
    let st = SimpleTypes::default();
    assert_eq!(
        to_string(&Value::List(vec![])).unwrap(),
        "()"
    );

    let vs = st.clone().values();
    assert_eq!(
        to_string(&Value::List(vs)).unwrap(),
        r#"(println :foo "foo" 42 42.3 true)"#
    );

    // convenient but impl makes it harder to tell what went wrong
    // leaving until it becomes a problem
//    round_trip(r#"(println :foo "foo" 42 42.3)"#, Value::List(st.values()));
    let st2 = SimpleTypes::default();
    assert_eq!(
        to_string(&Value::List(vec![st2.symbol.clone(),
                                    Value::Vector(vec![st2.keyword,
                                                       Value::List(vec![st2.symbol])])])
        ).unwrap(),
        r#"(println [:foo (println)])"#
    );
}

#[test]
fn serialize_set() {
    let st = SimpleTypes::default();
    assert_eq!(
        to_string(&Value::Set(vec![])).unwrap(),
        "#{}"
    );

    let vs = st.clone().values();
    assert_eq!(
        to_string(&Value::Set(vs)).unwrap(),
        r#"#{println :foo "foo" 42 42.3 true}"#
    );

    // convenient but impl makes it harder to tell what went wrong
    // leaving until it becomes a problem
//    round_trip(r#"(println :foo "foo" 42 42.3)"#, Value::List(st.values()));
    let st2 = SimpleTypes::default();
    assert_eq!(
        to_string(&Value::Set(vec![st2.symbol.clone(),
                                    Value::Vector(vec![st2.keyword,
                                                       Value::Set(vec![st2.symbol])])])
        ).unwrap(),
        r#"#{println [:foo #{println}]}"#
    );
}

#[test]
fn parse_arbitrary() {
    let x = Value::from_str(r#"(println(println[[:foo [(true 1 42.0)]]"hi"]))"#).unwrap();
    let k = Value::from_str(":foo");
    println!("x {:?}", &x);
    println!("x {}", &x);
    println!("one more again");
    println!("{}", format!("{}", &x));
    println!("k {:?}", k.unwrap());
}
