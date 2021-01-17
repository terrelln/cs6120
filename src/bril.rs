use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::cmp::Ordering;
use ordered_float::OrderedFloat;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<Argument>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<Type>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instrs: Vec<Code>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Code {
    Label { label: String },
    Instruction(Instruction),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Instruction {
    Constant {
        op: ConstOps,
        dest: String,
        #[serde(rename = "type")]
        const_type: Type,
        value: Literal,
    },
    Value {
        op: ValueOps,
        dest: String,
        #[serde(rename = "type")]
        op_type: Type,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
    },
    Effect {
        op: EffectOps,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
    },
}

impl Instruction {
    pub fn jump(label: String) -> Instruction {
        Instruction::Effect {
            op: EffectOps::Jump,
            args: Vec::new(),
            funcs: Vec::new(),
            labels: vec![label],
        }
    }
    
    pub fn ret() -> Instruction {
        Instruction::Effect {
            op: EffectOps::Return,
            args: Vec::new(),
            funcs: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn id(t: Type, dest: String, arg: String) -> Instruction {
        Instruction::Value {
            op: ValueOps::Id,
            op_type: t,
            dest: dest,
            args: vec![arg],
            funcs: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn constant(t: Type, dest: String, value: Literal) -> Instruction {
        Instruction::Constant {
            op: ConstOps::Const,
            const_type: t,
            dest: dest,
            value: value,
        }
    }

    pub fn const_int(dest: String, value: i64) -> Instruction {
        Self::constant(Type::Int, dest, Literal::Int(value))
    }

    pub fn const_bool(dest: String, value: bool) -> Instruction {
        Self::constant(Type::Bool, dest, Literal::Bool(value))
    }

    pub fn alloc(dest: String, size: String, ptr_type: Type) -> Instruction {
        let var_type = Type::Pointer(Box::new(ptr_type));
        Instruction::Value {
            op: ValueOps::Alloc,
            op_type: var_type,
            dest: dest,
            args: vec![size],
            funcs: Vec::new(),
            labels: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ConstOps {
    #[serde(rename = "const")]
    Const,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EffectOps {
    #[serde(rename = "jmp")]
    Jump,
    #[serde(rename = "br")]
    Branch,
    Call,
    #[serde(rename = "ret")]
    Return,
    Print,
    Nop,
    #[cfg(feature = "memory")]
    Store,
    #[cfg(feature = "memory")]
    Free,
    #[cfg(feature = "speculate")]
    Speculate,
    #[cfg(feature = "speculate")]
    Commit,
    #[cfg(feature = "speculate")]
    Guard,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ValueOps {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
    Not,
    And,
    Or,
    Call,
    Id,
    #[cfg(feature = "ssa")]
    Phi,
    #[cfg(feature = "float")]
    Fadd,
    #[cfg(feature = "float")]
    Fsub,
    #[cfg(feature = "float")]
    Fmul,
    #[cfg(feature = "float")]
    Fdiv,
    #[cfg(feature = "float")]
    Feq,
    #[cfg(feature = "float")]
    Flt,
    #[cfg(feature = "float")]
    Fgt,
    #[cfg(feature = "float")]
    Fle,
    #[cfg(feature = "float")]
    Fge,
    #[cfg(feature = "memory")]
    Alloc,
    #[cfg(feature = "memory")]
    Load,
    #[cfg(feature = "memory")]
    PtrAdd,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
    #[cfg(feature = "float")]
    Float,
    #[cfg(feature = "memory")]
    #[serde(rename = "ptr")]
    Pointer(Box<Type>),
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (x, y) if x == y => Ordering::Equal,
            (Type::Bool, _) => Ordering::Less,
            (_, Type::Bool) => Ordering::Greater,
            (Type::Int, _) => Ordering::Less,
            (_, Type::Int) => Ordering::Greater,
            (Type::Float, _) => Ordering::Less,
            (_, Type::Float) => Ordering::Greater,
            (Type::Pointer(x), Type::Pointer(y)) => x.cmp(y),
        }
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    #[cfg(feature = "float")]
    Float(OrderedFloat<f64>),
}

// impl PartialEq for Literal {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Literal::Bool(x), Literal::Bool(y)) => x == y,
//             (Literal::Int(x), Literal::Int(y)) => x == y,
//             (Literal::Float(x), Literal::Float(y)) => {
//                 if x.is_nan() && y.is_nan() {
//                     true
//                 } else {
//                     x == y
//                 }
//             }
//             _ => false,
//         }
//     }
// }

// impl Eq for Literal {}

// impl Hash for Literal {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         match self {
//             Literal::Bool(b) => {
//                 0.hash(state);
//             }
//         }
//     }
// }

pub fn load_program() -> Program {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    serde_json::from_str(&buffer).unwrap()
}

pub fn output_program(p: &Program) {
    io::stdout()
        .write_all(serde_json::to_string(p).unwrap().as_bytes())
        .unwrap();
}
