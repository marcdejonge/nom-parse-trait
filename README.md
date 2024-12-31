# nom-parsable
This is an extension to the popular nom crate, that provides a `Parsable` trait that can be implemented on any data that can be parsed. This means it should have a `parse` function available. It also provides a `parsable` macro that generates this trait implementation with many sensible defaults.
