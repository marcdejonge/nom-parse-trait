# nom-parsable

This is an extension to the popular nom crate, that provides a `ParseFrom` 
trait that can be implemented on any data that can be parsed in a singular way.
This means it should have a `parse` function available and the signature of
that function is compatible with the `nom::Parser` trait.

## Default implementations

### Primitive numbers
It provides a `ParseFrom` implementation for a couple of primitive types:

- `i16`
- `i32`
- `i64`
- `i128`
- `u16`
- `u32`
- `u64`
- `u128`

### `Vec<T>`
A default implementation for Vec<T> has been provided, as long as T implements 
`ParseFrom`, where it uses the `nom::character::complete::line_ending` parser
to separate the elements.

Since it's not possible to implement `ParseFrom` for `Vec<T>` outside of this
library, this is the choice that has been made for separating items. If you
want you're own implementation, wrap the `Vec<T>` in a struct and implement
`ParseFrom` for that struct.

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