use super::{analysis, bb, bril, cfg, util};
use itertools::izip;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub struct SSA {
    pub function: bril::Function,
}

fn rename_args(args: &mut Vec<String>, names: &HashMap<String, (usize, Vec<String>)>) {
    for arg in args {
        *arg = names[arg].1.last().unwrap().clone();
    }
}

fn rename_dest(dest: &mut String, names: &mut HashMap<String, (usize, Vec<String>)>) {
    let (count, stack) = names.get_mut(dest).unwrap();
    let name = format!("{}.{}", dest, count);
    *count += 1;
    stack.push(name.clone());
    *dest = name;
}

fn undefined_var() -> String {
    "__undefined".to_string()
}

fn rename<'a, 'b>(
    phis: &Vec<BTreeSet<String>>,
    cfg: &cfg::CFG,
    dom: &cfg::DominanceTree,
    args: impl Iterator<Item = &'a String>,
    vars: impl Iterator<Item = &'b String>,
) -> (
    Vec<BTreeMap<String, (String, Vec<(String, String)>)>>,
    Vec<Vec<bril::Instruction>>,
) {
    let mut blocks = std::iter::repeat(Vec::new()).take(cfg.len()).collect();
    let mut names = {
        let vars_iter = vars.map(|var| (var.clone(), (0, Vec::new())));
        // args comes second because its value takes priority
        let args_iter = args.map(|var| (var.clone(), (0, vec![var.clone()])));
        vars_iter.chain(args_iter).collect()
    };
    let mut out_phis = phis
        .iter()
        .map(|phis| {
            phis.iter()
                .map(|var| (var.clone(), (var.clone(), Vec::new())))
                .collect()
        })
        .collect();
    rename_impl(
        &mut blocks,
        &mut names,
        &mut out_phis,
        dom,
        cfg.get_block(0),
    );
    (out_phis, blocks)
}

fn rename_impl(
    blocks: &mut Vec<Vec<bril::Instruction>>,
    names: &mut HashMap<String, (usize, Vec<String>)>,
    phis: &mut Vec<BTreeMap<String, (String, Vec<(String, String)>)>>,
    dom: &cfg::DominanceTree,
    block: cfg::Block,
) {
    let out_block = &mut blocks[block.idx()];
    let lens: Vec<_> = names
        .iter()
        .map(|(var, (_, vec))| (var.clone(), vec.len()))
        .collect();
    {
        let phi = &mut phis[block.idx()];
        for (_var, (dest, _args)) in phi.iter_mut() {
            rename_dest(dest, names);
        }
    }
    for mut instr in block.instrs().clone() {
        match &mut instr {
            &mut bril::Instruction::Constant { ref mut dest, .. } => {
                // eprintln!("const dest = {}", dest);
                rename_dest(dest, names);
            }
            &mut bril::Instruction::Value {
                ref mut dest,
                ref mut args,
                ..
            } => {
                // assert_ne!(op, bril::ValueOps::Phi);
                rename_args(args, names);
                rename_dest(dest, names);
            }
            &mut bril::Instruction::Effect { ref mut args, .. } => {
                rename_args(args, names);
            }
        }
        out_block.push(instr);
    }

    for &idx in block.succ() {
        for (var, (_dest, args)) in phis[idx].iter_mut() {
            let name = match names[var].1.last() {
                None => undefined_var(),
                Some(name) => name.clone(),
            };
            args.push((block.label().clone(), name));
        }
    }
    eprintln!(
        "{} -> {:?}",
        block.idx(),
        dom.immediately_dominated(block.idx()).collect::<Vec<_>>()
    );
    for block in dom.immediately_dominated(block.idx()) {
        rename_impl(blocks, names, phis, dom, block);
    }
    for (var, len) in &lens {
        names.get_mut(var).unwrap().1.resize(*len, String::new());
    }
}

fn phi(dest: String, op_type: bril::Type, labels_args: Vec<(String, String)>) -> bril::Instruction {
    let (labels, args) = labels_args.into_iter().unzip();
    bril::Instruction::Value {
        dest,
        op_type,
        op: bril::ValueOps::Phi,
        labels: labels,
        args: args,
        funcs: Vec::new(),
    }
}

