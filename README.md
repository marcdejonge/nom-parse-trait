# nom-parse-trait

[![CI](https://github.com/marcdejonge/nom-parse-trait/actions/workflows/ci.yml/badge.svg)](https://github.com/marcdejonge/nom-parse-trait/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/nom-parse-trait.svg)](https://crates.io/crates/nom-parse-trait)
[![Documentation](https://docs.rs/nom-parse-trait/badge.svg)](https://docs.rs/nom-parse-trait)

This is an extension to the popular nom crate, that provides a `ParseFrom` 
trait that can be implemented on any data that can be parsed in a singular way.
This means it should have a `parse` function available and the signature of
that function is compatible with the `nom::Parser` trait.

## Generic vs Specific parsers

The `ParseFrom` trait is generic over the input type, which means that you can
define it generically over any input type that nom supports. The downside of this
is that you will need a bunch of restrictions to the input type in a `where` block. 
Also, using a generic parser implementation can be more annoying to use, since in
some cases Rust can't infer the type of the input or error. See the
[generic_input](examples/generic_input.rs) example for an example of this.

If you already know what types of input and error you are going to use in the program,
using a specific implementation can be more convenient. See the [simple](examples/simple.rs)
example for an example of this.

## Default implementations

There are a couple of default implementations provided by this library. Since for these
standard types you cannot implement `ParseFrom` outside of this library, there are
choices made for the parsing of these types. Mostly these are for text-based parsing.
If you want your own implementation, you can wrap the type in a struct and implement
`ParseFrom` for that struct.

### Primitives
It provides a `ParseFrom` implementation for a couple of primitive numbers, based on
normal text parsing (e.g. the string "123" will be parsed to the number 123).

- `i16`
- `i32`
- `i64`
- `i128`
- `u16`
- `u32`
- `u64`
- `u128`

Also, the `bool` type has a default implementation, where it will parse the string
"true" to `true` and "false" to `false`.

### `Vec<T>`
A default implementation for Vec<T> has been provided, as long as T implements 
`ParseFrom`, where it uses the `nom::character::complete::line_ending` parser
to separate the elements.

### `HashMap<T, S>`
Similar to the `Vec<T>` implementation, you can also use a HashSet directly.

### `HashMap<K, V, S>`
A default implementation for `HashMap<K, V>` has been provided, as long as K and V
implement `ParseFrom`. It uses the `nom::character::complete::line_ending` parser
to separate the key-value pairs and separates the key and value with an equals sign (=).

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.