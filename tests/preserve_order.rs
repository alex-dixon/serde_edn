// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate serde_edn;

use serde_edn::{from_str, Value};

#[test]
fn test_map_order() {
    // Sorted order
    #[cfg(not(feature = "preserve_order"))]
    const EXPECTED: &[&str] = &["a", "b", "c"];

    // Insertion order
    #[cfg(feature = "preserve_order")]
    const EXPECTED: &[&str] = &["b", "a", "c"];

    let v: Value = from_str(r#"{"b" nil "a" nil "c" nil}"#).unwrap();
    let keys: Vec<_> = v.as_object().unwrap().keys().collect();
    assert_eq!(keys, EXPECTED);
}
