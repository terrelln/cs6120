use crate::v2::context::{Context, ContextRef};
use crate::v2::error::{CompilerError, CompilerErrorType};
use crate::v2::instruction::{Code, Instruction};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub struct BasicBlocks<'a, I: Instruction> {
    ctx: &'a Context,
    blocks: HashMap<String, ContextRef<'a, BasicBlock<I>>>,
    entry: Option<ContextRef<'a, BasicBlock<I>>>,
    exit: Option<ContextRef<'a, BasicBlock<I>>>,
    label_prefix: String,
    next_label_idx: usize,
}

#[derive(Clone, Debug)]
pub struct BasicBlock<I: Instruction> {
    label: String,
    instrs: Vec<I>,
}

impl<I: Instruction> Hash for BasicBlock<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl<I: Instruction> PartialEq for BasicBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl<I: Instruction> Eq for BasicBlock<I> {}

impl<I: Instruction> PartialOrd for BasicBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.label.cmp(&other.label))
    }
}

impl<I: Instruction> Ord for BasicBlock<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.label.cmp(&other.label)
    }
}

impl<'a, I: 'static + Instruction> BasicBlocks<'a, I> {
    pub fn new(ctx: &'a Context, label_prefix: String) -> Self {
        BasicBlocks {
            ctx,
            blocks: HashMap::new(),
            entry: None,
            exit: None,
            label_prefix,
            next_label_idx: 0,
        }
    }

    pub fn entry(&self) -> Option<ContextRef<'a, BasicBlock<I>>> {
        self.entry
    }

    pub fn exit(&self) -> Option<ContextRef<'a, BasicBlock<I>>> {
        self.exit
    }

    pub fn blocks(&self) -> impl Iterator<Item = ContextRef<'a, BasicBlock<I>>> + '_ {
        self.blocks.values().cloned()
    }

    pub fn get(&self, label: &str) -> Result<ContextRef<'a, BasicBlock<I>>, CompilerError> {
        self.blocks
            .get(label)
            .cloned()
            .ok_or(CompilerErrorType::MissingLabel.with_label(label.to_string()))
    }

    /**
     * Inserts the block into the basic blocks, returning a reference to the block.
     * If the label already exists, returns an error.
     */
    pub fn insert_block(
        &mut self,
        block: BasicBlock<I>,
    ) -> Result<ContextRef<'a, BasicBlock<I>>, CompilerError> {
        let label = block.label.clone();
        let block = self.ctx.create(block);
        if self.blocks.contains_key(&label) {
            return Err(CompilerErrorType::DuplicateLabel.with_label(label));
        }
        self.blocks.insert(label, block);
        Ok(block)
    }

    /**
     * Replaces the block with the same label in the basic blocks, returning a reference to the block.
     * If the label does not exist, returns an error.
     */
    pub fn replace_block(
        &mut self,
        block: BasicBlock<I>,
    ) -> Result<ContextRef<'a, BasicBlock<I>>, CompilerError> {
        let label = block.label.clone();
        let block = self.ctx.create(block);
        if !self.blocks.contains_key(&label) {
            return Err(CompilerErrorType::MissingLabel.with_label(label));
        }
        self.blocks.insert(label, block);
        Ok(block)
    }

    /**
     * Sets the entry block to the block with the given label.
     * If the label is not present in basic blocks, returns an error.
     */
    pub fn set_entry(&mut self, block: ContextRef<'a, BasicBlock<I>>) -> Result<(), CompilerError> {
        if !self.blocks.contains_key(block.label()) {
            return Err(CompilerErrorType::MissingLabel.with_label(block.label().clone()));
        }
        self.entry = Some(block);
        Ok(())
    }

    pub fn set_exit(&mut self, block: ContextRef<'a, BasicBlock<I>>) -> Result<(), CompilerError> {
        if !self.blocks.contains_key(block.label()) {
            return Err(CompilerErrorType::MissingLabel.with_label(block.label().clone()));
        }
        self.exit = Some(block);
        Ok(())
    }

    /**
     * Returns a unique label that is guaranteed not to already exist.
     * If all new labels are created with this function, we also guarantee that
     * no label will ever be created with this as a prefix.
     */
    pub fn create_unique_label(&mut self) -> String {
        let label = format!("{}block{}", self.label_prefix, self.next_label_idx);
        assert!(!self.blocks.contains_key(&label));
        self.next_label_idx += 1;
        label
    }

    pub fn from_code(ctx: &'a Context, code: &[Code<I>]) -> Result<Self, CompilerError> {
        let mut blocks = BasicBlocks::new(ctx, Self::label_prefix(code));

        // Inserts basic blocks & adds entry block for first block
        let insert_block = |blocks: &mut Self, block: BasicBlock<I>| {
            let label = block.label.clone();
            if blocks.entry.is_none() {
                let entry_label = blocks.create_unique_label() + "_entry";
                let entry_block = BasicBlock::new(entry_label, vec![Instruction::jump(label)])?;
                blocks.insert_block(entry_block)?;
            }
            blocks.insert_block(block)
        };

        let mut curr_block: Option<BasicBlock<I>> = None;
        let mut exit_block = BasicBlock::from_label(blocks.create_unique_label() + "_exit");
        exit_block.instrs.push(Instruction::ret());
        for c in code {
            match c {
                Code::Label { label } => {
                    if let Some(mut block) = curr_block {
                        if !block.is_terminated() {
                            // Implicit fallthrough, add explicit terminator
                            block.instrs.push(Instruction::jump(label.clone()));
                        }
                        insert_block(&mut blocks, block)?;
                    }
                    curr_block = Some(BasicBlock::from_label(label.clone()));
                }
                Code::Instruction(instr) => {
                    if curr_block.is_none() {
                        // No label for this basic block, create one
                        curr_block = Some(BasicBlock::from_label(blocks.create_unique_label()));
                    }
                    let instrs = &mut curr_block.as_mut().unwrap().instrs;
                    if instr.is_return() {
                        instrs.push(Instruction::jump(exit_block.label().clone()));
                    } else {
                        instrs.push(instr.clone());
                    }
                    if instr.is_terminator() {
                        insert_block(&mut blocks, curr_block.unwrap())?;
                        curr_block = None;
                    }
                }
            }
        }
        if let Some(mut block) = curr_block {
            assert!(!block.is_terminated());
            block.instrs.push(Instruction::ret());
            insert_block(&mut blocks, block)?;
        }
        let exit_block = insert_block(&mut blocks, exit_block)?;
        blocks.set_exit(exit_block)?;
        Ok(blocks)
    }

    // pub fn to_code(&self) -> Result<Vec<Code>, CompilerError> {
    //     let mut code = Vec::new();
    //     let cfg = ControlFlowGraph::new(self)?;
    //     // TODO: Do we want to try to clean up inserted entry/exit labels?
    //     // TODO: Do we want to try to clean up fallthrough jumps?
    //     // TODO: Do we want to try to clean up inserted unused labels?
    //     // TODO: Do we want to preserve order of original labels?
    //     for block in cfg.pre_order_iter() {
    //         code.extend(block.code());
    //     }
    //     Ok(code)
    // }
}

