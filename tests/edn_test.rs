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

use serde_edn::{from_reader, from_slice, from_str, from_value, to_string, to_string_pretty, to_value, to_vec, to_writer, Deserializer, Number, Value, Keyword};
use serde_edn::value::Symbol;
use serde_edn::edn_ser::EDNSerialize;
use compiletest_rs::common::Mode::CompileFail;
use std::fs::File;
use std::io::{Write, BufReader};
use serde_edn::map::Map;

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

fn symbol(s: &str) -> Value {
    Value::Symbol(Symbol { value: String::from_str(s).unwrap() })
}

fn keyword(s: &str) -> Value {
    Value::Keyword(Keyword { value: String::from_str(s).unwrap() })
}

fn number(s: &str) -> Value {
    Value::Number(Number::from_str(s).unwrap())
}

fn string(s: &str) -> Value {
    Value::String(String::from_str(s).unwrap())
}

fn round_trip(s: &str, v: Value) {
    let a = Value::from_str(s).unwrap();
    assert_eq!(a, v);

    let b = to_string(&v).unwrap();
    assert_eq!(b, s)
}
fn round_trip2(s: &str, ) {
    let a = Value::from_str(s).unwrap();
//    assert_eq!(a, v);

    let b = to_string(&a).unwrap();
    assert_eq!(b, s)
}

fn read(s: &str) -> Value {
    from_reader(s.as_bytes().as_ref()).unwrap()
}

#[test]
fn parse_char() {
    assert_eq!(Value::Char('\n'), Value::from_str("\\newline").unwrap());
    assert_eq!(Value::Char('\r'), Value::from_str("\\return").unwrap());
    assert_eq!(Value::Char('\t'), Value::from_str("\\tab").unwrap());
    assert_eq!(Value::Char(' '), Value::from_str("\\space").unwrap());
    assert_eq!(Value::Char('n'), Value::from_str("\\n").unwrap());
    assert_eq!(Value::from_str(r#"[\n \e \w \l \i \n \e]"#).unwrap(),
               Value::Vector(vec![Value::Char('n'), Value::Char('e'), Value::Char('w'), Value::Char('l'), Value::Char('i'), Value::Char('n'), Value::Char('e')]),
    );
    assert_eq!(Value::Char('z'), Value::from_str("\\z").unwrap());
}
#[test]
fn serialize_char() {
//    round_trip2("\\newline");
    assert_eq!(to_string(&Value::Char('\n')).unwrap(), "\\newline");
    assert_eq!(to_string(&Value::Char(' ')).unwrap(), "\\space");
    assert_eq!(to_string(&Value::Char('\r')).unwrap(), "\\return");
    assert_eq!(to_string(&Value::Char('\t')).unwrap(), "\\tab");
    assert_eq!(to_string(&Value::Char('n')).unwrap(), "\\n");
}
macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = Map::new();
            $(
                m.insert($key, $value);
            )+
            Value::Object(m)
        }
     };
);

