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

fn main() {
    let program = bril::load_program();
    let program = to_ssa(&program);
    bril::output_program(&program);
}