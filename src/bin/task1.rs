use cs6120::bril;

fn main() {
    let program = bril::load_program();
    let num_functions = program.functions.len();
    let num_instrs_and_labels = program
        .functions
        .iter()
        .fold(0, |acc, func| acc + func.instrs.len());
    let num_labels = program.functions.iter().fold(0, |acc, func| {
        acc + func
            .instrs
            .iter()
            .filter(|instr| matches!(instr, bril::Code::Label{ label: _ }))
            .count()
    });
    let num_instrs = num_instrs_and_labels - num_labels;
    println!("num_functions: {}", num_functions);
    println!("num_instrs: {}", num_instrs);
    println!("num_labels: {}", num_labels);
}
