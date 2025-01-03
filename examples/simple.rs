use nom::character::complete::space1;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::{IResult, Parser};
use nom_parse_trait::{ParseFrom, ParseFromExt};

struct Numbers(Vec<u32>);

impl<'a> ParseFrom<&'a str> for Numbers {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(separated_list1(space1, u32::parse), |v| Numbers(v)).parse(input)
    }
}

fn main() {
    let input = "1 2 3 4 5";
    if let Ok(numbers) = Numbers::parse_complete(input) {
        println!("Parsed \"{}\" into {:?}", input, numbers.0);
    }
}
