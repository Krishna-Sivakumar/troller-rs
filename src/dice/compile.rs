use crate::dice::{
    Compile,
    eval::{Roll, RollHand, RollNode},
    parser::*,
};
use rand::distr::{Distribution, Uniform};
use std::rc::Rc;

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
            RollHand::Roll(mut roll) => match &self.filter {
                Some((count, take_higher)) => {
                    roll.rolls.sort_by(|a, b| match take_higher {
                        FilterType::Higher => a.cmp(b).reverse(),
                        FilterType::Lower => a.cmp(b),
                    });
                    roll.limit = Some(*count);
                    RollHand::Roll(roll)
                }
                None => RollHand::Roll(roll),
            },
            RollHand::RollNode(_node) => unreachable!(),
        }
    }
}

impl Compile for &TakeRecursive {
    fn compile(&self) -> RollHand {
        match self {
            TakeRecursive::Take(take) => take.compile(),
            TakeRecursive::TakeAdd(take_add) => take_add.compile(),
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
