use super::bril;
use std::collections::HashMap;

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

pub struct BasicBlocks {
    pub blocks: Vec<BasicBlock>,
    pub labels: HashMap<String, usize>,
    pub pred: Vec<Vec<usize>>,
    pub succ: Vec<Vec<usize>>,
}

impl BasicBlocks {
    pub fn add(&mut self, block: BasicBlock) {
        self.labels.insert(block.label.clone(), self.blocks.len());
        self.blocks.push(block);
    }

    pub fn create_label(&self) -> String {
        format!("__block{}", self.blocks.len())
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
            },
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
        };
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
        for (idx, block) in self.blocks.iter().enumerate() {
            let mut succ = Vec::new();
            for instr in &block.instrs {
                if let Some(labels) = get_labels(instr) {
                    succ.extend(labels.iter().map(|label| self.labels[label]));
                }
            }
            let fallthrough = match block.instrs.last() {
                None => true,
                Some(instr) => !is_jump(instr)
            };
            if fallthrough && idx + 1 < self.blocks.len() {
                succ.push(idx + 1);
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
        };
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