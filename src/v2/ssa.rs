use crate::bril;
use crate::v2::context::ContextRef;
use crate::v2::control_flow_graph;
use crate::v2::dominance_tree;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use bril::ConstOps;
use bril::EffectOps;
use bril::Literal;
use bril::Type;
use bril::ValueOps;

#[derive(Debug, Clone)]
pub enum Instruction<'a> {
    Constant {
        op: ConstOps,
        const_type: Type,
        value: Literal,
    },
    Value {
        op: ValueOps,
        op_type: Type,
        args: Vec<InstructionRef<'a>>,
        funcs: Vec<String>,
        labels: Vec<String>,
    },
    Effect {
        op: EffectOps,
        args: Vec<InstructionRef<'a>>,
        funcs: Vec<String>,
        labels: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionRef<'a>(ContextRef<'a, Instruction<'a>>);

impl PartialEq for InstructionRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr() == other.0.ptr()
    }
}

impl Eq for InstructionRef<'_> {}

impl<'a> Deref for InstructionRef<'a> {
    type Target = Instruction<'a>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a> Hash for InstructionRef<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.ptr().hash(state);
    }
}

impl<'a> Instruction<'a> {
    pub fn rename_label(mut self, old: &str, new: &str) -> Self {
        match &mut self {
            Instruction::Value { labels, .. } => {
                for label in labels.iter_mut() {
                    if label == old {
                        *label = new.to_string();
                    }
                }
            }
            Instruction::Effect { labels, .. } => {
                for label in labels.iter_mut() {
                    if label == old {
                        *label = new.to_string();
                    }
                }
            }
            _ => {}
        }

        self
    }

    pub fn rename_arg(mut self, old: InstructionRef<'a>, new: InstructionRef<'a>) -> Self {
        match &mut self {
            Instruction::Value { args, .. } => {
                for arg in args.iter_mut() {
                    if *arg == old {
                        *arg = new;
                    }
                }
            }
            Instruction::Effect { args, .. } => {
                for arg in args.iter_mut() {
                    if *arg == old {
                        *arg = new;
                    }
                }
            }
            _ => {}
        }

        self
    }
}