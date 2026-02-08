use crate::dice::parser::*;
use rand::distr::{Distribution, Uniform};
use std::rc::Rc;

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

/// represents a combination of roll nodes with a binary operator.
pub struct RollNode {
    /// left hand of node.
    pub left: Rc<RollHand>,
    /// right hand of node, if any.
    pub right: Option<(Op, Rc<RollHand>)>,
}

impl Compile for &Dice {
    fn compile(&self) -> RollHand {
        RollHand::Roll(match self.die {
            None => Roll {
                rolls: vec![self.count],
                limit: None,
                die: None,
            },
            Some(die) => {
                let mut rolls = Vec::new();
                let mut rng = rand::rng();
                let between =
                    Uniform::try_from(1..die + 1).expect("Could not create random distribution.");
                for _ in 0..self.count {
                    rolls.push(between.sample(&mut rng));
                }
                Roll {
                    rolls,
                    limit: None,
                    die: Some(die),
                }
            }
        })
    }
}

impl Compile for &Take {
    fn compile(&self) -> RollHand {
        match self.dice.as_ref().compile() {
            RollHand::Roll(mut roll) => match self.filter {
                Some((count, take_higher)) => {
                    roll.rolls.sort_by(|a, b| {
                        if take_higher {
                            a.cmp(b).reverse()
                        } else {
                            a.cmp(b)
                        }
                    });
                    roll.limit = Some(count);
                    RollHand::Roll(roll)
                }
                None => RollHand::Roll(roll),
            },
            RollHand::RollNode(_node) => unreachable!(),
        }
    }
}

impl Compile for &TakeFactor {
    fn compile(&self) -> RollHand {
        let left = self.left.as_ref().compile();
        match self.right.as_ref() {
            Some((op, take_factor)) => match take_factor.as_ref() {
                TakeFactorRight::TakeFactor(take_factor) => RollHand::RollNode(RollNode {
                    left: Rc::new(left),
                    right: Some((op.into(), Rc::new(take_factor.compile()))),
                }),
                TakeFactorRight::Take(take) => RollHand::RollNode(RollNode {
                    left: Rc::new(left),
                    right: Some((op.into(), Rc::new(take.compile()))),
                }),
            },
            None => left,
        }
    }
}

impl Compile for &TakeAdd {
    fn compile(&self) -> RollHand {
        let left = self.left.as_ref().compile();
        match self.right.as_ref() {
            Some((op, take_add)) => match take_add.as_ref() {
                TakeAddRight::TakeFactor(take_factor) => RollHand::RollNode(RollNode {
                    left: Rc::new(left),
                    right: Some((op.into(), Rc::new(take_factor.compile()))),
                }),
                TakeAddRight::TakeAdd(take_add) => RollHand::RollNode(RollNode {
                    left: Rc::new(left),
                    right: Some((op.into(), Rc::new(take_add.compile()))),
                }),
            },
            None => left,
        }
    }
}
