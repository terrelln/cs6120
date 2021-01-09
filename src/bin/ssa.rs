use cs6120::bril;
use cs6120::ssa;

fn to_ssa(program: &bril::Program) -> bril::Program {
    let mut program = program.clone();
    for func in &mut program.functions {
        let ssa = ssa::SSA::from_function(func);
        *func = ssa.function;
    }
    program
}

fn from_ssa(program: &bril::Program) -> bril::Program {
    let mut program = program.clone();
    for func in &mut program.functions {
        let ssa = ssa::SSA {
            function: func.clone(),
        };
        *func = ssa.from_ssa();
    }
    program
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let mut program = bril::load_program();
    if args[1] == "to" {
        program = to_ssa(&program);
    } else if args[1] == "from" {
        program = from_ssa(&program);
    } else {
        panic!("Command not to/from");
    }
    bril::output_program(&program);
}
