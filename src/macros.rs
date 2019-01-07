// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Construct a `serde_edn::Value` from a edn literal.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_edn;
/// #
/// # fn main() {
/// let value = edn!({
///     "code": 200,
///     "success": true,
///     "payload": {
///         "features": [
///             "serde",
///             "edn"
///         ]
///     }
/// });
/// # }
/// ```
///
/// Variables or expressions can be interpolated into the edn literal. Any type
/// interpolated into an array element or object value must implement Serde's
/// `Serialize` trait, while any type interpolated into a object key must
/// implement `Into<String>`. If the `Serialize` implementation of the
/// interpolated type decides to fail, or if the interpolated type contains a
/// map with non-string keys, the `edn!` macro will panic.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_edn;
/// #
/// # fn main() {
/// let code = 200;
/// let features = vec!["serde", "edn"];
///
/// let value = edn!({
///    "code": code,
///    "success": code == 200,
///    "payload": {
///        features[0]: features[1]
///    }
/// });
/// # }
/// ```
///
/// Trailing commas are allowed inside both arrays and objects.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_edn;
/// #
/// # fn main() {
/// let value = edn!([
///     "notice",
///     "the",
///     "trailing",
///     "comma -->",
/// ]);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! edn {
    // Hide distracting implementation details from the generated rustdoc.
    ($($edn:tt)+) => {
        edn_internal!($($edn)+)
    };
}

// Rocket relies on this because they export their own `edn!` with a different
// doc comment than ours, and various Rust bugs prevent them from calling our
// `edn!` from their `edn!` so they call `edn_internal!` directly. Check with
// @SergioBenitez before making breaking changes to this macro.
//
// Changes are fine as long as `edn_internal!` does not call any new helper
// macros and can still be invoked as `edn_internal!($($edn)+)`.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! edn_internal {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an array [...]. Produces a vec![...]
    // of the elements.
    //
    // Must be invoked as: edn_internal!(@array [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        edn_internal_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        edn_internal_vec![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)* edn_internal!(null)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)* edn_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)* edn_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)* edn_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)* edn_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)* edn_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        edn_internal!(@array [$($elems,)* edn_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        edn_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        edn_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: edn_internal!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        edn_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        edn_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!(null)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        edn_internal!(@object $object [$($key)+] (edn_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        edn_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        edn_internal!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        edn_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        edn_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        edn_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: edn_internal!($($edn)+)
    //////////////////////////////////////////////////////////////////////////

    (null) => {
        $crate::Value::Null
    };

    (true) => {
        $crate::Value::Bool(true)
    };

    (false) => {
        $crate::Value::Bool(false)
    };

    ([]) => {
        $crate::Value::Array(edn_internal_vec![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::Array(edn_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Object($crate::Map::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Object({
            let mut object = $crate::Map::new();
            edn_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        $crate::to_value(&$other).unwrap()
    };
}

// The edn_internal macro above cannot invoke vec directly because it uses
// local_inner_macros. A vec invocation there would resolve to $crate::vec.
// Instead invoke vec here outside of local_inner_macros.
#[macro_export]
#[doc(hidden)]
macro_rules! edn_internal_vec {
    ($($content:tt)*) => {
        vec![$($content)*]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! edn_unexpected {
    () => {};
}