impl<I: Instruction> BasicBlock<I> {
    pub fn new(label: String, instrs: Vec<I>) -> Result<Self, CompilerError> {
        BasicBlock::validate_instructions(&label, &instrs)?;
        Ok(BasicBlock { label, instrs })
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn non_terminating_instrs(&self) -> &[I] {
        &self.instrs[..self.instrs.len() - 1]
    }

    pub fn instrs(&self) -> &[I] {
        // Including terminator
        &self.instrs
    }

    pub fn terminator(&self) -> &I {
        // We guarantee terminators by construction
        &self.instrs.last().unwrap()
    }

    pub fn uses(&self, var: &I::Arg) -> bool {
        self.instrs
            .iter()
            .flat_map(|instr| instr.args().iter())
            .any(|arg| arg == var)
    }

    pub fn successors(&self) -> &[String] {
        self.terminator().labels()
    }

    // pub fn code(&self) -> impl Iterator<Item = Code> + '_ {
    //     let label = Code::Label {
    //         label: self.label.clone(),
    //     };
    //     let instrs = self
    //         .instrs
    //         .iter()
    //         .map(|inst| Code::Instruction(inst.clone()));
    //     std::iter::once(label).chain(instrs)
    // }
}

impl<I: Instruction> BasicBlock<I> {
    fn validate_instructions(label: &String, instrs: &[I]) -> Result<(), CompilerError> {
        if instrs.is_empty() {
            return Err(CompilerErrorType::BasicBlockEmpty.with_label(label.clone()));
        }
        if !instrs.last().unwrap().is_terminator() {
            return Err(CompilerErrorType::BasicBlockNoTerminator.with_label(label.clone()));
        }
        if instrs[..instrs.len() - 1]
            .iter()
            .any(|instr| instr.is_terminator())
        {
            return Err(CompilerErrorType::BasicBlockMultipleTerminators.with_label(label.clone()));
        }
        Ok(())
    }

    fn from_label(label: String) -> Self {
        BasicBlock {
            label,
            instrs: Vec::new(),
        }
    }

    fn is_terminated(&self) -> bool {
        self.instrs.last().map_or(false, Instruction::is_terminator)
    }
}

impl<'a, I: Instruction> BasicBlocks<'a, I> {
    fn label_prefix(code: &[Code<I>]) -> String {
        let max = code
            .iter()
            .filter_map(|c| match c {
                Code::Label { label } => Some(label),
                _ => None,
            })
            .map(|label| label.chars().filter(|c| c == &'_').count())
            .max()
            .unwrap_or(0);
        std::iter::repeat('_').take(max + 1).collect()
    }
}

#[cfg(test)]
mod tests {}
