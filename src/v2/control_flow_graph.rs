use crate::v2::basic_block::{BasicBlock, BasicBlocks};
use crate::v2::context::ContextRef;
use crate::v2::error::{CompilerError, CompilerErrorType};
use crate::v2::graph::Graph;
use crate::v2::instruction::Instruction;
use std::collections::HashMap;

pub struct ControlFlowGraph<'a, 'b, I: Instruction> {
    entry: ContextRef<'b, BasicBlock<I>>,
    exit: ContextRef<'b, BasicBlock<I>>,
    pred: HashMap<String, Vec<ContextRef<'b, BasicBlock<I>>>>,
    succ: HashMap<String, Vec<ContextRef<'b, BasicBlock<I>>>>,
    // Pro: Holding onto the reference guarantees the CFG remains correct, because the blocks are immutable
    // Con: The CFG's lifetime is more complicated
    blocks: &'a BasicBlocks<'b, I>,
}

impl<'a, 'b, I: 'static + Instruction> ControlFlowGraph<'a, 'b, I> {
    pub fn new(blocks: &'a BasicBlocks<'b, I>) -> Result<Self, CompilerError> {
        let succ: HashMap<_, Vec<_>> = blocks
            .blocks()
            .map(|block| {
                let label = block.label().clone();
                let succ = block
                    .successors()
                    .iter()
                    .map(|label| blocks.get(label))
                    .collect::<Result<_, _>>()?;
                Ok((label, succ))
            })
            .collect::<Result<_, CompilerError>>()?;
        let mut pred = HashMap::new();
        for block in blocks.blocks() {
            for succ in block.successors() {
                pred.entry(succ.clone())
                    .or_insert_with(Vec::new)
                    .push(block);
            }
        }

        // Validate entry / exit blocks are present and have no
        // predecessors / successors respectively
        if blocks.entry().is_none() {
            return Err(CompilerErrorType::ControlFlowNoEntryBlock.into());
        }
        if blocks.exit().is_none() {
            return Err(CompilerErrorType::ControlFlowNoExitBlock.into());
        }
        let entry = blocks.entry().unwrap();
        let exit = blocks.exit().unwrap();
        assert!(blocks.get(entry.label()).is_ok());
        assert!(blocks.get(exit.label()).is_ok());

        if pred.get(entry.label()).unwrap().len() > 0 {
            return Err(CompilerErrorType::ControlFlowEntryBlockHasPredecessors
                .with_label(entry.label().clone()));
        }
        if succ.get(exit.label()).unwrap().len() > 0 {
            return Err(CompilerErrorType::ControlFlowExitBlockHasSuccessors
                .with_label(exit.label().clone()));
        }

        let cfg = ControlFlowGraph {
            entry,
            exit,
            pred,
            succ,
            blocks,
        };
        Ok(cfg)
    }

    pub fn blocks(&self) -> impl Iterator<Item = ContextRef<'b, BasicBlock<I>>> + '_ {
        self.blocks.blocks()
    }

    pub fn entry(&self) -> ContextRef<'b, BasicBlock<I>> {
        self.entry
    }

    pub fn exit(&self) -> ContextRef<'b, BasicBlock<I>> {
        self.entry
    }

    pub fn predecessors(
        &self,
        block: ContextRef<'b, BasicBlock<I>>,
    ) -> &[ContextRef<'b, BasicBlock<I>>] {
        self.pred
            .get(block.label())
            .map(|pred| pred.as_slice())
            .unwrap_or(&[])
    }

    pub fn successors(
        &self,
        block: ContextRef<'b, BasicBlock<I>>,
    ) -> &[ContextRef<'b, BasicBlock<I>>] {
        self.succ
            .get(block.label())
            .map(|pred| pred.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::context::Context;
    use crate::v2::instruction::BrilInstruction;

    #[test]
    fn test_basic() {
        let ctx = Context::new();
        let bbs = BasicBlocks::<BrilInstruction>::new(&ctx, "_".to_string());
        let _cfg = ControlFlowGraph::new(&bbs);
    }
}

impl<'a, 'b, I: 'static + Instruction> Graph for ControlFlowGraph<'a, 'b, I> {
    type Node = ContextRef<'b, BasicBlock<I>>;

    fn entry_node(&self) -> Option<Self::Node> {
        Some(self.entry)
    }

    fn exit_node(&self) -> Option<Self::Node> {
        Some(self.exit)
    }

    fn nodes(&self) -> impl Iterator<Item = ContextRef<'b, BasicBlock<I>>> + '_ {
        self.blocks.blocks()
    }

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.pred.get(node.label()).unwrap().iter().cloned()
    }

    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.succ.get(node.label()).unwrap().iter().cloned()
    }
}
