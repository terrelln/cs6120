use std::convert::From;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerErrorType {
    DuplicateLabel,
    MissingLabel,
    BasicBlockEmpty,
    BasicBlockMultipleTerminators,
    BasicBlockNoTerminator,
    ControlFlowNoEntryBlock,
    ControlFlowNoExitBlock,
    ControlFlowEntryBlockHasPredecessors,
    ControlFlowExitBlockHasSuccessors,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerError {
    typ: CompilerErrorType,
    label: Option<String>,
    block: Option<String>,
    line: Option<usize>,
}

impl CompilerError {
    pub fn new(typ: CompilerErrorType) -> Self {
        CompilerError {
            typ,
            label: None,
            block: None,
            line: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_block(mut self, block: String) -> Self {
        self.block = Some(block);
        self
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

impl CompilerErrorType {
    pub fn into_error(self) -> CompilerError {
        self.into()
    }

    pub fn with_label(self, label: String) -> CompilerError {
        CompilerError::new(self).with_label(label)
    }

    pub fn with_block(self, block: String) -> CompilerError {
        CompilerError::new(self).with_block(block)
    }

    pub fn with_line(self, line: usize) -> CompilerError {
        CompilerError::new(self).with_line(line)
    }
}

impl From<CompilerErrorType> for CompilerError {
    fn from(typ: CompilerErrorType) -> Self {
        CompilerError::new(typ)
    }
}
