use super::{bb, bril};
use std::collections::HashMap;
use std::mem::swap;

fn is_effect(instr: &bril::Instruction) -> bool {
    match instr {
        bril::Instruction::Effect { .. } => true,
        _ => false,
    }
}

fn unwrap_int(lit: &bril::Literal) -> i64 {
    match lit {
        bril::Literal::Int(x) => *x,
        _ => panic!("Not an int!"),
    }
}

fn unwrap_bool(lit: &bril::Literal) -> bool {
    match lit {
        bril::Literal::Bool(x) => *x,
        _ => panic!("Not an int!"),
    }
}

fn evaluate(op: &bril::ValueOps, args: &Vec<bril::Literal>) -> bril::Literal {
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

fn commutative(op: &bril::ValueOps) -> bool {
    match op {
        bril::ValueOps::Add => true,
        bril::ValueOps::Mul => true,
        bril::ValueOps::Eq => true,
        bril::ValueOps::And => true,
        bril::ValueOps::Or => true,
        _ => false,
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Op {
    ConstantOps(bril::ConstOps),
    ValueOps(bril::ValueOps),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NumInstr {
    op: Op,
    args: Vec<usize>,
    value: Option<bril::Literal>,
}

pub struct LVN {
    var_to_num: HashMap<String, usize>,
    value_table: HashMap<NumInstr, (String, usize)>,
    values: Vec<NumInstr>,
    num_writes: HashMap<String, usize>,
}

fn unwrap_type(instr: &bril::Instruction) -> bril::Type {
    match instr {
        bril::Instruction::Constant { const_type, .. } => const_type.clone(),
        bril::Instruction::Value { op_type, .. } => op_type.clone(),
        _ => panic!("Instruction has no type!"),
    }
}

fn unwrap_dest_mut(instr: &mut bril::Instruction) -> &mut String {
    match instr {
        bril::Instruction::Constant { dest, .. } => dest,
        bril::Instruction::Value { dest, .. } => dest,
        _ => panic!("Instruction has no dest!"),
    }
}

fn unwrap_dest(instr: &bril::Instruction) -> &String {
    match instr {
        bril::Instruction::Constant { dest, .. } => dest,
        bril::Instruction::Value { dest, .. } => dest,
        _ => panic!("Instruction has no dest!"),
    }
}

fn id(t: bril::Type, dest: String, arg: String) -> bril::Instruction {
    bril::Instruction::Value {
        op: bril::ValueOps::Id,
        op_type: t,
        dest: dest,
        args: vec![arg],
        funcs: Vec::new(),
        labels: Vec::new(),
    }
}

impl LVN {
    pub fn new() -> Self {
        LVN {
            var_to_num: HashMap::new(),
            value_table: HashMap::new(),
            values: Vec::new(),
            num_writes: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.var_to_num.clear();
        self.value_table.clear();
        self.values.clear();
        self.num_writes.clear();
    }

    fn add_write(&mut self, var: &String) {
        *self.num_writes.entry(var.clone()).or_insert(0) += 1;
    }

    fn count_writes(&mut self, block: &bb::BasicBlock) {
        for instr in &block.instrs {
            match instr {
                bril::Instruction::Constant { dest, .. } => self.add_write(dest),
                bril::Instruction::Value { dest, .. } => self.add_write(dest),
                _ => {}
            };
        }
    }

    fn sub_write(&mut self, var: &String) -> usize {
        let count = self.num_writes.get_mut(var);
        match count {
            Some(count) => {
                if *count > 0 {
                    *count -= 1;
                }
                *count
            },
            None => 0
        }
    }

    fn new_id_value(&mut self, var: &String) -> usize {
        assert!(!self.var_to_num.contains_key(var));
        let num = self.values.len();
        self.var_to_num.insert(var.clone(), num);
        let value = NumInstr {
            op: Op::ValueOps(bril::ValueOps::Id),
            args: vec![num],
            value: None,
        };
        self.value_table.insert(value.clone(), (var.clone(), num));
        self.values.push(value);
        num
    }

    fn to_num(&mut self, var: &String) -> usize {
        match self.var_to_num.get(var) {
            Some(num) => *num,
            None => self.new_id_value(var),
        }
    }
    fn convert_args(&mut self, op: &bril::ValueOps, args: &Vec<String>) -> Vec<usize> {
        let mut args: Vec<usize> = args.iter().map(|var| self.to_num(var)).collect();
        if commutative(op) {
            args.sort();
        }
        return args;
    }

    fn convert(&mut self, instr: &bril::Instruction) -> Option<NumInstr> {
        match instr {
            bril::Instruction::Constant { op, value, .. } => Some(NumInstr {
                op: Op::ConstantOps(*op),
                args: Vec::new(),
                value: Some(value.clone()),
            }),
            bril::Instruction::Value { op, args, .. } => Some(NumInstr {
                op: Op::ValueOps(*op),
                args: self.convert_args(op, args),
                value: None,
            }),
            bril::Instruction::Effect { .. } => None,
        }
    }

    fn compute_constant(&self, op: &bril::ValueOps, args: &Vec<String>) -> bril::Literal {
        let args: Vec<bril::Literal> = args
            .iter()
            .map(|arg| {
                let num = self.var_to_num[arg];
                let value = &self.values[num];
                value.value.clone().unwrap()
            })
            .collect();
        evaluate(op, &args)
    }

    fn constant(
        &self,
        op: &bril::ValueOps,
        const_type: bril::Type,
        dest: String,
        args: &Vec<String>,
    ) -> bril::Instruction {
        let value = self.compute_constant(op, args);
        bril::Instruction::Constant {
            op: bril::ConstOps::Const,
            dest,
            const_type,
            value,
        }
    }

    fn is_constant(&self, arg: &String) -> bool {
        let num = self.var_to_num[arg];
        let value = &self.values[num];
        return value.op == Op::ConstantOps(bril::ConstOps::Const);
    }

    fn rewrite_args(&mut self, args: &mut Vec<String>) {
        for arg in args {
            let num = self.to_num(arg);
            let value = &self.values[num];
            let (var, num2) = self.value_table.get(value).unwrap();
            assert_eq!(num, *num2);
            *arg = var.clone();
        }
    }

    fn rewrite(&mut self, instr: bril::Instruction) -> (bril::Instruction, String, usize) {
        let mut instr = instr;
        match &mut instr {
            bril::Instruction::Constant { dest, .. } => {
                let dest = dest.clone();
                (instr, dest, self.values.len())
            }
            bril::Instruction::Value {
                op,
                dest,
                args,
                op_type,
                ..
            } => {
                self.rewrite_args(args);
                if *op != bril::ValueOps::Call && args.iter().all(|arg| self.is_constant(arg)) {
                    (
                        self.constant(op, op_type.clone(), dest.clone(), args),
                        dest.clone(),
                        self.values.len(),
                    )
                } else if *op == bril::ValueOps::Id {
                    let num = self.to_num(&args[0]);
                    let value = &self.values[num];
                    let (var, _) = self.value_table.get(value).unwrap();
                    (
                        id(
                            unwrap_type(&instr),
                            unwrap_dest(&instr).clone(),
                            var.clone(),
                        ),
                        var.clone(),
                        num,
                    )
                } else {
                    let dest = dest.clone();
                    (instr, dest, self.values.len())
                }
            }
            bril::Instruction::Effect { args, .. } => {
                self.rewrite_args(args);
                (instr, String::new(), 0)
            }
        }
    }

    fn add_value(
        &mut self,
        instrs: &mut Vec<bril::Instruction>,
        instr: bril::Instruction,
        num_instr: NumInstr,
        dest: String,
        num: usize,
        remaining_writes: usize,
    ) {
        let dest = if remaining_writes == 0 {
            instrs.push(instr);
            dest
        } else {
            let mut instr = instr;
            let new_dest = format!("__var{}", num);
            let mut dest = new_dest.clone();
            let instr_dest = unwrap_dest_mut(&mut instr);
            swap(&mut dest, instr_dest);
            // Run the old instruciton with the new dest
            instrs.push(instr.clone());
            // Copy to old dest
            instrs.push(id(
                unwrap_type(&instr),
                dest.clone(),
                unwrap_dest(&instr).clone(),
            ));
            self.var_to_num.insert(new_dest.clone(), num);
            new_dest
        };

        self.value_table
            .insert(num_instr.clone(), (dest.clone(), num));
        self.values.push(num_instr);
    }

    fn process_instr(&mut self, instrs: &mut Vec<bril::Instruction>, instr: &bril::Instruction) {
        let (instr, dest, mut num) = self.rewrite(instr.clone());
        let num_instr = self.convert(&instr).unwrap();
        // let dest = unwrap_dest(instr);
        let remaining_writes = self.sub_write(&dest);
        match self.value_table.get(&num_instr) {
            Some((var, old_num)) => {
                let var = var.clone();
                num = *old_num;
                instrs.push(self.rewrite(id(unwrap_type(&instr), unwrap_dest(&instr).clone(), var)).0);
            }
            None => self.add_value(instrs, instr.clone(), num_instr, dest.clone(), num, remaining_writes),
        };
        self.var_to_num.insert(unwrap_dest(&instr).clone(), num);
    }

    pub fn process(&mut self, block: &bb::BasicBlock) -> bb::BasicBlock {
        self.clear();
        self.count_writes(block);
        let mut instrs = Vec::new();
        for instr in &block.instrs {
            if is_effect(instr) {
                instrs.push(self.rewrite(instr.clone()).0);
            } else {
                self.process_instr(&mut instrs, instr);
            }
        }
        bb::BasicBlock {
            label: block.label.clone(),
            instrs,
        }
    }
}

pub fn lvn(program: &bril::Program) -> bril::Program {
    let mut lvn = LVN::new();
    let mut lvn_program = program.clone();
    for func in &mut lvn_program.functions {
        let blocks = bb::BasicBlocks::from(&func.instrs);
        let mut lvn_blocks = Vec::new();
        for block in &blocks.blocks {
            lvn_blocks.push(lvn.process(block));
        }
        func.instrs = bb::to_instrs(lvn_blocks);
    }
    return lvn_program;
}
