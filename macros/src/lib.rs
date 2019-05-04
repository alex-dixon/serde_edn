#![feature(proc_macro_hygiene)]

#![feature(proc_macro_diagnostic, proc_macro_span)]
extern crate proc_macro;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use self::proc_macro::TokenStream;
use syn::Expr;

#[proc_macro]
pub fn say_hello(input: TokenStream) -> TokenStream {
    // This macro will accept any input because it ignores it.
    // To enforce correctness in macros which don't take input,
    // you may want to add `assert!(_input.to_string().is_empty());`.
    let item: Expr  = parse_macro_input!(input);
    // syn::parse(input).expect("failed to parse input");
    println!("{:?}",item);

//    let output = quote!{ #item };
//    output.into()
    TokenStream::from(quote!())

}

#[cfg(test)]
mod tests {
    use super::say_hello;
    #[test]
    fn it_works() {
        assert_eq!(say_hello!([1,2,3]), 2);
    }
}
