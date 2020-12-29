use cs6120::analysis;
use cs6120::bb;
use cs6120::bril;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = bril::load_program();
    assert_eq!(args.len(), 2);
    for func in &program.functions {
        println!("@{}", func.name);
        let analysis = &args[1];
        let blocks = bb::BasicBlocks::from(&func.instrs);
        if analysis == "const_prop" {
            let constants = analysis::constant_propagation(func.clone());
            for (idx, block) in blocks.blocks.iter().enumerate() {
                println!("\t.{}", block.label);
                let mut consts: Vec<_> = constants[idx].iter().collect();
                consts.sort_by(|(var1, _), (var2, _)| var1.cmp(var2));
                for (var, value) in consts {
                    let (type_str, value_str) = match value {
                        bril::Literal::Bool(v) => ("bool", v.to_string()),
                        bril::Literal::Int(v) => ("int", v.to_string()),
                    };
                    println!("\t\t{}: {} = {}", var, type_str, value_str);
                }
            }
        } else if analysis == "reaching_defs" {
            let reaching_defs = analysis::reaching_defs(&blocks);
            for (idx, block) in blocks.blocks.iter().enumerate() {
                println!("\t.{}", block.label);
                let mut defs: Vec<(String, _)> = reaching_defs[idx]
                    .clone()
                    .into_iter()
                    .map(|(var, locs)| {
                        let mut locs: Vec<_> = locs.into_iter().collect();
                        locs.sort();
                        (var, locs)
                    })
                    .collect();
                defs.sort();
                for (var, locs) in &defs {
                    println!("\t\t{} @", var);
                    for analysis::Loc { block, instr } in locs {
                        println!("\t\t\t{}:{}", blocks.blocks[*block].label, instr);
                    }
                }
            }
        } else if analysis == "graph" {
            for (idx, block) in blocks.blocks.iter().enumerate() {
                println!("\t.{}", block.label);
                println!("\t\tpred:");
                for pred in &blocks.pred[idx] {
                    println!("\t\t\t{}", blocks.blocks[*pred].label);
                }
                println!("\t\tsucc:");
                for succ in &blocks.succ[idx] {
                    println!("\t\t\t{}", blocks.blocks[*succ].label);
                }
            }
        } else {
            panic!("Unsupported analysis: {}", analysis);
        }
    }
}
