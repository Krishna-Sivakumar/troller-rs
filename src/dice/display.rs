use crate::{
    dice::eval::{Op, Roll, RollHand, RollNode},
    dice::parser::*,
};

use std::fmt::Display;

impl Display for OpAdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => f.write_str("+"),
            Self::Minus => f.write_str("-"),
        }
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.die {
            None => f.write_fmt(format_args!("{}", self.count)),
            Some(die) => f.write_fmt(format_args!("{}d{}", self.count, die)),
        }
    }
}

impl Display for Take {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.filter {
            None => f.write_fmt(format_args!("{}", self.dice)),
            Some((count, take_higher)) => f.write_fmt(format_args!(
                "{}{}{}",
                self.dice,
                if take_higher { "h" } else { "l" },
                count
            )),
        }
    }
}

impl Display for TakeFactorRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TakeFactorRight::Take(take) => take.fmt(f),
            TakeFactorRight::TakeFactor(take_factor) => take_factor.fmt(f),
        }
    }
}

impl Display for TakeFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.left))?;
        self.right
            .as_ref()
            .map(|right_expr| f.write_fmt(format_args!(" {} {}", right_expr.0, right_expr.1)));

        Ok(())
    }
}

impl Display for TakeAddRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TakeAddRight::TakeFactor(take_factor) => take_factor.fmt(f),
            TakeAddRight::TakeAdd(take_add) => take_add.fmt(f),
        }
    }
}

impl Display for TakeAdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.left))?;
        self.right
            .as_ref()
            .map(|right_expr| f.write_fmt(format_args!(" {} {}", right_expr.0, right_expr.1)));

        Ok(())
    }
}

impl Display for NamedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, named_expr) in self.expressions.iter().enumerate() {
            if idx > 0 {
                f.write_str(", ")?;
            }
            match named_expr.name.as_ref() {
                None => f.write_fmt(format_args!("{}", named_expr.expression)),
                Some(name) => f.write_fmt(format_args!("{}: {}", name, named_expr.expression)),
            }?;
        }
        Ok(())
    }
}

impl Display for Roll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let limit = self.limit.unwrap_or(self.rolls.len() as u32);

        if self.rolls.len() > 1 {
            f.write_str("[")?;
        }

        for (idx, roll) in self.rolls.iter().enumerate() {
            if idx as u32 == limit {
                f.write_str(" | ")?;
            } else if idx > 0 {
                f.write_str(", ")?;
            }
            match self.die {
                None => {
                    f.write_fmt(format_args!("{}", roll))?;
                }
                Some(die) => {
                    if *roll == 1 || *roll == die {
                        f.write_fmt(format_args!("**{}**", roll))?;
                    } else {
                        f.write_fmt(format_args!("{}", roll))?;
                    }
                }
            }
        }

        if self.rolls.len() > 1 {
            f.write_str("]")?;
        }

        Ok(())
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Plus => f.write_str("+"),
            Op::Minus => f.write_str("-"),
            Op::Multiply => f.write_str("*"),
            Op::Divide => f.write_str("/"),
        }
    }
}

impl Display for RollNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.left.fmt(f)?;
        match self.right.as_ref() {
            None => Ok(()),
            Some((op, roll_hand)) => f.write_fmt(format_args!(" {} {}", op, roll_hand)),
        }?;
        Ok(())
    }
}

impl Display for RollHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RollHand::Roll(roll) => roll.fmt(f),
            RollHand::RollNode(roll_node) => roll_node.fmt(f),
        }
    }
}
