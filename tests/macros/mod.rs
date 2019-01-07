// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! edn_str {
    ([]) => {
        "[]"
    };
    ([ $e1:tt $(, $e:tt)* ]) => {
        concat!("[",
            edn_str!($e1),
            $(",", edn_str!($e),)*
        "]")
    };
    ({}) => {
        "{}"
    };
    ({ $k1:tt : $v1:tt $(, $k:tt : $v:tt)* }) => {
        concat!("{",
            stringify!($k1), ":", edn_str!($v1),
            $(",", stringify!($k), ":", edn_str!($v),)*
        "}")
    };
    (($other:tt)) => {
        $other
    };
    ($other:tt) => {
        stringify!($other)
    };
}

macro_rules! pretty_str {
    ($edn:tt) => {
        pretty_str_impl!("", $edn)
    };
}

macro_rules! pretty_str_impl {
    ($indent:expr, []) => {
        "[]"
    };
    ($indent:expr, [ $e1:tt $(, $e:tt)* ]) => {
        concat!("[\n  ",
            $indent, pretty_str_impl!(concat!("  ", $indent), $e1),
            $(",\n  ", $indent, pretty_str_impl!(concat!("  ", $indent), $e),)*
        "\n", $indent, "]")
    };
    ($indent:expr, {}) => {
        "{}"
    };
    ($indent:expr, { $k1:tt : $v1:tt $(, $k:tt : $v:tt)* }) => {
        concat!("{\n  ",
            $indent, stringify!($k1), ": ", pretty_str_impl!(concat!("  ", $indent), $v1),
            $(",\n  ", $indent, stringify!($k), ": ", pretty_str_impl!(concat!("  ", $indent), $v),)*
        "\n", $indent, "}")
    };
    ($indent:expr, ($other:tt)) => {
        $other
    };
    ($indent:expr, $other:tt) => {
        stringify!($other)
    };
}
