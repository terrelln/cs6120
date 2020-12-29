use super::{bb, bril};
use super::util::{is_effect, evaluate, commutative, unwrap_type, unwrap_dest_mut, get_dest, unwrap_dest, id};
use std::collections::HashMap;
use std::mem::swap;

// fn get_arg_type(
//     program: &bril::Program,
//     function: &bril::Function,
//     instr: &bril::Instruction,
//     arg: usize,
// ) -> Option<bril::Type> {
//     match instr {
//         bril::Instruction::Constant { .. } => panic!("No arguments -> impossible!"),
//         bril::Instruction::Value {
//             op, op_type, funcs, ..
//         } => {
//             let value_type = match op {
//                 bril::ValueOps::Add => bril::Type::Int,
//                 bril::ValueOps::Sub => bril::Type::Int,
//                 bril::ValueOps::Mul => bril::Type::Int,
//                 bril::ValueOps::Div => bril::Type::Int,
//                 bril::ValueOps::Eq => bril::Type::Int,
//                 bril::ValueOps::Lt => bril::Type::Int,
//                 bril::ValueOps::Gt => bril::Type::Int,
//                 bril::ValueOps::Le => bril::Type::Int,
//                 bril::ValueOps::Ge => bril::Type::Int,
//                 bril::ValueOps::Not => bril::Type::Bool,
//                 bril::ValueOps::And => bril::Type::Bool,
//                 bril::ValueOps::Or => bril::Type::Bool,
//                 bril::ValueOps::Call => {
//                     assert_eq!(funcs.len(), 1);
//                     let func = program
//                         .functions
//                         .iter()
//                         .filter(|f| f.name == funcs[0])
//                         .next()
//                         .unwrap();
//                     func.args[arg].arg_type.clone()
//                 }
//                 bril::ValueOps::Id => op_type.clone(),
//             };
//             Some(value_type)
//         }
//         bril::Instruction::Effect { op, funcs, .. } => {
//             match op {
//                 bril::EffectOps::Jump => None,
//                 bril::EffectOps::Branch => Some(bril::Type::Bool),
//                 bril::EffectOps::Call => {
//                     assert_eq!(funcs.len(), 1);
//                     let func = program
//                         .functions
//                         .iter()
//                         .filter(|f| f.name == funcs[0])
//                         .next()
//                         .unwrap();
//                     Some(func.args[arg].arg_type.clone())
//                 }
//                 bril::EffectOps::Return => function.return_type.clone(),
//                 bril::EffectOps::Print =>
//                 /* Unknown type */
//                 {
//                     None
//                 }
//                 bril::EffectOps::Nop => panic!("No args -> impossible!"),
//             }
//         }
//     }
// }

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
    var_types: HashMap<String, bril::Type>,
    instrs: Vec<bril::Instruction>,
}


impl LVN {
    pub fn new() -> Self {
        LVN {
            var_to_num: HashMap::new(),
            value_table: HashMap::new(),
            values: Vec::new(),
            num_writes: HashMap::new(),
            var_types: HashMap::new(),
            instrs: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.var_to_num.clear();
        self.value_table.clear();
        self.values.clear();
        self.num_writes.clear();
        self.var_types.clear();
        self.instrs.clear();
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
                *count -= 1;
                *count
            }
            None => 0,
        }
    }

    fn get_writes(&self, var: &String) -> usize {
        *self.num_writes.get(var).unwrap_or(&0)
    }

    fn to_num(&mut self, var: &String) -> usize {
        match self.var_to_num.get(var) {
            Some(num) => *num,
            None => self.new_id_value(self.var_types[var].clone(), var),
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
        instr: bril::Instruction,
        num_instr: NumInstr,
        dest: String,
        num: usize,
        remaining_writes: usize,
        push_instr: bool,
    ) {
        let dest = if remaining_writes == 0 {
            if push_instr {
                self.instrs.push(instr);
            }
            dest
        } else {
            let mut instr = instr;
            let new_dest = format!("__var{}", num);
            let mut dest = new_dest.clone();
            let instr_dest = unwrap_dest_mut(&mut instr);
            // Run the old instruciton with the new dest
            if push_instr {
                swap(&mut dest, instr_dest);
                self.instrs.push(instr.clone());
            }
            // Copy to old dest
            self.instrs.push(id(
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

    fn new_id_value(&mut self, id_type: bril::Type, var: &String) -> usize {
        assert!(!self.var_to_num.contains_key(var));
        let num = self.values.len();
        let instr = id(id_type, var.clone(), var.clone());
        let value = NumInstr {
            op: Op::ValueOps(bril::ValueOps::Id),
            args: vec![num],
            value: None,
        };
        // self.value_table.insert(value.clone(), (var.clone(), num));
        // self.values.push(value);
        let remaining_writes = self.get_writes(var);
        self.add_value(instr, value, var.clone(), num, remaining_writes, false);
        self.var_to_num.insert(var.clone(), num);
        num
    }

    fn process_instr(&mut self, instr: &bril::Instruction) {
        let (instr, dest, mut num) = self.rewrite(instr.clone());
        let num_instr = self.convert(&instr).unwrap();
        let remaining_writes = if &dest == unwrap_dest(&instr) {
            self.sub_write(&dest)
        } else {
            self.get_writes(&dest)
        };
        match self.value_table.get(&num_instr) {
            Some((var, old_num)) => {
                let var = var.clone();
                num = *old_num;
                let instr = self
                    .rewrite(id(unwrap_type(&instr), unwrap_dest(&instr).clone(), var))
                    .0;
                self.instrs.push(instr);
            }
            None => self.add_value(
                instr.clone(),
                num_instr,
                dest.clone(),
                num,
                remaining_writes,
                true,
            ),
        };
        self.var_to_num.insert(unwrap_dest(&instr).clone(), num);
    }

    fn assign_types(&mut self, func: &bril::Function) {
        for bril::Argument { name, arg_type } in &func.args {
            self.var_types.insert(name.clone(), arg_type.clone());
        }
        for instr in &func.instrs {
            if let bril::Code::Instruction(instr) = instr {
                if let Some(dest) = get_dest(&instr) {
                    let dest_type = unwrap_type(&instr);
                    self.var_types.insert(dest.clone(), dest_type);
                }
            }
        }
    }

    pub fn process(&mut self, func: &bril::Function, block: &bb::BasicBlock) -> bb::BasicBlock {
        self.clear();
        self.count_writes(block);
        self.assign_types(func);
        for instr in &block.instrs {
            if is_effect(instr) {
                let instr = self.rewrite(instr.clone()).0;
                self.instrs.push(instr);
            } else {
                self.process_instr(instr);
            }
        }
        let mut instrs = Vec::new();
        swap(&mut self.instrs, &mut instrs);
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
            lvn_blocks.push(lvn.process(func, block));
        }
        func.instrs = bb::to_instrs(lvn_blocks);
    }
    return lvn_program;
}
