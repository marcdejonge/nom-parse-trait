use nom::character::complete::space1;
use nom::combinator::map;
use nom::error::{Error, ParseError};
use nom::multi::separated_list1;
use nom::*;
use nom_parse_trait::{ParseFrom, ParseFromExt};

struct Numbers(Vec<u32>);

impl<I, E: ParseError<I>> ParseFrom<I, E> for Numbers
where
    I: Input,
    <I as Input>::Item: AsChar,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        map(separated_list1(space1, u32::parse), |v| Numbers(v)).parse(input)
    }
}

fn main() {
    let input = "1 2 3 4 5";
    if let Ok::<_, Error<_>>(numbers) = Numbers::parse_complete(input) {
        println!("Parsed \"{}\" into {:?}", input, numbers.0);
    }
}
