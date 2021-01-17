use super::bril;
use ordered_float::OrderedFloat;

pub fn is_effect(instr: &bril::Instruction) -> bool {
    match instr {
        bril::Instruction::Effect { .. } => true,
        _ => false,
    }
}

pub fn unwrap_int(lit: &bril::Literal) -> i64 {
    match lit {
        bril::Literal::Int(x) => *x,
        _ => panic!("Not an int!"),
    }
}

pub fn unwrap_bool(lit: &bril::Literal) -> bool {
    match lit {
        bril::Literal::Bool(x) => *x,
        _ => panic!("Not an int!"),
    }
}

pub fn unwrap_float(lit: &bril::Literal) -> OrderedFloat<f64> {
    match lit {
        bril::Literal::Float(x) => *x,
        _ => panic!("Not a float!"),
    }
}

pub fn evaluate(op: &bril::ValueOps, args: &Vec<bril::Literal>) -> bril::Literal {
    match op {
        bril::ValueOps::Add => bril::Literal::Int(unwrap_int(&args[0]) + unwrap_int(&args[1])),
        bril::ValueOps::Sub => bril::Literal::Int(unwrap_int(&args[0]) - unwrap_int(&args[1])),
        bril::ValueOps::Mul => bril::Literal::Int(unwrap_int(&args[0]) * unwrap_int(&args[1])),
        bril::ValueOps::Div => bril::Literal::Int(unwrap_int(&args[0]) / unwrap_int(&args[1])),
        bril::ValueOps::Eq => bril::Literal::Bool(unwrap_int(&args[0]) == unwrap_int(&args[1])),
        bril::ValueOps::Lt => bril::Literal::Bool(unwrap_int(&args[0]) < unwrap_int(&args[1])),
        bril::ValueOps::Gt => bril::Literal::Bool(unwrap_int(&args[0]) > unwrap_int(&args[1])),
        bril::ValueOps::Le => bril::Literal::Bool(unwrap_int(&args[0]) <= unwrap_int(&args[1])),
        bril::ValueOps::Ge => bril::Literal::Bool(unwrap_int(&args[0]) >= unwrap_int(&args[1])),
        bril::ValueOps::Not => bril::Literal::Bool(!unwrap_bool(&args[0])),
        bril::ValueOps::And => bril::Literal::Bool(unwrap_bool(&args[0]) && unwrap_bool(&args[1])),
        bril::ValueOps::Or => bril::Literal::Bool(unwrap_bool(&args[0]) || unwrap_bool(&args[1])),
        bril::ValueOps::Fadd => bril::Literal::Float(unwrap_float(&args[0]) + unwrap_float(&args[1])),
        bril::ValueOps::Fdiv => bril::Literal::Float(unwrap_float(&args[0]) / unwrap_float(&args[1])),
        bril::ValueOps::Feq => bril::Literal::Bool(unwrap_float(&args[0]) == unwrap_float(&args[1])),
        bril::ValueOps::Fge => bril::Literal::Bool(unwrap_float(&args[0]) >= unwrap_float(&args[1])),
        bril::ValueOps::Fgt => bril::Literal::Bool(unwrap_float(&args[0]) > unwrap_float(&args[1])),
        bril::ValueOps::Fle => bril::Literal::Bool(unwrap_float(&args[0]) <= unwrap_float(&args[1])),
        bril::ValueOps::Flt => bril::Literal::Bool(unwrap_float(&args[0]) < unwrap_float(&args[1])),
        bril::ValueOps::Fmul => bril::Literal::Float(unwrap_float(&args[0]) * unwrap_float(&args[1])),
        bril::ValueOps::Fsub => bril::Literal::Float(unwrap_float(&args[0]) - unwrap_float(&args[1])),
        bril::ValueOps::Phi => {
            if args.windows(2).all(|w| w[0] == w[1]) {
                args[0].clone()
            } else {
                panic!("All args must be identical!");
            }
        }
        bril::ValueOps::Call => panic!("Unsupported!"),
        bril::ValueOps::Id => args[0].clone(),
        bril::ValueOps::Alloc => panic!("Unsupported!"),
        bril::ValueOps::Load => panic!("Unsupported!"),
        bril::ValueOps::PtrAdd => panic!("Unsupported!"),
    }
}

pub fn commutative(op: &bril::ValueOps) -> bool {
    match op {
        bril::ValueOps::Add => true,
        bril::ValueOps::Mul => true,
        bril::ValueOps::Eq => true,
        bril::ValueOps::And => true,
        bril::ValueOps::Or => true,
        _ => false,
    }
}

