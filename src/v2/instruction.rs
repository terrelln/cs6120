use crate::bril;
use crate::v2::context::ContextRef;

use std::fmt::Debug;

use bril::ConstOps;
use bril::EffectOps;
use bril::Literal;
use bril::Type;
use bril::ValueOps;

#[derive(Debug, Clone)]
enum InstructionBase<Dest, Arg> {
    Constant {
        op: ConstOps,
        dest: Dest,
        const_type: Type,
        value: Literal,
    },
    Value {
        op: ValueOps,
        dest: Dest,
        op_type: Type,
        args: Vec<Arg>,
        funcs: Vec<String>,
        labels: Vec<String>,
    },
    Effect {
        op: EffectOps,
        args: Vec<Arg>,
        funcs: Vec<String>,
        labels: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct BrilInstruction {
    base: InstructionBase<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct SSAInstRef<'a>(ContextRef<'a, SSAInstruction<'a>>);

#[derive(Debug, Clone)]
pub struct SSAInstruction<'a> {
    base: InstructionBase<(), SSAInstRef<'a>>,
}

impl PartialEq for SSAInstRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr() == other.0.ptr()
    }
}

impl Eq for SSAInstRef<'_> {}

impl From<InstructionBase<String, String>> for BrilInstruction {
    fn from(base: InstructionBase<String, String>) -> Self {
        BrilInstruction { base }
    }
}

impl<'a> From<InstructionBase<(), SSAInstRef<'a>>> for SSAInstruction<'a> {
    fn from(base: InstructionBase<(), SSAInstRef<'a>>) -> Self {
        SSAInstruction { base }
    }
}

impl From<bril::Instruction> for BrilInstruction {
    fn from(instr: bril::Instruction) -> Self {
        match instr {
            bril::Instruction::Constant {
                op: ConstOps::Const,
                const_type,
                dest,
                value,
            } => Self::constant(const_type, dest, value),
            bril::Instruction::Value {
                op,
                op_type,
                dest,
                args,
                funcs,
                labels,
            } => Self::value(op, op_type, dest, args, funcs, labels),
            bril::Instruction::Effect {
                op,
                args,
                funcs,
                labels,
            } => Self::effect(op, args, funcs, labels),
        }
    }
}

impl Into<bril::Instruction> for BrilInstruction {
    fn into(self) -> bril::Instruction {
        match self.base {
            InstructionBase::Constant {
                op,
                const_type,
                dest,
                value,
            } => bril::Instruction::Constant {
                op,
                const_type,
                dest,
                value,
            },
            InstructionBase::Value {
                op,
                op_type,
                dest,
                args,
                funcs,
                labels,
            } => bril::Instruction::Value {
                op,
                op_type,
                dest,
                args,
                funcs,
                labels,
            },
            InstructionBase::Effect {
                op,
                args,
                funcs,
                labels,
            } => bril::Instruction::Effect {
                op,
                args,
                funcs,
                labels,
            },
        }
    }
}

