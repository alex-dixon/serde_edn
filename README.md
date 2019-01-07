# Serde edn &emsp; [![Build Status]][travis] [![Latest Version]][crates.io] [![Rustc Version 1.15+]][rustc]

[Build Status]: https://api.travis-ci.org/serde-rs/edn.svg?branch=master
[travis]: https://travis-ci.org/serde-rs/edn
[Latest Version]: https://img.shields.io/crates/v/serde_edn.svg
[crates.io]: https://crates.io/crates/serde\_edn
[Rustc Version 1.15+]: https://img.shields.io/badge/rustc-1.15+-lightgray.svg
[rustc]: https://blog.rust-lang.org/2017/02/02/Rust-1.15.html

**Serde is a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.**

---

```toml
[dependencies]
serde_edn = "1.0"
```

You may be looking for:

- [edn API documentation](https://docs.serde.rs/serde_edn/)
- [Serde API documentation](https://docs.serde.rs/serde/)
- [Detailed documentation about Serde](https://serde.rs/)
- [Setting up `#[derive(Serialize, Deserialize)]`](https://serde.rs/codegen.html)
- [Release notes](https://github.com/serde-rs/edn/releases)

edn is a ubiquitous open-standard format that uses human-readable text to
transmit data objects consisting of key-value pairs.

```edn,ignore
{
  "name": "John Doe",
  "age": 43,
  "address": {
    "street": "10 Downing Street",
    "city": "London"
  },
  "phones": [
    "+44 1234567",
    "+44 2345678"
  ]
}
```

There are three common ways that you might find yourself needing to work
with edn data in Rust.

 - **As text data.** An unprocessed string of edn data that you receive on
   an HTTP endpoint, read from a file, or prepare to send to a remote
   server.
 - **As an untyped or loosely typed representation.** Maybe you want to
   check that some edn data is valid before passing it on, but without
   knowing the structure of what it contains. Or you want to do very basic
   manipulations like insert a key in a particular spot.
 - **As a strongly typed Rust data structure.** When you expect all or most
   of your data to conform to a particular structure and want to get real
   work done without edn's loosey-goosey nature tripping you up.

Serde edn provides efficient, flexible, safe ways of converting data
between each of these representations.

## Operating on untyped edn values

Any valid edn data can be manipulated in the following recursive enum
representation. This data structure is [`serde_edn::Value`][value].

```rust,ignore
enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

A string of edn data can be parsed into a `serde_edn::Value` by the
[`serde_edn::from_str`][from_str] function. There is also
[`from_slice`][from_slice] for parsing from a byte slice &[u8] and
[`from_reader`][from_reader] for parsing from any `io::Read` like a File or
a TCP stream.

<a href="https://play.rust-lang.org/?gist=a266662bc71712e080efbf25ce30f306" target="_blank">
<img align="right" width="50" src="https://raw.githubusercontent.com/serde-rs/serde-rs.github.io/master/img/run.png">
</a>

```rust
extern crate serde_edn;

use serde_edn::{Value, Error};

fn untyped_example() -> Result<(), Error> {
    // Some edn input data as a &str. Maybe this comes from the user.
    let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ]
                  }"#;

    // Parse the string of data into serde_edn::Value.
    let v: Value = serde_edn::from_str(data)?;

    // Access parts of the data by indexing with square brackets.
    println!("Please call {} at the number {}", v["name"], v["phones"][0]);

    Ok(())
}
```

The result of square bracket indexing like `v["name"]` is a borrow of the data
at that index, so the type is `&Value`. A edn map can be indexed with string
keys, while a edn array can be indexed with integer keys. If the type of the
data is not right for the type with which it is being indexed, or if a map does
not contain the key being indexed, or if the index into a vector is out of
bounds, the returned element is `Value::Null`.

When a `Value` is printed, it is printed as a edn string. So in the code above,
the output looks like `Please call "John Doe" at the number "+44 1234567"`. The
quotation marks appear because `v["name"]` is a `&Value` containing a edn
string and its edn representation is `"John Doe"`. Printing as a plain string
without quotation marks involves converting from a edn string to a Rust string
with [`as_str()`] or avoiding the use of `Value` as described in the following
section.

[`as_str()`]: https://docs.serde.rs/serde_edn/enum.Value.html#method.as_str

The `Value` representation is sufficient for very basic tasks but can be tedious
to work with for anything more significant. Error handling is verbose to
implement correctly, for example imagine trying to detect the presence of
unrecognized fields in the input data. The compiler is powerless to help you
when you make a mistake, for example imagine typoing `v["name"]` as `v["nmae"]`
in one of the dozens of places it is used in your code.

## Parsing edn as strongly typed data structures

Serde provides a powerful way of mapping edn data into Rust data structures
largely automatically.

<a href="https://play.rust-lang.org/?gist=cff572b80d3f078c942a2151e6020adc" target="_blank">
<img align="right" width="50" src="https://raw.githubusercontent.com/serde-rs/serde-rs.github.io/master/img/run.png">
</a>

```rust
extern crate serde;
extern crate serde_edn;

#[macro_use]
extern crate serde_derive;

use serde_edn::Error;

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

fn typed_example() -> Result<(), Error> {
    // Some edn input data as a &str. Maybe this comes from the user.
    let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ]
                  }"#;

    // Parse the string of data into a Person object. This is exactly the
    // same function as the one that produced serde_edn::Value above, but
    // now we are asking it for a Person as output.
    let p: Person = serde_edn::from_str(data)?;

    // Do things just like with any other Rust data structure.
    println!("Please call {} at the number {}", p.name, p.phones[0]);

    Ok(())
}
```

This is the same `serde_edn::from_str` function as before, but this time we
assign the return value to a variable of type `Person` so Serde will
automatically interpret the input data as a `Person` and produce informative
error messages if the layout does not conform to what a `Person` is expected
to look like.

Any type that implements Serde's `Deserialize` trait can be deserialized
this way. This includes built-in Rust standard library types like `Vec<T>`
and `HashMap<K, V>`, as well as any structs or enums annotated with
`#[derive(Deserialize)]`.

Once we have `p` of type `Person`, our IDE and the Rust compiler can help us
use it correctly like they do for any other Rust code. The IDE can
autocomplete field names to prevent typos, which was impossible in the
`serde_edn::Value` representation. And the Rust compiler can check that
when we write `p.phones[0]`, then `p.phones` is guaranteed to be a
`Vec<String>` so indexing into it makes sense and produces a `String`.

## Constructing edn values

Serde edn provides a [`edn!` macro][macro] to build `serde_edn::Value`
objects with very natural edn syntax. In order to use this macro,
`serde_edn` needs to be imported with the `#[macro_use]` attribute.

<a href="https://play.rust-lang.org/?gist=c216d6beabd9429a6ac13b8f88938dfe" target="_blank">
<img align="right" width="50" src="https://raw.githubusercontent.com/serde-rs/serde-rs.github.io/master/img/run.png">
</a>

```rust
#[macro_use]
extern crate serde_edn;

fn main() {
    // The type of `john` is `serde_edn::Value`
    let john = edn!({
      "name": "John Doe",
      "age": 43,
      "phones": [
        "+44 1234567",
        "+44 2345678"
      ]
    });

    println!("first phone number: {}", john["phones"][0]);

    // Convert to a string of edn and print it out
    println!("{}", john.to_string());
}
```

The `Value::to_string()` function converts a `serde_edn::Value` into a
`String` of edn text.

One neat thing about the `edn!` macro is that variables and expressions can
be interpolated directly into the edn value as you are building it. Serde
will check at compile time that the value you are interpolating is able to
be represented as edn.

<a href="https://play.rust-lang.org/?gist=aae3af4d274bd249d1c8a947076355f2" target="_blank">
<img align="right" width="50" src="https://raw.githubusercontent.com/serde-rs/serde-rs.github.io/master/img/run.png">
</a>

```rust
let full_name = "John Doe";
let age_last_year = 42;

// The type of `john` is `serde_edn::Value`
let john = edn!({
  "name": full_name,
  "age": age_last_year + 1,
  "phones": [
    format!("+44 {}", random_phone())
  ]
});
```

This is amazingly convenient but we have the problem we had before with
`Value` which is that the IDE and Rust compiler cannot help us if we get it
wrong. Serde edn provides a better way of serializing strongly-typed data
structures into edn text.

## Creating edn by serializing data structures

A data structure can be converted to a edn string by
[`serde_edn::to_string`][to_string]. There is also
[`serde_edn::to_vec`][to_vec] which serializes to a `Vec<u8>` and
[`serde_edn::to_writer`][to_writer] which serializes to any `io::Write`
such as a File or a TCP stream.

<a href="https://play.rust-lang.org/?gist=40967ece79921c77fd78ebc8f177c063" target="_blank">
<img align="right" width="50" src="https://raw.githubusercontent.com/serde-rs/serde-rs.github.io/master/img/run.png">
</a>

```rust
extern crate serde;
extern crate serde_edn;

#[macro_use]
extern crate serde_derive;

use serde_edn::Error;

#[derive(Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
}

fn print_an_address() -> Result<(), Error> {
    // Some data structure.
    let address = Address {
        street: "10 Downing Street".to_owned(),
        city: "London".to_owned(),
    };

    // Serialize it to a edn string.
    let j = serde_edn::to_string(&address)?;

    // Print, write to a file, or send to an HTTP server.
    println!("{}", j);

    Ok(())
}
```

Any type that implements Serde's `Serialize` trait can be serialized this
way. This includes built-in Rust standard library types like `Vec<T>` and
`HashMap<K, V>`, as well as any structs or enums annotated with
`#[derive(Serialize)]`.

## Performance

It is fast. You should expect in the ballpark of 500 to 1000 megabytes per
second deserialization and 600 to 900 megabytes per second serialization,
depending on the characteristics of your data. This is competitive with the
fastest C and C++ edn libraries or even 30% faster for many use cases.
Benchmarks live in the [serde-rs/edn-benchmark] repo.

[serde-rs/edn-benchmark]: https://github.com/serde-rs/edn-benchmark

## Getting help

Serde developers live in the #serde channel on
[`irc.mozilla.org`](https://wiki.mozilla.org/IRC). The #rust channel is also a
good resource with generally faster response time but less specific knowledge
about Serde. If IRC is not your thing, we are happy to respond to [GitHub
issues](https://github.com/serde-rs/edn/issues/new) as well.

## No-std support

This crate currently requires the Rust standard library. For edn support in
Serde without a standard library, please see the [`serde-edn-core`] crate.

[`serde-edn-core`]: https://japaric.github.io/serde-edn-core/serde_edn_core/

## License

Serde edn is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde edn by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[value]: https://docs.serde.rs/serde_edn/value/enum.Value.html
[from_str]: https://docs.serde.rs/serde_edn/de/fn.from_str.html
[from_slice]: https://docs.serde.rs/serde_edn/de/fn.from_slice.html
[from_reader]: https://docs.serde.rs/serde_edn/de/fn.from_reader.html
[to_string]: https://docs.serde.rs/serde_edn/ser/fn.to_string.html
[to_vec]: https://docs.serde.rs/serde_edn/ser/fn.to_vec.html
[to_writer]: https://docs.serde.rs/serde_edn/ser/fn.to_writer.html
[macro]: https://docs.serde.rs/serde_edn/macro.edn.html
