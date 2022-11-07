use super::{Op, Constraint};

use nom::{
    IResult,
    character::complete::one_of,
    bytes::complete::is_a,
    multi::{separated_list1},
    Parser, sequence::{delimited, separated_pair},
};

fn char(c: char) -> impl Fn(&str) -> IResult<&str, char> {
    move |input| {
        let input = input.trim_start();
        nom::character::complete::char(c).parse(input)
    }
}

pub fn constraint(input: &str) -> IResult<&str, Constraint> {
    let (input, result) = u8(input)?;
    let (input, op) = op(input)?;
    let (input, cells) = cells(input)?;
    Ok((input, Constraint { cells, op, result }))
}

fn cell(input: &str) -> IResult<&str, (usize,usize)> {
    let input = input.trim_start();
    delimited(char('('), separated_pair(usize, char(','), usize), char(')')).parse(input)
}

fn cells(input: &str) -> IResult<&str, Vec<(usize,usize)>> {
    let input = input.trim_start();
    delimited(char('['),
              separated_list1(char(','), cell),
              char(']')).parse(input)
}

pub fn op(input: &str) -> IResult<&str, Op> {
    let input = input.trim_start();
    one_of("+-*/").map(|c| match c {
        '+' => Op::Plus,
        '-' => Op::Minus,
        '*' => Op::Times,
        '/' => Op::Div,
         _  => unreachable!(),
    }).parse(input)
}

fn usize(input: &str) -> IResult<&str, usize> {
    let input = input.trim_start();
    is_a("0123456789").map(|s: &str| s.parse().unwrap()).parse(input)    
}

fn u8(input: &str) -> IResult<&str, u8> {
    let input = input.trim_start();
    is_a("0123456789").map(|s: &str| s.parse().unwrap()).parse(input)
}

#[test]
fn test_parser() {
    assert_eq!(constraint("30* [ (0,3), (1,3), (2,2), (2,3) ]").unwrap(), ("", Constraint { op: Op::Times, result: 30, cells: vec![ (0,3), (1,3), (2,2), (2,3)] } ));
}