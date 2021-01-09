use super::{bril, util};
use std::collections::HashMap;

// TODO: Disentangle the CFG data from the BB data

#[derive(Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instrs: Vec<bril::Instruction>,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            label: String::new(),
            instrs: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instrs.is_empty() && self.label.is_empty()
    }
}

impl From<String> for BasicBlock {
    fn from(label: String) -> BasicBlock {
        BasicBlock {
            label: label,
            instrs: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct BasicBlocks {
    pub blocks: Vec<BasicBlock>,
    pub labels: HashMap<String, usize>,
    pub pred: Vec<Vec<usize>>,
    pub succ: Vec<Vec<usize>>,
    prefix: String,
}

impl BasicBlocks {
    pub fn add(&mut self, block: BasicBlock) {
        self.labels.insert(block.label.clone(), self.blocks.len());
        self.blocks.push(block);
    }

    fn set_prefix<'a>(&mut self, labels: impl Iterator<Item=&'a String>) {
        let max = labels
            .map(|label| label.chars().filter(|c| c == &'_' ).count())
            .max()
            .unwrap_or(0);
        self.prefix = std::iter::repeat('_').take(max + 1).collect();
    }

    pub fn create_label(&self) -> String {
        // Labels guaranteed to be unique by self.prefix
        format!("{}block{}", self.prefix, self.blocks.len())
    }

    pub fn add_entry(&mut self, instrs: &Vec<bril::Code>) {
        let entry_label = match instrs.iter().next() {
            Some(bril::Code::Label { label }) => label,
            _ => return,
        };
        let need_entry = instrs.iter().any(|instr| {
            if let bril::Code::Instruction(instr) = instr {
                if let Some(labels) = util::get_labels(instr) {
                    return labels.iter().any(|label| label == entry_label);
                }
            }
            return false;
        });
        if need_entry {
            // let mut block = BasicBlock::from(self.create_label());
            let mut block = BasicBlock::from(self.create_label());
            let jmp = bril::Instruction::Effect {
                op: bril::EffectOps::Jump,
                labels: vec![entry_label.clone()],
                args: Vec::new(),
                funcs: Vec::new(),
            };
            block.instrs.push(jmp);
            self.add(block);
        }
    }
}

pub fn get_labels(instr: &bril::Instruction) -> Option<&Vec<String>> {
    match instr {
        bril::Instruction::Effect { op, labels, .. } => match op {
            bril::EffectOps::Jump => Some(labels),
            bril::EffectOps::Branch => Some(labels),
            bril::EffectOps::Return => Some(labels),
            _ => {
                assert_eq!(labels.len(), 0);
                None
            }
        },
        _ => None,
    }
}

pub fn is_jump(instr: &bril::Instruction) -> bool {
    get_labels(instr).is_some()
}

impl BasicBlocks {
    pub fn from(instrs: &Vec<bril::Code>) -> BasicBlocks {
        let mut blocks = BasicBlocks {
            blocks: Vec::new(),
            labels: HashMap::new(),
            pred: Vec::new(),
            succ: Vec::new(),
            prefix: String::new(),
        };
        blocks.set_prefix(
            instrs.iter().filter_map(|instr| match instr {
                bril::Code::Label{ label } => Some(label),
                _ => None,
            })
        );
        blocks.add_entry(instrs);
        let mut block = BasicBlock::new();
        for instr in instrs {
            match instr {
                bril::Code::Label { label } => {
                    // Labels start a new block
                    if !block.is_empty() {
                        blocks.add(block);
                    }
                    block = BasicBlock::from(label.clone());
                }
                bril::Code::Instruction(instr) => {
                    // Create a new block if needed
                    if block.is_empty() {
                        block = BasicBlock::from(blocks.create_label());
                    }
                    // Add our instruction
                    block.instrs.push(instr.clone());
                    // Finish the block
                    if is_jump(instr) {
                        blocks.add(block);
                        block = BasicBlock::new();
                    }
                }
            }
        }
        if !block.is_empty() {
            blocks.add(block);
        }
        blocks.compute_successors();
        blocks
    }

    fn compute_successors(&mut self) {
        for _ in 0usize..self.blocks.len() {
            self.pred.push(Vec::new());
        }
        for idx in 0..self.blocks.len() {
            let mut succ = Vec::new();
            let fallthrough = match self.blocks[idx].instrs.last() {
                None => true,
                Some(instr) => !is_jump(instr),
            };
            // Add a jump instruction if needed
            if fallthrough {
                if idx + 1 < self.blocks.len() {
                    let label = self.blocks[idx + 1].label.clone();
                    let block = &mut self.blocks[idx];
                    block.instrs.push(bril::Instruction::jump(label));
                } else {
                    let block = &mut self.blocks[idx];
                    block.instrs.push(bril::Instruction::ret());
                }
            }
            for instr in &self.blocks[idx].instrs {
                if let Some(l) = get_labels(instr) {
                    succ.extend(l.iter().map(|label| self.labels[label]));
                }
            }
            for s in &succ {
                self.pred[*s].push(idx);
            }
            self.succ.push(succ);
        }
    }

    pub fn from_blocks(blocks: Vec<BasicBlock>) -> BasicBlocks {
        let mut labels = HashMap::new();
        for (idx, block) in blocks.iter().enumerate() {
            labels.insert(block.label.clone(), idx);
        }
        let mut blocks = BasicBlocks {
            labels,
            blocks,
            pred: Vec::new(),
            succ: Vec::new(),
            prefix: String::new(),
        };
        let labels: Vec<_> = blocks.blocks.iter().map(|block| block.label.clone()).collect();
        blocks.set_prefix(labels.iter());
        blocks.compute_successors();
        blocks
    }

    pub fn to_instrs(self) -> Vec<bril::Code> {
        return to_instrs(self.blocks);
    }
}

pub fn to_instrs(blocks: Vec<BasicBlock>) -> Vec<bril::Code> {
    let mut instrs = Vec::new();
    for block in blocks {
        instrs.push(bril::Code::Label { label: block.label });
        instrs.extend(
            block
                .instrs
                .iter()
                .map(|instr| bril::Code::Instruction(instr.clone())),
        );
    }
    instrs
}
