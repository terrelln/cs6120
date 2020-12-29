use super::bril;

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
        bril::ValueOps::Call => panic!("Unsupported!"),
        bril::ValueOps::Id => args[0].clone(),
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
    };
    bril::Instruction::Constant {
        op: bril::ConstOps::Const,
        const_type,
        dest,
        value
    }
}