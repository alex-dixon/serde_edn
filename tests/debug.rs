#[macro_use]
extern crate serde_edn;

use serde_edn::{Number, Value};

#[test]
fn number() {
    assert_eq!(format!("{:?}", Number::from(1)), "Number(1)");
    assert_eq!(format!("{:?}", Number::from(-1)), "Number(-1)");
    assert_eq!(
        format!("{:?}", Number::from_f64(1.0).unwrap()),
        "Number(1.0)"
    );
}

#[test]
fn value_null() {
    assert_eq!(format!("{:?}", edn!(nil)), "Nil");
}

#[test]
fn value_bool() {
    assert_eq!(format!("{:?}", edn!(true)), "Bool(true)");
    assert_eq!(format!("{:?}", edn!(false)), "Bool(false)");
}

#[test]
fn value_number() {
    assert_eq!(format!("{:?}", edn!(1)), "Number(1)");
    assert_eq!(format!("{:?}", edn!(-1)), "Number(-1)");
    assert_eq!(format!("{:?}", edn!(1.0)), "Number(1.0)");
}

#[test]
fn value_string() {
    assert_eq!(format!("{:?}", edn!("s")), "String(\"s\")");
}

#[test]
fn value_array() {
    assert_eq!(format!("{:?}", edn!([])), "Array([])");
}

#[test]
fn value_object() {
    assert_eq!(format!("{:?}", edn!({})), "Object({})");
}

#[test]
fn error() {
    let err = serde_edn::from_str::<Value>("{0}").unwrap_err();
    let expected = "Error(\"key must be a string\", line: 1, column: 2)";
    assert_eq!(format!("{:?}", err), expected);
}
