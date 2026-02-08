use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, space0, space1},
    combinator::{map_res, opt},
    multi::many0,
};

use std::rc::Rc;

pub enum OpAdd {
    Plus,
    Minus,
}

impl OpAdd {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, char) = alt((char('+'), char('-'))).parse(input)?;
        if char == '+' {
            Ok((input, OpAdd::Plus))
        } else {
            Ok((input, OpAdd::Minus))
        }
    }
}

pub enum OpFactor {
    Multiply,
    Divide,
}

impl std::fmt::Display for OpFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Multiply => f.write_str("*"),
            Self::Divide => f.write_str("/"),
        }
    }
}

impl OpFactor {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, char) = alt((char('*'), char('/'))).parse(input)?;
        if char == '*' {
            Ok((input, OpFactor::Multiply))
        } else {
            Ok((input, OpFactor::Divide))
        }
    }
}

pub struct Dice {
    pub count: u32,
    pub die: Option<u32>,
}

impl Dice {
    pub fn parse(input: &str) -> IResult<&str, Self> {
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

pub struct Take {
    pub dice: Rc<Dice>,
    pub filter: Option<(u32, bool)>,
}

impl Take {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, dice) = Dice::parse(input)?;

        let (input, optional_filter): (&str, Option<(char, u32)>) = opt((
            alt((char('h'), char('H'), char('l'), char('L'))),
            map_res(digit1, str::parse),
        ))
        .parse(input)?;

        let take_object = match optional_filter {
            None => Take {
                dice: Rc::new(dice),
                filter: None,
            },
            Some((higher_or_lower_char, count)) => {
                let take_higher = higher_or_lower_char.to_ascii_lowercase() == 'h';
                Take {
                    dice: Rc::new(dice),
                    filter: Some((count, take_higher)),
                }
            }
        };

        Ok((input, take_object))
    }
}

pub enum TakeFactorRight {
    TakeFactor(TakeFactor),
    Take(Take),
}

pub struct TakeFactor {
    pub left: Rc<Take>,
    pub right: Option<(OpFactor, Rc<TakeFactorRight>)>,
}

impl TakeFactor {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        fn parse_take_or_take_factor(input: &str) -> IResult<&str, TakeFactorRight> {
            let maybe_take_factor = TakeFactor::parse(input);
            match maybe_take_factor {
                Ok((input, take_factor)) => Ok((input, TakeFactorRight::TakeFactor(take_factor))),
                Err(_) => {
                    Take::parse(input).map(|(input, take)| (input, TakeFactorRight::Take(take)))
                }
            }
        }

        let (input, left_expr) = Take::parse(input)?;
        let (input, right_option) =
            opt((space0, OpFactor::parse, space0, parse_take_or_take_factor)).parse(input)?;
        match right_option {
            Some((_, op, _, right_expr)) => Ok((
                input,
                TakeFactor {
                    left: Rc::new(left_expr),
                    right: Some((op, Rc::new(right_expr))),
                },
            )),
            None => Ok((
                input,
                TakeFactor {
                    left: Rc::new(left_expr),
                    right: None,
                },
            )),
        }
    }
}

pub enum TakeAddRight {
    TakeFactor(TakeFactor),
    TakeAdd(TakeAdd),
}

pub struct TakeAdd {
    pub left: Rc<TakeFactor>,
    pub right: Option<(OpAdd, Rc<TakeAddRight>)>,
}

impl TakeAdd {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        fn parse_take_factor_or_take_add(input: &str) -> IResult<&str, TakeAddRight> {
            let maybe_take_add = TakeAdd::parse(input);
            match maybe_take_add {
                Ok((input, take_add)) => Ok((input, TakeAddRight::TakeAdd(take_add))),
                Err(_) => TakeFactor::parse(input)
                    .map(|(input, take_factor)| (input, TakeAddRight::TakeFactor(take_factor))),
            }
        }

        let (input, left_expr) = TakeFactor::parse(input)?;
        let (input, right_option) =
            opt((space0, OpAdd::parse, space0, parse_take_factor_or_take_add)).parse(input)?;
        match right_option {
            Some((_, op, _, right_expr)) => Ok((
                input,
                TakeAdd {
                    left: Rc::new(left_expr),
                    right: Some((op, Rc::new(right_expr))),
                },
            )),
            None => Ok((
                input,
                TakeAdd {
                    left: Rc::new(left_expr),
                    right: None,
                },
            )),
        }
    }
}

pub struct NamedTakeAdd {
    pub name: Option<String>,
    pub expression: Rc<TakeAdd>,
}

impl NamedTakeAdd {
    pub fn parse(input: &str) -> IResult<&str, Self> {
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

pub struct NamedList {
    pub expressions: Vec<NamedTakeAdd>,
}

impl NamedList {
    pub fn parse(input: &str) -> IResult<&str, Self> {
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