pub fn get_type(instr: &bril::Instruction) -> Option<bril::Type> {
    match instr {
        bril::Instruction::Constant { const_type, .. } => Some(const_type.clone()),
        bril::Instruction::Value { op_type, .. } => Some(op_type.clone()),
        _ => None,
    }
}

pub fn get_labels(instr: &bril::Instruction) -> Option<&Vec<String>> {
    match instr {
        bril::Instruction::Constant { .. } => None,
        bril::Instruction::Value { labels, .. } => Some(labels),
        bril::Instruction::Effect { labels, .. } => Some(labels),
    }
}

pub fn get_labels_mut(instr: &mut bril::Instruction) -> Option<&mut Vec<String>> {
    match instr {
        bril::Instruction::Constant { .. } => None,
        bril::Instruction::Value { labels, .. } => Some(labels),
        bril::Instruction::Effect { labels, .. } => Some(labels),
    }
}

pub fn unwrap_type(instr: &bril::Instruction) -> bril::Type {
    get_type(instr).unwrap()
}

pub fn unwrap_dest_mut(instr: &mut bril::Instruction) -> &mut String {
    match instr {
        bril::Instruction::Constant { dest, .. } => dest,
        bril::Instruction::Value { dest, .. } => dest,
        _ => panic!("Instruction has no dest!"),
    }
}

pub fn get_dest(instr: &bril::Instruction) -> Option<&String> {
    match instr {
        bril::Instruction::Constant { dest, .. } => Some(dest),
        bril::Instruction::Value { dest, .. } => Some(dest),
        _ => None,
    }
}

pub fn unwrap_dest(instr: &bril::Instruction) -> &String {
    get_dest(instr).unwrap()
}

pub fn get_args(instr: &bril::Instruction) -> Option<&Vec<String>> {
    match instr {
        bril::Instruction::Constant { .. } => None,
        bril::Instruction::Value { args, .. } => Some(args),
        bril::Instruction::Effect { args, .. } => Some(args),
    }
}

pub fn is_value_op(instr: &bril::Instruction, expect: bril::ValueOps) -> bool {
    match instr {
        bril::Instruction::Value { op, .. } => *op == expect,
        _ => false,
    }
}

pub fn id(t: bril::Type, dest: String, arg: String) -> bril::Instruction {
    bril::Instruction::Value {
        op: bril::ValueOps::Id,
        op_type: t,
        dest: dest,
        args: vec![arg],
        funcs: Vec::new(),
        labels: Vec::new(),
    }
}

pub fn constant(dest: String, value: bril::Literal) -> bril::Instruction {
    let const_type = match value {
        bril::Literal::Bool(_) => bril::Type::Bool,
        bril::Literal::Int(_) => bril::Type::Int,
        bril::Literal::Float(_) => bril::Type::Float,
    };
    bril::Instruction::Constant {
        op: bril::ConstOps::Const,
        const_type,
        dest,
        value,
    }
}

pub fn get_instr(code: &bril::Code) -> Option<&bril::Instruction> {
    match code {
        bril::Code::Instruction ( instr ) => Some(instr),
        bril::Code::Label { .. } => None,
    }
}

pub fn get_variable_types(function: &bril::Function) -> impl Iterator<Item=(&String, bril::Type)> {
    let args_iter = function.args.iter().map(|arg| (&arg.name, arg.arg_type.clone()));
    let instr_iter = function
        .instrs
        .iter()
        .filter_map(|code| get_instr(code))
        .filter(|instr| get_dest(instr).is_some())
        .map(|instr| (unwrap_dest(instr), unwrap_type(instr)));
    args_iter.chain(instr_iter)
}

// All referenced variables (may or may not be defined)
pub fn get_referenced_variables(function: &bril::Function) -> impl Iterator<Item=&String> {
    let args_iter = function.args.iter().map(|arg| &arg.name);
    let instr_dest_iter = function
        .instrs
        .iter()
        .filter_map(|code| get_instr(code))
        .filter_map(|instr| get_dest(instr));
    let instr_args_iter = function
        .instrs
        .iter()
        .filter_map(|code| get_instr(code))
        .filter_map(|instr| get_args(instr))
        .flat_map(|args| args.iter());
    args_iter.chain(instr_dest_iter).chain(instr_args_iter)
}