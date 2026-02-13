//! Grammar Reference
//!
//! NamedList := NamedTakeAdd (,NamedTakeAdd)*
//! NamedTakeAdd := (Name ':')? _ TakeAdd
//! Name := [A-Za-z_]+
//! TakeAdd := TakeFactor (_ [* | /] _ TakeAdd | TakeFactor)
//! TakeFactor := TakeRecursive (_ [+ | -] _ TakeFactor | TakeRecursive)
//! TakeRecursive := Take | _ '(' _ TakeAdd _ ')'
//! Take := Dice ([hHlL]\d+)?
//! Dice := [\d+] 'd' [\d+]
//! _ := [ \n\r]*

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, space0, space1},
    combinator::{map_res, opt},
    multi::many0,
};

use crate::dice::Parse;
use std::rc::Rc;

#[derive(Debug)]
pub enum OpAdd {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum OpFactor {
    Multiply,
    Divide,
}

#[derive(Debug)]
pub struct Dice {
    pub count: u32,
    pub die: Option<u32>,
}

#[derive(Debug)]
pub enum FilterType {
    Higher,
    Lower,
}

#[derive(Debug)]
pub struct Take {
    pub dice: Rc<Dice>,
    pub filter: Option<(u32, FilterType)>,
}

#[derive(Debug)]
pub enum TakeRecursive {
    Take(Take),
    TakeAdd(TakeAdd),
}

#[derive(Debug)]
pub enum TakeFactorRight {
    TakeFactor(TakeFactor),
    Take(TakeRecursive),
}

#[derive(Debug)]
pub struct TakeFactor {
    pub left: Rc<TakeRecursive>,
    pub right: Option<(OpFactor, Rc<TakeFactorRight>)>,
}

#[derive(Debug)]
pub enum TakeAddRight {
    TakeFactor(TakeFactor),
    TakeAdd(TakeAdd),
}

#[derive(Debug)]
pub struct TakeAdd {
    pub left: Rc<TakeFactor>,
    pub right: Option<(OpAdd, Rc<TakeAddRight>)>,
}

#[derive(Debug)]
pub struct NamedTakeAdd {
    pub name: Option<String>,
    pub expression: Rc<TakeAdd>,
}

#[derive(Debug)]
pub struct NamedList {
    pub expressions: Vec<NamedTakeAdd>,
}

impl Parse for OpAdd {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, char) = alt((char('+'), char('-'))).parse(input)?;
        if char == '+' {
            Ok((input, OpAdd::Plus))
        } else {
            Ok((input, OpAdd::Minus))
        }
    }
}

impl Parse for OpFactor {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, char) = alt((char('*'), char('/'))).parse(input)?;
        if char == '*' {
            Ok((input, OpFactor::Multiply))
        } else {
            Ok((input, OpFactor::Divide))
        }
    }
}

impl Parse for Dice {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, count): (&str, u32) = map_res(digit1, str::parse).parse(input)?;

        let (input, optional_die): (&str, Option<(char, u32)>) =
            opt((char('d'), map_res(digit1, str::parse))).parse(input)?;

        Ok((
            input,
            Dice {
                count,
                die: optional_die.map(|(_, die)| die),
            },
        ))
    }
}

impl Parse for Take {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, dice) = Dice::parse(input)?;

        let (input, optional_filter): (&str, Option<(char, u32)>) = opt((
            alt((char('h'), char('H'), char('l'), char('L'))),
            map_res(digit1, str::parse),
        ))
        .parse(input)?;

        Ok((
            input,
            Take {
                dice: Rc::new(dice),
                filter: optional_filter.map(|(filter_type_char, count)| {
                    (
                        count,
                        match filter_type_char {
                            'h' | 'H' => FilterType::Higher,
                            _ => FilterType::Lower,
                        },
                    )
                }),
            },
        ))
    }
}

impl Parse for TakeRecursive {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            (space0, char('('), space0, TakeAdd::parse, space0, char(')'))
                .map(|(_, _, _, take_add, _, _)| TakeRecursive::TakeAdd(take_add)),
            Take::parse.map(|take| TakeRecursive::Take(take)),
        ))
        .parse(input)
    }
}

impl Parse for TakeFactor {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, left_expr) = TakeRecursive::parse(input)?;
        let (input, right_option) = opt((
            space0,
            OpFactor::parse,
            space0,
            alt((
                TakeRecursive::parse.map(|take_recursive| TakeFactorRight::Take(take_recursive)),
                TakeFactor::parse.map(|take_factor| TakeFactorRight::TakeFactor(take_factor)),
            )),
        ))
        .parse(input)?;

        Ok((
            input,
            TakeFactor {
                left: Rc::new(left_expr),
                right: right_option.map(|(_, op, _, node)| (op, Rc::new(node))),
            },
        ))
    }
}

impl Parse for TakeAdd {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, left_expr) = TakeFactor::parse(input)?;
        let (input, right_option) = opt((
            space0,
            OpAdd::parse,
            space0,
            alt((
                TakeAdd::parse.map(|take_add| TakeAddRight::TakeAdd(take_add)),
                TakeFactor::parse.map(|take_factor| TakeAddRight::TakeFactor(take_factor)),
            )),
        ))
        .parse(input)?;

        Ok((
            input,
            TakeAdd {
                left: Rc::new(left_expr),
                right: right_option.map(|(_, op, _, node)| (op, Rc::new(node))),
            },
        ))
    }
}

impl Parse for NamedTakeAdd {
    fn parse(input: &str) -> IResult<&str, Self> {
        fn parse_name(input: &str) -> IResult<&str, String> {
            let (input, slice) = many0(alt((alpha1, space1))).parse(input)?;
            let (input, _) = tag(":")(input)?;
            Ok((input, slice.join("").to_owned()))
        }

        let (input, name_option) = opt(parse_name).parse(input)?;
        let (input, _) = space0(input)?;
        let (input, dice_expression) = TakeAdd::parse(input)?;
        Ok((
            input,
            NamedTakeAdd {
                name: name_option,
                expression: Rc::new(dice_expression),
            },
        ))
    }
}

impl Parse for NamedList {
    fn parse(input: &str) -> IResult<&str, Self> {
        let mut expressions: Vec<NamedTakeAdd> = Vec::new();

        let (input, named_take) = NamedTakeAdd::parse(input)?;
        expressions.push(named_take);

        let (input, optional_named_list) =
            opt(many0((space0, tag(","), space0, NamedTakeAdd::parse))).parse(input)?;

        match optional_named_list {
            None => {}
            Some(parsed_expressions) => {
                for (_, _, _, named_take) in parsed_expressions {
                    expressions.push(named_take);
                }
            }
        }

        Ok((input, NamedList { expressions }))
    }
}