impl SSA {
    pub fn from_function(function: &bril::Function) -> SSA {
        let blocks = bb::BasicBlocks::from(&function.instrs);
        let mut defs = HashMap::new();
        let mut types = HashMap::new();
        for arg in &function.args {
            defs.entry(arg.name.clone())
                .or_insert(HashSet::new())
                .insert(0);
            types.insert(arg.name.clone(), arg.arg_type.clone());
        }
        for (idx, block) in blocks.blocks.iter().enumerate() {
            for instr in &block.instrs {
                if let Some(dest) = util::get_dest(&instr) {
                    defs.entry(dest.clone())
                        .or_insert(HashSet::new())
                        .insert(idx);
                    types.insert(dest.clone(), util::unwrap_type(&instr));
                }
            }
        }
        let cfg = cfg::CFG::new(&blocks);
        // eprintln!("{:?}", cfg);
        let dom = cfg::DominanceTree::new(cfg);
        // eprintln!("{:?}", dom);
        // for idx in 0..cfg.len() {
        //     eprintln!("df {}", idx);
        //     eprintln!("{:?}", dom.dominance_frontier(idx).collect::<Vec<_>>());
        // }

        let mut phis: Vec<BTreeSet<String>> =
            std::iter::repeat(BTreeSet::new()).take(cfg.len()).collect();

        // eprintln!("defs = {:?}", defs);
        for (var, mut defs) in defs {
            let mut visited = HashSet::new();
            while let Some(&def) = defs.iter().next() {
                let df = dom.dominance_frontier(def);
                defs.remove(&def);
                for (_pred, block) in df {
                    if visited.insert(block.idx()) {
                        phis[block.idx()].insert(var.clone());
                        defs.insert(block.idx());
                    }
                }
            }
        }
        // eprintln!("phis: {:?}", phis);

        let mut names: HashMap<_, _> = types.keys().map(|var| (var.clone(), Vec::new())).collect();
        for arg in &function.args {
            names.get_mut(&arg.name).unwrap().push(arg.name.clone());
        }
        let (out_phis, out_blocks) = rename(
            &phis,
            &cfg,
            &dom,
            function.args.iter().map(|arg| &arg.name),
            types.keys(),
        );
        let iter = izip!(
            out_phis.into_iter(),
            out_blocks.into_iter(),
            blocks.blocks.iter().map(|block| block.label.clone()),
        );
        let blocks = iter
            .map(|(phis, instrs, label)| {
                let instrs = phis
                    .into_iter()
                    .map(|(var, (dest, args))| phi(dest, types[&var].clone(), args))
                    .chain(instrs.into_iter())
                    .collect();
                bb::BasicBlock { instrs, label }
            })
            .collect();
        let blocks = bb::BasicBlocks::from_blocks(blocks);
        let mut function = function.clone();
        function.instrs = blocks.to_instrs();

        SSA { function }
    }

    pub fn from_ssa(self) -> bril::Function {
        let mut blocks = bb::BasicBlocks::from(&self.function.instrs);
        let init_after = analysis::initialized_variables(
            &blocks,
            self.function
                .args
                .iter()
                .map(|arg| arg.name.clone())
                .collect(),
        )
        .1;
        eprintln!("INIT = {:?}", init_after);

        // let mut rewrites = Vec::new();
        let mut rewrites = BTreeMap::new();
        for idx in 0..blocks.blocks.len() {
            let block = &blocks.blocks[idx];
            let mut instrs = Vec::new();
            for instr in &block.instrs {
                if util::is_value_op(instr, bril::ValueOps::Phi) {
                    let dest = util::unwrap_dest(instr);
                    let typ = util::unwrap_type(instr);
                    let args = util::get_args(instr).unwrap();
                    let labels = util::get_labels(instr).unwrap();
                    for (arg, label) in args.iter().zip(labels.iter()) {
                        let pred = blocks.labels[label];
                        rewrites.entry((pred, idx)).or_insert(Vec::new()).push((
                            typ.clone(),
                            dest.clone(),
                            arg.clone(),
                        ));
                    }
                } else {
                    instrs.push(instr.clone());
                }
            }
            blocks.blocks[idx].instrs = instrs;
        }

        for ((pred_idx, block_idx), vars) in rewrites {
            let mut defined = HashSet::new();
            let old_label = blocks.blocks[block_idx].label.clone();
            let new_label = blocks.create_label();
            // Add a new block
            {
                let mut block = bb::BasicBlock::from(new_label.clone());
                for (typ, dest, arg) in vars {
                    if init_after[pred_idx].contains(&arg) || defined.contains(&arg) {
                        block
                            .instrs
                            .push(bril::Instruction::id(typ, dest.clone(), arg));
                    }
                    defined.insert(dest);
                }
                block
                    .instrs
                    .push(bril::Instruction::jump(old_label.clone()));
                blocks.blocks.push(block);
            }
            // Rewrite labels in predecessor: old_label -> new_label
            blocks.blocks[pred_idx]
                .instrs
                .iter_mut()
                .filter_map(|instr| util::get_labels_mut(instr))
                .flat_map(|labels| labels.iter_mut())
                .filter(|label| *label == &old_label)
                .for_each(|label| *label = new_label.clone());
        }

        let instrs = blocks.to_instrs();
        bril::Function {
            instrs,
            ..self.function
        }
    }
}
