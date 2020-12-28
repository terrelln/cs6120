use super::bb;
use super::bril;
use std::collections::HashSet;

fn get_dest(instr: &bril::Instruction) -> Option<&String> {
    match instr {
        bril::Instruction::Constant { dest, .. } => Some(&dest),
        bril::Instruction::Value { dest, .. } => Some(&dest),
        _ => None,
    }
}

fn block_dce(block: &bb::BasicBlock) -> (bool, bb::BasicBlock) {
    //> Keeps track of the skipped instructions
    let mut skips = Vec::new();
    //> If in set, then previous assignment are dead, else alive
    let mut dead = HashSet::new();
    //> Iterate through instructions in reverse
    for (idx, instr) in block.instrs.iter().enumerate().rev() {
        match instr {
            bril::Instruction::Constant { dest, .. } => {
                if dead.contains(dest) {
                    skips.push(idx);
                } else {
                    dead.insert(dest.clone());
                }
            }
            bril::Instruction::Value { dest, args, .. } => {
                if dead.contains(dest) {
                    skips.push(idx);
                } else {
                    dead.insert(dest.clone());
                }
                for arg in args {
                    dead.remove(arg);
                }
            }
            bril::Instruction::Effect { args, .. } => {
                for arg in args {
                    dead.remove(arg);
                }
            }
        }
    }
    if skips.len() == 0 {
        return (false, block.clone());
    }

    let mut dce_block = bb::BasicBlock::new();
    dce_block.label = block.label.clone();
    dce_block.instrs.reserve(block.instrs.len() - skips.len());
    let mut skip_idx = 0;
    let mut skip = skips[skip_idx];
    for (idx, instr) in block.instrs.iter().enumerate() {
        if idx == skip {
            skip_idx += 1;
            skip = if skip_idx == skips.len() {
                block.instrs.len()
            } else {
                skips[skip_idx]
            };
        } else {
            dce_block.instrs.push(instr.clone());
        }
    }
    assert!(dce_block.instrs.len() < block.instrs.len());
    return (true, dce_block);
}

fn global_dce(blocks: &Vec<bb::BasicBlock>) -> (bool, Vec<bb::BasicBlock>) {
    let mut used = HashSet::new();
    for block in blocks {
        for instr in &block.instrs {
            match instr {
                bril::Instruction::Effect { args, .. } => used.extend(args.iter()),
                bril::Instruction::Value { args, .. } => used.extend(args.iter()),
                _ => {}
            }
        }
    }
    let mut changed = false;
    let mut dce_blocks = Vec::new();
    for block in blocks {
        let mut instrs = Vec::new();
        for instr in &block.instrs {
            if let Some(dest) = get_dest(&instr) {
                if !used.contains(&dest) {
                    changed = true;
                    continue;
                }
            }
            instrs.push(instr.clone());
        }
        dce_blocks.push(bb::BasicBlock {
            label: block.label.clone(),
            instrs,
        })
    }
    (changed, dce_blocks)
}

pub fn trivial_dce(program: &bril::Program) -> bril::Program {
    let mut dce_program = program.clone();
    for func in &mut dce_program.functions {
        let blocks = bb::BasicBlocks::from(&func.instrs);

        let mut dce_blocks = Vec::new();
        for mut block in blocks.blocks {
            loop {
                let (changed, dce_block) = block_dce(&block);
                if !changed {
                    break;
                }
                block = dce_block;
            }
            if block.instrs.len() > 0 {
                dce_blocks.push(block);
            }
        }

        loop {
            let (changed, new_blocks) = global_dce(&dce_blocks);
            if !changed {
                break;
            }
            dce_blocks = new_blocks;
        }

        func.instrs = bb::to_instrs(dce_blocks);
    }
    return dce_program;
}
