use nom::*;
use std::ops::*;
use nom::error::ParseError;

/// #nom-parsable
///
///

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
    I: InputLength,
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

macro_rules! unsigned_parsable {
    ($($ty:tt)+) => {
        $(
        impl<I, E: error::ParseError<I>> ParseFrom<I, E> for $ty
        where
            I: InputIter + Slice<RangeFrom<usize>> + InputLength,
            <I as InputIter>::Item: AsChar,
        {
            fn parse(input: I) -> nom::IResult<I, $ty, E> {
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
            I: InputIter + Slice<RangeFrom<usize>> + InputLength + InputTake + Clone,
            <I as InputIter>::Item: AsChar,
            I: for <'a> Compare<&'a[u8]>,
        {
            fn parse(input: I) -> nom::IResult<I, $ty, E> {
                nom::character::complete::$ty(input)
            }
        }
        )*
    }
}

signed_parsable!(i16 i32 i64 i128);

impl<I, E: ParseError<I>> ParseFrom<I, E> for bool
where
    // From alt
    I: Clone,
    // From tag
    I: InputTake + Compare<&'static str>,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        branch::alt((
            combinator::value(true, bytes::complete::tag("true")),
            combinator::value(false, bytes::complete::tag("false")),
        )).parse(input)
    }
}

impl<I, E: ParseError<I>, T: ParseFrom<I, E>> ParseFrom<I, E> for Vec<T>
where
    // From separated_list0
    I: Clone + InputLength,
    // From line_ending
    I: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
    I: InputIter + InputLength,
    I: Compare<&'static str>,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        multi::separated_list0(character::complete::line_ending, T::parse).parse(input)
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

    mod vec {
        use crate::*;
        use nom::error::*;

        #[test]
        fn test_list_of_numbers() {
            let input = "1\n2\n3\n4\n5";
            let expected = vec![1, 2, 3, 4, 5];

            assert_eq!(
                Ok::<_, Error<_>>(expected),
                Vec::<u32>::parse_complete(input)
            );
        }
    }
}
