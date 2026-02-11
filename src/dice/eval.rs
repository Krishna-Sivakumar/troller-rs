use crate::dice::{Eval, parser::*};
use std::rc::Rc;

pub enum Op {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl From<&OpFactor> for Op {
    fn from(value: &OpFactor) -> Self {
        match value {
            OpFactor::Multiply => Op::Multiply,
            OpFactor::Divide => Op::Divide,
        }
    }
}

impl From<&OpAdd> for Op {
    fn from(value: &OpAdd) -> Self {
        match value {
            OpAdd::Plus => Op::Plus,
            OpAdd::Minus => Op::Minus,
        }
    }
}

/// Represents the node types for a compiled Roll AST
pub enum RollHand {
    /// one or more rolls
    Roll(Roll),
    /// combines rolls with a binary operator
    RollNode(RollNode),
}

/// Represents a set of rolled dice
pub struct Roll {
    /// individual rolls
    pub rolls: Vec<u32>,
    /// number of dice to take from rolls
    pub limit: Option<u32>,
    /// size of die rolled
    pub die: Option<u32>,
}

/// represents a combination of roll nodes with a binary operator.
pub struct RollNode {
    /// left hand of node.
    pub left: Rc<RollHand>,
    /// right hand of node, if any.
    pub right: Option<(Op, Rc<RollHand>)>,
}

impl Eval for Roll {
    fn eval(&self) -> u32 {
        let mut total = 0u32;
        match self.limit {
            None => {
                for i in self.rolls.iter() {
                    total += i;
                }
            }
            Some(limit) => {
                for i in self.rolls.iter().take(limit as usize) {
                    total += i;
                }
            }
        };
        total
    }
}

impl Eval for RollNode {
    fn eval(&self) -> u32 {
        let left_eval = match self.left.as_ref() {
            RollHand::Roll(roll) => roll.eval(),
            RollHand::RollNode(roll_node) => roll_node.eval(),
        };

        match self.right.as_ref() {
            None => return left_eval,
            Some((op, right)) => {
                let right_eval = match right.as_ref() {
                    RollHand::Roll(roll) => roll.eval(),
                    RollHand::RollNode(roll_node) => roll_node.eval(),
                };

                match op {
                    Op::Plus => left_eval + right_eval,
                    Op::Minus => left_eval - right_eval,
                    Op::Multiply => left_eval * right_eval,
                    Op::Divide => left_eval / right_eval,
                }
            }
        }
    }
}

impl Eval for RollHand {
    fn eval(&self) -> u32 {
        match self {
            RollHand::Roll(roll) => roll.eval(),
            RollHand::RollNode(roll_node) => roll_node.eval(),
        }
    }
}
