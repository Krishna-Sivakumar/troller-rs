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

pub enum OpFactor {
    Multiply,
    Divide,
}

pub struct Dice {
    pub count: u32,
    pub die: Option<u32>,
}

pub enum FilterType {
    Higher,
    Lower,
}

pub struct Take {
    pub dice: Rc<Dice>,
    pub filter: Option<(u32, FilterType)>,
}

pub enum TakeFactorRight {
    TakeFactor(TakeFactor),
    Take(Take),
}

pub struct TakeFactor {
    pub left: Rc<Take>,
    pub right: Option<(OpFactor, Rc<TakeFactorRight>)>,
}

pub enum TakeAddRight {
    TakeFactor(TakeFactor),
    TakeAdd(TakeAdd),
}

pub struct TakeAdd {
    pub left: Rc<TakeFactor>,
    pub right: Option<(OpAdd, Rc<TakeAddRight>)>,
}

pub struct NamedTakeAdd {
    pub name: Option<String>,
    pub expression: Rc<TakeAdd>,
}

pub struct NamedList {
    pub expressions: Vec<NamedTakeAdd>,
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

impl Take {
    pub fn parse(input: &str) -> IResult<&str, Self> {
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

impl TakeFactor {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, left_expr) = Take::parse(input)?;
        let (input, right_option) = opt((
            space0,
            OpFactor::parse,
            space0,
            alt((
                TakeFactor::parse.map(|tf| TakeFactorRight::TakeFactor(tf)),
                Take::parse.map(|take| TakeFactorRight::Take(take)),
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

impl TakeAdd {
    pub fn parse(input: &str) -> IResult<&str, Self> {
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
