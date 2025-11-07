//! #nom-parse-trait
//!
//! This is an extension to the popular nom crate, that provides a `ParseFrom`
//! trait that can be implemented on any data that can be parsed in a singular way.
//! This means it should have a `parse` function available and the signature of
//! that function is compatible with the `nom::Parser` trait.
//!
//! The main usage of this is to easily combine parsers of different types.
//! To see the real power of this trait, take a look at he nom-parse-macros trait,
//! which makes it possible easily implement this trait on data types.
//!
//! ## Generic vs Specific parsers
//!
//! The `ParseFrom` trait is generic over the input type, which means that you can
//! define it generically over any input type that nom supports. The downside of this
//! is that you will need a bunch of restrictions to the input type in a `where` block.
//! Also, using a generic parser implementation can be more annoying to use, since in
//! some cases Rust can't infer the type of the input or error. See the
//! [generic_input](examples/generic_input.rs) example for an example of this.
//!
//! If you already know what types of input and error you are going to use in the program,
//! using a specific implementation can be more convenient. See the [simple](examples/simple.rs)
//! example for an example of this.

use branch::alt;
use combinator::value;
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::*;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hash};

/// A trait for types that can be parsed from the given input.
pub trait ParseFrom<I, E = error::Error<I>>
where
    Self: Sized,
{
    /// A function that can act as a nom Parser type that parses some of the input and
    /// returns an instance of this type.
    fn parse(input: I) -> IResult<I, Self, E>;
}

/// An extension for the ParseFrom trait with extra functionality to make parse a bit easier.
pub trait ParseFromExt<I, E>
where
    Self: Sized,
{
    fn parse_complete(input: I) -> Result<Self, E>;
}

impl<I, E, T: ParseFrom<I, E>> ParseFromExt<I, E> for T
where
    I: Input,
    E: error::ParseError<I>,
{
    /// Parse the input and return the result if the input is fully consumed.
    ///
    /// If the input is not fully consumed, an "Eof" error is returned with the rest of the characters.
    ///
    /// # Panics
    /// This function panics if the parser returns an "Incomplete" error. If you want to use this
    /// on streaming parser, please make sure you handle the "Incomplete" error before calling this function.
    fn parse_complete(input: I) -> Result<Self, E> {
        match T::parse(input) {
            Ok((rest, result)) if rest.input_len() == 0 => Ok(result),
            Ok((rest, _)) => Err(E::from_error_kind(rest, error::ErrorKind::Eof)),
            Err(Err::Incomplete(_)) => panic!("Parser returned an incomplete error"),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(e),
        }
    }
}

macro_rules! wrapper_types {
    ($($ty:ty),+) => {
        $(
            impl<I, E: error::ParseError<I>, T: ParseFrom<I, E>> ParseFrom<I, E> for $ty {
                fn parse(input: I) -> IResult<I, Self, E> {
                    combinator::map(T::parse, |val| Self::new(val)).parse(input)
                }
            }
        )*
    }
}

wrapper_types!(
    Box<T>,
    std::cell::Cell<T>,
    std::cell::RefCell<T>,
    std::rc::Rc<T>,
    std::sync::Arc<T>,
    std::sync::Mutex<T>,
    std::sync::RwLock<T>
);

macro_rules! unsigned_parsable {
    ($($ty:tt)+) => {
        $(
        impl<I, E: error::ParseError<I>> ParseFrom<I, E> for $ty
        where
            I: Input,
            <I as Input>::Item: AsChar,
        {
            fn parse(input: I) -> nom::IResult<I, Self, E> {
                nom::character::complete::$ty(input)
            }
        }
        )*
    }
}

unsigned_parsable!(u16 u32 u64 u128);

macro_rules! signed_parsable {
    ($($ty:tt)+) => {
        $(
        impl<I, E: error::ParseError<I>> ParseFrom<I, E> for $ty
        where
            I: Input,
            <I as Input>::Item: AsChar,
            I: for <'a> Compare<&'a[u8]>,
        {
            fn parse(input: I) -> nom::IResult<I, Self, E> {
                nom::character::complete::$ty(input)
            }
        }
        )*
    }
}

signed_parsable!(i8 i16 i32 i64 i128);