pub trait Instruction
where
    Self: From<InstructionBase<Self::Dest, Self::Arg>>,
    Self: Debug + Clone,
    Self::Dest: PartialEq + Eq,
    Self::Arg: PartialEq + Eq,
{
    type Dest;
    type Arg;

    fn base(&self) -> &InstructionBase<Self::Dest, Self::Arg>;

    fn mut_base(&mut self) -> &mut InstructionBase<Self::Dest, Self::Arg>;

    fn into_base(self) -> InstructionBase<Self::Dest, Self::Arg>;

    fn dest(&self) -> Option<&Self::Dest> {
        match self.base() {
            InstructionBase::Constant { dest, .. } => Some(dest),
            InstructionBase::Value { dest, .. } => Some(dest),
            InstructionBase::Effect { .. } => None,
        }
    }

    fn args(&self) -> &[Self::Arg] {
        match self.base() {
            InstructionBase::Constant { .. } => &[],
            InstructionBase::Value { args, .. } => args,
            InstructionBase::Effect { args, .. } => args,
        }
    }

    fn labels(&self) -> &[String] {
        match self.base() {
            InstructionBase::Constant { .. } => &[],
            InstructionBase::Value { labels, .. } => labels,
            InstructionBase::Effect { labels, .. } => labels,
        }
    }

    fn constant(t: Type, dest: Self::Dest, value: Literal) -> Self {
        let base = InstructionBase::Constant {
            op: ConstOps::Const,
            const_type: t,
            dest,
            value,
        };
        base.into()
    }

    fn effect(
        op: EffectOps,
        args: Vec<Self::Arg>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) -> Self {
        let base = InstructionBase::Effect {
            op,
            args,
            funcs,
            labels,
        };
        base.into()
    }

    fn value(
        op: ValueOps,
        op_type: Type,
        dest: Self::Dest,
        args: Vec<Self::Arg>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) -> Self {
        let base = InstructionBase::Value {
            op,
            op_type,
            dest,
            args,
            funcs,
            labels,
        };
        base.into()
    }

    fn jump(label: String) -> Self {
        Self::effect(EffectOps::Jump, Vec::new(), Vec::new(), vec![label])
    }

    fn ret() -> Self {
        Self::effect(EffectOps::Return, Vec::new(), Vec::new(), Vec::new())
    }

    fn id(t: Type, dest: Self::Dest, arg: Self::Arg) -> Self {
        Self::value(ValueOps::Id, t, dest, vec![arg], Vec::new(), Vec::new())
    }

    fn const_int(dest: Self::Dest, value: i64) -> Self {
        Self::constant(Type::Int, dest, Literal::Int(value))
    }

    fn const_bool(dest: Self::Dest, value: bool) -> Self {
        Self::constant(Type::Bool, dest, Literal::Bool(value))
    }

    fn alloc(dest: Self::Dest, size: Self::Arg, ptr_type: Type) -> Self {
        let var_type = Type::Pointer(Box::new(ptr_type));
        let base = InstructionBase::Value {
            op: ValueOps::Alloc,
            op_type: var_type,
            dest,
            args: vec![size],
            funcs: Vec::new(),
            labels: Vec::new(),
        };
        base.into()
    }

    fn is_terminator(&self) -> bool {
        match self.base() {
            InstructionBase::Effect { op, .. } => match op {
                EffectOps::Jump | EffectOps::Branch | EffectOps::Return => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn is_return(&self) -> bool {
        match self.base() {
            InstructionBase::Effect {
                op: EffectOps::Return,
                ..
            } => true,
            _ => false,
        }
    }

    fn rename_label(mut self, old: &str, new: &str) -> Self {
        match self.mut_base() {
            InstructionBase::Effect { labels, .. } => {
                for label in labels.iter_mut() {
                    if label == old {
                        *label = new.to_string();
                    }
                }
            }
            InstructionBase::Value { labels, .. } => {
                for label in labels.iter_mut() {
                    if label == old {
                        *label = new.to_string();
                    }
                }
            }
            _ => {}
        }
        self.into_base().into()
    }

    fn rename_arg(mut self, old: &Self::Arg, new: &Self::Arg) -> Self
    where
        Self::Arg: Clone + Eq,
    {
        match self.mut_base() {
            InstructionBase::Value { args, .. } => {
                for arg in args.iter_mut() {
                    if &arg == &old {
                        *arg = new.clone();
                    }
                }
            }
            InstructionBase::Effect { args, .. } => {
                for arg in args.iter_mut() {
                    if &arg == &old {
                        *arg = new.clone();
                    }
                }
            }
            _ => {}
        }
        self.into_base().into()
    }

    fn rename_dest(mut self, old: &Self::Dest, new: &Self::Dest) -> Self
    where
        Self::Dest: Clone + Eq,
    {
        match self.mut_base() {
            InstructionBase::Value { dest, .. } => {
                if &dest == &old {
                    *dest = new.clone();
                }
            }
            InstructionBase::Constant { dest, .. } => {
                if &dest == &old {
                    *dest = new.clone();
                }
            }
            _ => {}
        }
        self.into_base().into()
    }
}

impl Instruction for BrilInstruction {
    type Dest = String;
    type Arg = String;

    fn base(&self) -> &InstructionBase<Self::Dest, Self::Arg> {
        &self.base
    }

    fn mut_base(&mut self) -> &mut InstructionBase<Self::Dest, Self::Arg> {
        &mut self.base
    }

    fn into_base(self) -> InstructionBase<Self::Dest, Self::Arg> {
        self.base
    }
}

impl<'a> Instruction for SSAInstruction<'a> {
    type Dest = ();
    type Arg = SSAInstRef<'a>;

    fn base(&self) -> &InstructionBase<Self::Dest, Self::Arg> {
        &self.base
    }

    fn mut_base(&mut self) -> &mut InstructionBase<Self::Dest, Self::Arg> {
        &mut self.base
    }

    fn into_base(self) -> InstructionBase<Self::Dest, Self::Arg> {
        self.base
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Code<I: Instruction> {
    Label { label: String },
    Instruction(I),
}

impl<I: Instruction + From<bril::Instruction>> From<bril::Code> for Code<I> {
    fn from(code: bril::Code) -> Self {
        match code {
            bril::Code::Label { label } => Code::Label { label },
            bril::Code::Instruction(instr) => Code::Instruction(instr.into()),
        }
    }
}

impl<I: Instruction + Into<bril::Instruction>> Into<bril::Code> for Code<I> {
    fn into(self) -> bril::Code {
        match self {
            Code::Label { label } => bril::Code::Label { label },
            Code::Instruction(instr) => bril::Code::Instruction(instr.into()),
        }
    }
}
