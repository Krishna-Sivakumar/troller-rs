pub mod compile;
pub mod display;
pub mod eval;
pub mod parser;

use nom::IResult;
use parser::{NamedList, TakeAdd};
use std::{ops::Range, time::Instant};

use crate::dice::eval::RollHand;

/// This trait is implemented by structs that compile to a Roll AST.
pub trait Compile {
    /// Returns a compiled Roll AST node.
    fn compile(&self) -> RollHand;
}

/// This trait is implemented by trees that can return a summed up roll.
pub trait Eval {
    /// Returns the summed roll of a node.
    fn eval(&self) -> u32;
}

/// This trait is implemented by structs that can parse a version of themselves out from a string.
pub trait Parse<NodeType = Self> {
    /// Returns a result of the remaining `input` and the parsed struct `Self` if it can be parsed from `input`.
    fn parse(input: &str) -> IResult<&str, NodeType>;
}

pub struct RollResult {
    pub name: String,
    pub value: String,
}

pub fn handle_dice_string(
    dice_string: String,
) -> Result<Vec<RollResult>, Box<dyn std::error::Error>> {
    let (_remaining, list) =
        NamedList::parse(dice_string.as_ref()).map_err(|err| err.to_string())?;

    let mut out = String::new();
    let mut roll_results = Vec::new();

    for (idx, item) in list.expressions.iter().enumerate() {
        let compiled_expr = item.expression.as_ref().compile();
        if idx > 0 {
            out += ", ";
        }

        match item.name.as_ref() {
            Some(name) => {
                roll_results.push(RollResult {
                    name: name.clone(),
                    value: format!("{compiled_expr} => {}", compiled_expr.eval()),
                });
            }
            None => {
                roll_results.push(RollResult {
                    name: format!("Roll {}", idx + 1),
                    value: format!("{compiled_expr} => {}", compiled_expr.eval()),
                });
            }
        };
    }

    Ok(roll_results)
}

#[allow(dead_code)]
fn test_roll_performance(
    unnamed_expression: &'static str,
    range: Range<i32>,
) -> Result<(Vec<u32>, u128), Box<dyn std::error::Error>> {
    let (_input, parsed_expression) = TakeAdd::parse(unnamed_expression)?;

    let compiled_node = (&parsed_expression).compile();

    let mut rolls: Vec<u32> = Vec::with_capacity(range.clone().count());
    let start = Instant::now();

    for _ in range.clone() {
        rolls.push(compiled_node.eval());
    }

    let time_taken_ms = start.elapsed().as_millis();

    Ok((rolls, time_taken_ms))
}
#[test]
fn test_roll_performance_simple() -> Result<(), Box<dyn std::error::Error>> {
    let expression = "4d6 + 3";
    let (rolls, time_taken_ms) = test_roll_performance(expression, 0..1_00_000_000)?;
    println!(
        "took {}ms to evaluate {} rolls on {}",
        time_taken_ms,
        rolls.len(),
        expression
    );
    Ok(())
}

#[test]
fn test_roll_performance_take_higher() -> Result<(), Box<dyn std::error::Error>> {
    let expression = "3d6h1 + 9 + 2";
    let (rolls, time_taken_ms) = test_roll_performance(expression, 0..1_00_000_000)?;
    println!(
        "took {}ms to evaluate {} rolls on {}",
        time_taken_ms,
        rolls.len(),
        expression
    );
    Ok(())
}