macro_rules! floating_parsable {
    ($($ty:tt)+) => {
        $(
        impl<I, E: error::ParseError<I>> ParseFrom<I, E> for $ty
        where
            I: Input + Offset + AsBytes + Compare<&'static str>,
            <I as Input>::Item: AsChar,
            <I as Input>::Iter: Clone,
            I: for<'a> Compare<&'a [u8]>,
        {
            fn parse(input: I) -> nom::IResult<I, Self, E> {
                use std::str::FromStr;
                use nom::number::complete::recognize_float_or_exceptions;
                use std::str::from_utf8;

                let (i, s) = recognize_float_or_exceptions(input)?;
                match from_utf8(s.as_bytes()).ok().and_then(|s| $ty::from_str(s).ok()) {
                    Some(f) => Ok((i, f)),
                    None => Err(nom::Err::Error(E::from_error_kind(i, nom::error::ErrorKind::Float))),
                }
            }
        }
        )*
    }
}

floating_parsable!(f32 f64);

/// Support reading the words "true" or "false" from the input and interpreting them as boolean values.
impl<I, E: error::ParseError<I>> ParseFrom<I, E> for bool
where
    I: Input + Compare<&'static str>,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        alt((value(true, tag("true")), value(false, tag("false")))).parse(input)
    }
}

/// Support reading a single character from the input.
impl<I, E: error::ParseError<I>> ParseFrom<I, E> for char
where
    I: Input,
    <I as Input>::Item: AsChar,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        let char = input
            .iter_elements()
            .next()
            .ok_or_else(|| Err::Error(E::from_error_kind(input.clone(), error::ErrorKind::Eof)))?
            .as_char();
        let (rest, _) = input.take_split(char.len());
        Ok((rest, char))
    }
}

/// Support reading a single byte from the input. This is NOT a parsed number, but the raw byte value.
impl<I, E: error::ParseError<I>> ParseFrom<I, E> for u8
where
    I: Input,
    <I as Input>::Item: AsBytes,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        let item = input
            .iter_elements()
            .next()
            .ok_or_else(|| Err::Error(E::from_error_kind(input.clone(), error::ErrorKind::Eof)))?;
        let bytes = item.as_bytes();
        if bytes.len() != 1 {
            return Err(Err::Error(E::from_error_kind(
                input,
                error::ErrorKind::Char,
            )));
        }
        let (rest, _) = input.take_split(bytes.len());
        Ok((rest, bytes[0]))
    }
}

/// Support parsing a vector of ParseFrom types from the input. This uses the line_ending parser
/// to separate the items.
impl<I, E: error::ParseError<I>, T: ParseFrom<I, E>> ParseFrom<I, E> for Vec<T>
where
    I: Input + Compare<&'static str>,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        multi::separated_list0(character::complete::line_ending, T::parse).parse(input)
    }
}

/// Support parsing a HashSet of ParseFrom types from the input. This uses the line_ending parser
/// to separate the items.
impl<I, E: error::ParseError<I>, T: ParseFrom<I, E>, S> ParseFrom<I, E> for HashSet<T, S>
where
    I: Input + Compare<&'static str>,
    T: Eq + Hash,
    S: BuildHasher + Default,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        combinator::map(
            multi::separated_list0(character::complete::line_ending, T::parse),
            |list| list.into_iter().collect(),
        )
        .parse(input)
    }
}

/// Support parsing a HashMap of ParseFrom types from the input. This uses the line_ending parser
/// to separate the items and the "=" sign to separate the key and value.
impl<I, E: error::ParseError<I>, K: ParseFrom<I, E>, V: ParseFrom<I, E>, S> ParseFrom<I, E>
    for HashMap<K, V, S>
where
    I: Input + Compare<&'static str>,
    <I as Input>::Item: AsChar + Copy,
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        combinator::map(
            multi::separated_list0(
                character::complete::line_ending,
                sequence::separated_pair(K::parse, (space0, tag("="), space0), V::parse),
            ),
            |list| list.into_iter().collect(),
        )
        .parse(input)
    }
}

impl<const N: usize, I, E: error::ParseError<I>, T: ParseFrom<I, E>> ParseFrom<I, E> for [T; N]
where
    I: Input + Compare<&'static str>,
    <I as Input>::Item: AsChar + Copy,
{
    fn parse(mut input: I) -> IResult<I, Self, E> {
        use std::mem::*;
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        if N > 0 {
            let mut separator = (space0, tag::<_, I, E>(","), space0);

            let (rest, value) = T::parse(input)?;
            arr[0].write(value);
            input = rest;

            for i in 1..N {
                match separator.parse(input).map(|(rest, _)| T::parse(rest)) {
                    Ok(Ok((rest, value))) => {
                        arr[i].write(value);
                        input = rest;
                    }
                    Ok(Err(e)) | Err(e) => {
                        // There was an error parsing the separator or the value
                        // We need to clean up the already initialized elements
                        unsafe {
                            arr[0..i].iter_mut().for_each(|it| it.assume_init_drop());
                        }
                        return Err(e);
                    }
                }
            }
        }
        Ok((input, arr.map(|x| unsafe { x.assume_init() })))
    }
}

