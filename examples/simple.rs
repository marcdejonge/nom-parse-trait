use nom::character::complete::space1;
use nom::combinator::map;
use nom::error::Error;
use nom::multi::separated_list1;
use nom::{IResult, Parser};
use nom_parsable::Parsable;

struct Numbers(Vec<u32>);

impl Parsable for Numbers {
    fn parse(input: &str) -> IResult<&str, Self, Error<&str>> {
        map(separated_list1(space1, u32::parse), |v| Numbers(v)).parse(input)
    }
}

fn main() {
    let input = "1 2 3 4 5";
    if let Ok( numbers) = Numbers::parse_complete(input) {
        println!("Parsed \"{}\" into {:?}", input, numbers.0);
    }
}
