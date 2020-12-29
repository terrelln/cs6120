use super::bb;
use super::bril;
use super::data_flow::{data_flow, DataFlowAlgorithm, DataFlowDirection};
// use super::lvn::LVN;
use super::util;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Copy, Clone)]
pub struct Loc {
    pub block: usize,
    pub instr: usize,
}

struct ReachingDefinitionsAlgorithm {}

impl DataFlowAlgorithm for ReachingDefinitionsAlgorithm {
    type Result = HashMap<String, HashSet<Loc>>;

    fn direction(&self) -> DataFlowDirection {
        DataFlowDirection::Forward
    }

    fn init(&self) -> Self::Result {
        HashMap::new()
    }

    fn transfer(
        &self,
        block_id: usize,
        block: &bb::BasicBlock,
        input: &Self::Result,
    ) -> Self::Result {
        let mut killed = HashMap::new();
        let mut defined = HashMap::new();
        for (idx, instr) in block.instrs.iter().enumerate().rev() {
            if let Some(dest) = util::get_dest(instr) {
                let loc = Loc {
                    block: block_id,
                    instr: idx,
                };
                killed.insert(dest.clone(), loc);
                defined.insert(dest.clone(), loc);
            }
        }
        let mut output = input.clone();
        for var in killed.keys() {
            output.remove(var);
        }
        for (var, loc) in defined {
            output.insert(var, vec![loc].into_iter().collect::<HashSet<Loc>>());
        }
        output
    }

    fn merge<'a>(&self, input: impl Iterator<Item = &'a Self::Result>) -> Self::Result {
        input.fold(HashMap::new(), |mut merged, input| {
            for (var, locs) in input {
                merged
                    .entry(var.clone())
                    .or_insert(HashSet::new())
                    .extend(locs.iter());
            }
            merged
        })
    }
}

pub fn reaching_defs(blocks: &bb::BasicBlocks) -> Vec<HashMap<String, HashSet<Loc>>> {
    let algo = ReachingDefinitionsAlgorithm {};
    data_flow(algo, blocks).0
}

struct ConstantPropagationAlgorithm {
    // function: &'a bril::Function,
}

impl DataFlowAlgorithm for ConstantPropagationAlgorithm {
    type Result = Option<HashMap<String, bril::Literal>>;

    fn direction(&self) -> DataFlowDirection {
        DataFlowDirection::Forward
    }

    fn init(&self) -> Self::Result {
        None
    }

    fn transfer(
        &self,
        _block_id: usize,
        block: &bb::BasicBlock,
        input: &Self::Result,
    ) -> Self::Result {
        let mut constants = input.clone().unwrap_or(HashMap::new());
        for instr in &block.instrs {
            match instr.clone() {
                bril::Instruction::Constant { dest, value, .. } => {
                    constants.insert(dest, value);
                }
                bril::Instruction::Value { dest, op, args, .. } => {
                    let const_args: Vec<bril::Literal> = args
                        .iter()
                        .filter_map(|arg| constants.get(arg))
                        .cloned()
                        .collect();
                    if op != bril::ValueOps::Call && const_args.len() == args.len() {
                        constants.insert(dest, util::evaluate(&op, &const_args));
                    } else {
                        constants.remove(&dest);
                    }
                }
                _ => {}
            }
        }
        Some(constants)

        // // Only insert constants that were already present, not newly created variables
        // let vars: Vec<String> = block
        //     .instrs
        //     .iter()
        //     .filter_map(|instr| util::get_dest(instr))
        //     .chain(input.keys())
        //     .cloned()
        //     .collect();
        // let instrs: Vec<bril::Instruction> = input
        //     .into_iter()
        //     .map(|(var, lit)| util::constant(var, lit))
        //     .chain(block.instrs.iter().cloned())
        //     .collect();
        // let block = bb::BasicBlock {
        //     label: block.label.clone(),
        //     instrs,
        // };
        // let mut lvn = LVN::new();
        // let block = lvn.process(&self.function, &block);
        // let mut output = HashMap::new();
        // for instr in block.instrs {
        //     match instr {
        //         bril::Instruction::Constant { dest, value, .. } => {
        //             if vars.contains(&dest) {
        //                 output.insert(dest, value);
        //             }
        //         }
        //         bril::Instruction::Value { dest, .. } => {
        //             // Non-constant kills the constant
        //             output.remove(&dest);
        //         }
        //         _ => {}
        //     }
        // }
        // Some(output)
    }

    fn merge<'a>(&self, input: impl Iterator<Item = &'a Self::Result>) -> Self::Result {
        let mut input = input.filter_map(|x| match x {
            None => None,
            Some(x) => Some(x),
        });
        let first = input.next();
        match first {
            None => None,
            Some(first) => Some(input.fold(first.clone(), |mut merged, input| {
                merged.retain(|var, lit| Some(&*lit) == input.get(var));
                merged
            })),
        }
    }
}

pub fn constant_propagation(function: bril::Function) -> Vec<HashMap<String, bril::Literal>> {
    let algo = ConstantPropagationAlgorithm {
        // function: &function,
    };
    let blocks = bb::BasicBlocks::from(&function.instrs);
    let const_prop = data_flow(algo, &blocks).0;
    const_prop
        .into_iter()
        .map(|x| x.unwrap_or(HashMap::new()))
        .collect()
}