#[cfg(test)]
mod tests {
    macro_rules! test_unsigned {
        ($($ty:tt)+) => {
            $(
                mod $ty {
                    use crate::*;
                    use nom::error::*;

                    #[test]
                    fn test_normal_parsing() {
                        assert_eq!(Ok::<_, Error<_>>(123), $ty::parse_complete(b"123".as_ref()));
                        assert_eq!(Ok::<_, Err<Error<_>>>((b"a".as_ref(), 999)), $ty::parse(b"999a".as_ref()));

                        assert_eq!(Ok::<_, Error<_>>(123), $ty::parse_complete("123"));
                        assert_eq!(Ok::<_, Err<Error<_>>>(("a", 999)), $ty::parse("999a"));
                    }

                    #[test]
                    fn test_overflow() {
                        let too_big = format!("{}00", $ty::MAX);

                        assert_eq!(
                            Err(Error::from_error_kind(too_big.as_str(), ErrorKind::Digit)),
                            u16::parse_complete(too_big.as_str())
                        );
                        assert_eq!(
                            Err(Error::from_error_kind(too_big.as_bytes(), ErrorKind::Digit)),
                            u16::parse_complete(too_big.as_bytes())
                        );
                    }
                }
            )*
        };
    }

    test_unsigned!(u16 u32 u64 u128);
    test_unsigned!(i16 i32 i64 i128);

    mod floats {
        use crate::*;

        #[test]
        fn parse_f32() {
            assert_eq!(Ok::<_, ()>(6e8), f32::parse_complete("6e8"));
            assert_eq!(
                Ok::<_, ()>(3.14e-2),
                f32::parse_complete(b"3.14e-2".as_ref())
            );
        }

        #[test]
        fn parse_f64() {
            assert_eq!(Ok::<_, ()>(6e8), f64::parse_complete("6e8"));
            assert_eq!(
                Ok::<_, ()>(3.14e-2),
                f64::parse_complete(b"3.14e-2".as_ref())
            );
        }
    }

    mod char {
        use crate::*;
        use nom::error::*;
        use nom::multi::many1;

        #[test]
        fn read_characters() {
            let input = "TðŒ🏃";

            let result: Result<_, Error<_>> = many1(char::parse).parse(input).finish();

            assert_eq!(Ok(("", vec!['T', 'ð', 'Œ', '🏃'])), result);
        }

        #[test]
        fn read_bytes() {
            let input = b"1234".as_ref();

            let result: Result<_, Error<_>> = many1(char::parse).parse(input).finish();

            assert_eq!(Ok((b"".as_ref(), vec!['1', '2', '3', '4'])), result);
        }
    }

    mod collections {
        use crate::*;
        use nom::error::*;

        #[test]
        fn test_vec_of_numbers() {
            let input = "1\n2\n3\n4\n5";
            let expected = vec![1, 2, 3, 4, 5];

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                Vec::<u32>::parse_complete(input)
            );
        }

        #[test]
        fn test_set_of_numbers() {
            let input = "1\n2\n3\n4\n5";
            let expected = vec![1, 2, 3, 4, 5].into_iter().collect();

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                HashSet::<u32>::parse_complete(input)
            );
        }

        #[test]
        fn test_map_of_numbers() {
            let input = "a = 1\nb = 2\nc = 3\nd = 4\ne = 5";
            let expected = vec![('a', 1), ('b', 2), ('c', 3), ('d', 4), ('e', 5)]
                .into_iter()
                .collect();

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                HashMap::<char, u32>::parse_complete(input)
            );
        }

        #[test]
        fn test_array_of_numbers() {
            let input = "1, 2, 3, 4, 5";
            let expected = [1, 2, 3, 4, 5];

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                <[u32; 5]>::parse_complete(input)
            )
        }

        #[test]
        fn test_empty_array_of_numbers() {
            let input = "";
            let expected: [u32; 0] = [];

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                <[u32; 0]>::parse_complete(input)
            );
        }
    }

    mod wrapping {
        use std::{rc::Rc, sync::Arc};

        use crate::*;
        use nom::error::*;

        #[test]
        fn test_box() {
            let input = "12";
            let expected = Box::new(12i32);

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                Box::<i32>::parse_complete(input)
            )
        }

        #[test]
        fn test_rc() {
            let input = "12";
            let expected = Rc::new(12i32);

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                Rc::<i32>::parse_complete(input)
            )
        }

        #[test]
        fn test_arc() {
            let input = "12";
            let expected = Arc::new(12i32);

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                Arc::<i32>::parse_complete(input)
            )
        }
    }
}
