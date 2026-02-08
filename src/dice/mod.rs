pub mod display;
pub mod eval;
pub mod parser;

use parser::NamedList;

pub fn handle_dice_string(input: String) -> Result<(), String> {
    let (remaining, list) = NamedList::parse(dice_string.as_ref()).map_err(|err| err.to_string())?;

    for (idx, item) in list.expressions.iter().enumerate() {
        let compiled_expr = item.expression.as_ref().compile();
        if idx > 0 {
            print!(", ");
        }

        match item.name.as_ref() {
            Some(name) => print!("{name}: {compiled_expr} -> {}", compiled_expr.eval()),
            None => print!("Roll {}: {compiled_expr} -> {}", idx + 1, compiled_expr.eval()),
        };
    }

    Ok(())
}