#[test]
fn parse_map() {
    let empty = Value::from_str(r#"{}"#).unwrap();
    assert_eq!(empty, Value::Object(Map::new()));

//    let max_float:String = Number::from_f64(f64::MAX).unwrap().to_string();
//    assert_eq!(
//        map!(number(&max_float)=>number("1")),
//        map!(number(&max_float)=>number("1")),
//    );

    // float key
    let b =  Value::from_str(r#"{42.39  foo 42.39 bar}"#).unwrap();
    let mut bm = Map::new();
    bm.insert(number("42.39"),symbol("foo"));
    bm.insert(number("42.39"),symbol("bar"));
    assert_eq!(b, Value::Object(bm));

    let mut cm = Map::new();
    cm.insert(number("42.39"),symbol("foo"));
    cm.insert(number("42.39"),symbol("bar"));
    assert_eq!(b, Value::Object(cm));


    // vector key
    let a =  Value::from_str(r#"{[42 43 44] bar}"#).unwrap();
    let mut am = Map::new();
    am.insert(
        Value::Vector(vec![number("42"),number("43"),number("44")]),
        symbol("bar")
    );
    assert_eq!(a, am.clone()); //  not sure the utility of this yet
    assert_eq!(a, Value::Object(am));


    // map key
    assert_eq!(
        map!(map!(keyword("a")=>number("1"))=> symbol("foo")),
        map!(map!(keyword("a")=>number("1"))=> symbol("foo"))
    );
    assert_eq!(
        Value::from_str(r#"{{:a 1} foo}"#).unwrap(),
        map!(map!(keyword("a")=>number("1"))=> symbol("foo"))
    );
    assert_ne!(map!(map!(keyword("a")=>number("1"))=> symbol("foo")),
               map!(map!(keyword("a")=>number("1"))=> symbol("bar"))
    );
    assert_ne!(map!(map!(keyword("b")=>number("1"))=> symbol("bar")),
               map!(map!(keyword("a")=>number("1"))=> symbol("bar"))
    );


    // set key
    assert_eq!(
        Value::from_str(r#"{#{1 2 3} 1}"#).unwrap(),
        map!(Value::Set(vec![number("1"),number("2"),number("3")])=>number("1"))
    );


    // list key
    assert_eq!(
        Value::from_str(r#"{(1 2 3) 1}"#).unwrap(),
        map!(Value::List(vec![number("1"),number("2"),number("3")])=>number("1"))
    );

    assert_eq!(
        map!(map!(keyword("a")=>number("1"))=> symbol("foo")),
        read(r#"{{:a 1} foo}"#)
    );
}

#[test]
fn serialize_map() {
    let st = SimpleTypes::default();
    assert_eq!(
        to_string(&Value::Object(Map::new())).unwrap(),
        "{}"
    );

    let vs = st.clone().values();
    assert_eq!(
        to_string(&map!(keyword("a")=>number("42"))).unwrap(),
            r#"{:a 42}"#
    );

    let st2 = SimpleTypes::default();
    assert_eq!(
        to_string(&map!(string("a")=>string("b"))
        ).unwrap(),
        r#"{"a" "b"}"#
    );

//    let in_str = r#"{"statuses" [{"id_str" "505874924095815681", "retweeted" false, "lang" "ja", "truncated" false, "place" nil, "in_reply_to_status_id" nil, "user" {"listed_count" 0, "id_str" "1186275104", "profile_link_color" "0084B4", "profile_sidebar_border_color" "C0DEED", "lang" "en", "follow_request_sent" false, "profile_text_color" "333333", "url" nil, "profile_background_tile" false, "contributors_enabled" false, "favourites_count" 235, "notifications" false, "friends_count" 252, "profile_image_url_https" "https://pbs.twimg.com/profile_images/497760886795153410/LDjAwR_y_normal.jpeg", "profile_background_color" "C0DEED", "id" 1186275104, "is_translator" false, "profile_background_image_url_https" "https://abs.twimg.com/images/themes/theme1/bg.png", "protected" false, "utc_offset" nil, "name" "AYUMI", "verified" false, "time_zone" nil, "location" "", "is_translation_enabled" false, "profile_image_url" "http://pbs.twimg.com/profile_images/497760886795153410/LDjAwR_y_normal.jpeg", "default_profile_image" false, "profile_background_image_url" "http://abs.twimg.com/images/themes/theme1/bg.png", "profile_banner_url" "https://pbs.twimg.com/profile_banners/1186275104/1409318784", "statuses_count" 1769, "created_at" "Sat Feb 16 13:40:25 +0000 2013", "geo_enabled" false, "followers_count" 262, "profile_sidebar_fill_color" "DDEEF6", "profile_use_background_image" true, "default_profile" true, "screen_name" "ayuu0123", "following" false, "description" "元野球部マネージャー❤︎…最高の夏をありがとう…❤︎", "entities" {"description" {"urls" []}}}, "favorited" false, "id" 505874924095815700, "in_reply_to_screen_name" "aym0566x", "coordinates" nil, "favorite_count" 0, "geo" nil, "text" "@aym0566x \n\n名前:前田あゆみ\n第一印象:なんか怖っ！\n今の印象:とりあえずキモい。噛み合わない\n好きなところ:ぶすでキモいとこ😋✨✨\n思い出:んーーー、ありすぎ😊❤️\nLINE交換できる？:あぁ……ごめん✋\nトプ画をみて:照れますがな😘✨\n一言:お前は一生もんのダチ💖", "in_reply_to_user_id" 866260188, "metadata" {"result_type" "recent", "iso_language_code" "ja"}, "in_reply_to_user_id_str" "866260188", "source" "<a href=\"http://twitter.com/download/iphone\" rel=\"nofollow\">Twitter for iPhone</a>", "created_at" "Sun Aug 31 00:29:15 +0000 2014", "in_reply_to_status_id_str" nil, "contributors" nil, "retweet_count" 0, "entities" {"hashtags" [], "symbols" [], "urls" [], "user_mentions" [{"screen_name" "aym0566x", "name" "前田あゆみ", "id" 866260188, "id_str" "866260188", "indices" [0 9]}]}}]}"#;
//    let v = Value::from_str(in_str).unwrap();
//    println!("{}", v.to_string());
//    assert_eq!(true,false)
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
fn parse_vector() {
    let unclosed1 = Value::from_str("[").unwrap_err();
    assert!(unclosed1.is_eof())
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
fn deserialize_reserved_vs_symbol() {
    assert_eq!(symbol("t"), Value::from_str("t").unwrap());
    assert_eq!(symbol("tr"), Value::from_str("tr").unwrap());
    assert_eq!(symbol("tru"), Value::from_str("tru").unwrap());
    assert_eq!(Value::Bool(true), Value::from_str("true").unwrap());
    assert_eq!(symbol("trub"), Value::from_str("trub").unwrap());
    assert_eq!(symbol("trued"), Value::from_str("trued").unwrap());

    assert_eq!(symbol("t"), read("t"));
    assert_eq!(symbol("tr"), read("tr"));
    assert_eq!(symbol("tru"), read("tru"));
    assert_eq!(Value::Bool(true), read("true"));
    assert_eq!(symbol("trub"), read("trub"));
    assert_eq!(symbol("trued"), read("trued"));
}

#[test]
fn deserialize_file() {
    let x = Value::from_str(r#"(println(println[[:foo [(true 1 42.0)]]"hi"]))"#).unwrap();
    let y = Value::List(vec![symbol("println"),
                             Value::List(vec![symbol("println"),
                                              Value::Vector(vec![Value::Vector(vec![keyword("foo"),
                                                                                    Value::Vector(vec![Value::List(vec![
                                                                                        Value::Bool(true),
                                                                                        number("1"),
                                                                                        number("42.0")
                                                                                    ])])
                                              ]),
                                                                 string("hi")]),
                             ])
    ]);
    assert_eq!(x, y);
    assert_eq!(Value::Bool(true), Value::from_str("true").unwrap());
    let mut w = File::create("foo.edn").unwrap();
    let s = r#"(println(println[[:foo [(true 1 42.0)]]"hi"]))"#;
    w.write_all(s.as_bytes());
    let r = BufReader::new(File::open("foo.edn").unwrap());
    let v: Value = from_reader(r).unwrap();
    assert_eq!(v, y);
    std::fs::remove_file("foo.edn").unwrap();
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
