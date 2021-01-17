use super::{bb, bril, cfg, util};
use itertools::{izip, Itertools};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub struct SSA {
    pub function: bril::Function,
}

fn type_name(var_type: &bril::Type) -> String {
    match var_type {
        bril::Type::Bool => "bool".to_string(),
        bril::Type::Int => "int".to_string(),
        bril::Type::Float => "float".to_string(),
        bril::Type::Pointer(ptr_type) => format!("ptr.{}", type_name(ptr_type)),
    }
}

fn undefined_var(var_type: &bril::Type) -> String {
    format!("__undefined.{}", type_name(var_type))
}

fn undefined_value(var_type: &bril::Type) -> bril::Literal {
    match var_type {
        bril::Type::Bool => bril::Literal::Bool(false),
        bril::Type::Int => bril::Literal::Int(0),
        bril::Type::Float => bril::Literal::Float(OrderedFloat(0.0)),
        bril::Type::Pointer(_) => panic!("Unsupported"),
    }
}

fn add_undefined_vars(blocks: &mut bb::BasicBlocks, types: &HashMap<String, bril::Type>) {
    let mut types: Vec<_> = types.values().cloned().collect();
    types.sort();
    let types: Vec<_> = types.into_iter().unique().collect();
    let mut instrs = Vec::new();
    if types.iter().any(|t| matches!(t, bril::Type::Pointer(_))) {
        instrs.push(bril::Instruction::const_int(
            "__undefined.zero".to_string(),
            0,
        ));
    }
    for undef_type in types {
        let dest = undefined_var(&undef_type);
        let instr = match undef_type {
            bril::Type::Pointer(ptr_type) => {
                bril::Instruction::alloc(dest, "__undefined.zero".to_string(), *ptr_type)
            }
            undef_type => {
                let value = undefined_value(&undef_type);
                bril::Instruction::constant(undef_type, dest, value)
            }
        };
        instrs.push(instr);
    }
    // if let Some(block) = block {
    //     block.instrs.insert(
    //         0,
    //         bril::Instruction::const_int(undefined_var(&bril::Type::Int), 0),
    //     );
    //     block.instrs.insert(
    //         0,
    //         bril::Instruction::const_bool(undefined_var(&bril::Type::Bool), false),
    //     );
    // }
    let block = blocks.blocks.first_mut();
    if let Some(block) = block {
        std::mem::swap(&mut instrs, &mut block.instrs);
        block.instrs.extend(instrs.into_iter());
    }
}

#[derive(Clone)]
struct Def {
    block: usize,
    name: String,
}

struct ReachingDefs<'a> {
    dom: &'a cfg::DominanceTree<'a>,
    counts: HashMap<String, usize>,
    defs: HashMap<String, Def>,
    types: &'a HashMap<String, bril::Type>,
}

impl<'a> ReachingDefs<'a> {
    fn new<'b>(
        dom: &'a cfg::DominanceTree<'a>,
        types: &'a HashMap<String, bril::Type>,
        args: impl Iterator<Item = &'b String>,
    ) -> ReachingDefs<'a> {
        let defs = args
            .map(|arg| {
                (
                    arg.clone(),
                    Def {
                        block: 0,
                        name: arg.clone(),
                    },
                )
            })
            .collect();
        ReachingDefs {
            dom,
            defs,
            counts: HashMap::new(),
            types,
        }
    }

    fn fresh(&mut self, var: &String, block: usize) -> String {
        let count = self.counts.entry(var.clone()).or_insert(0);
        let new = format!("{}.{}", var, count);
        *count += 1;
        let def = self.update_reaching_def(var, block);
        if let Some(def) = def {
            self.defs.insert(new.clone(), def);
        }
        self.defs.insert(
            var.clone(),
            Def {
                block,
                name: new.clone(),
            },
        );
        new
    }

    fn reaching_def(&self, var: &String) -> Option<Def> {
        self.defs.get(var).map(|def| def.clone())
    }

    fn update_reaching_def(&mut self, var: &String, block: usize) -> Option<Def> {
        let mut r = self.reaching_def(var);
        loop {
            if let Some(def) = &r {
                let dominates = |x, y| self.dom.dominated(x).any(|b| b.idx() == y);
                if !dominates(def.block, block) {
                    r = self.reaching_def(&def.name);
                    continue;
                }
            }
            break;
        }
        match &r {
            Some(def) => self.defs.insert(var.clone(), def.clone()),
            None => self.defs.remove(var),
        };
        r
    }

    fn rename_args(&mut self, block: usize, args: &mut Vec<String>) {
        for arg in args {
            *arg = self.new_arg(block, arg);
        }
    }

    fn new_arg(&mut self, block: usize, arg: &String) -> String {
        let def = self.update_reaching_def(arg, block);
        match def {
            Some(def) => def.name,
            None => undefined_var(&self.types[arg]),
        }
    }

    fn rename_dest(&mut self, block: usize, dest: &mut String) {
        let new_dest = self.fresh(dest, block);
        *dest = new_dest;
    }
}

fn rename<'a, 'b>(
    phis: &Vec<BTreeSet<String>>,
    cfg: &cfg::CFG,
    dom: &cfg::DominanceTree,
    types: &HashMap<String, bril::Type>,
    args: impl Iterator<Item = &'a String>,
) -> (
    Vec<BTreeMap<String, (String, Vec<(String, String)>)>>,
    Vec<Vec<bril::Instruction>>,
) {
    let mut blocks = std::iter::repeat(Vec::new()).take(cfg.len()).collect();
    let mut out_phis = phis
        .iter()
        .map(|phis| {
            phis.iter()
                .map(|var| (var.clone(), (var.clone(), Vec::new())))
                .collect()
        })
        .collect();
    let mut reaching_defs = ReachingDefs::new(dom, types, args);
    rename_impl(
        &mut blocks,
        &mut reaching_defs,
        &mut out_phis,
        dom,
        cfg.get_block(0),
    );
    (out_phis, blocks)
}

fn rename_impl(
    blocks: &mut Vec<Vec<bril::Instruction>>,
    reaching_defs: &mut ReachingDefs,
    phis: &mut Vec<BTreeMap<String, (String, Vec<(String, String)>)>>,
    dom: &cfg::DominanceTree,
    block: cfg::Block,
) {
    let out_block = &mut blocks[block.idx()];
    {
        let phi = &mut phis[block.idx()];
        for (_var, (dest, _args)) in phi.iter_mut() {
            reaching_defs.rename_dest(block.idx(), dest);
        }
    }
    for mut instr in block.instrs().clone() {
        match &mut instr {
            &mut bril::Instruction::Constant { ref mut dest, .. } => {
                // eprintln!("const dest = {}", dest);
                reaching_defs.rename_dest(block.idx(), dest);
            }
            &mut bril::Instruction::Value {
                ref mut dest,
                ref mut args,
                ..
            } => {
                // assert_ne!(op, bril::ValueOps::Phi);
                reaching_defs.rename_args(block.idx(), args);
                reaching_defs.rename_dest(block.idx(), dest);
            }
            &mut bril::Instruction::Effect { ref mut args, .. } => {
                reaching_defs.rename_args(block.idx(), args);
            }
        }
        out_block.push(instr);
    }

    for &idx in block.succ() {
        for (var, (_dest, args)) in phis[idx].iter_mut() {
            let name = reaching_defs.new_arg(block.idx(), var);
            args.push((block.label().clone(), name));
        }
    }
    eprintln!(
        "{} -> {:?}",
        block.idx(),
        dom.immediately_dominated(block.idx()).collect::<Vec<_>>()
    );
    for block in dom.immediately_dominated(block.idx()) {
        rename_impl(blocks, reaching_defs, phis, dom, block);
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
            &types,
            function.args.iter().map(|arg| &arg.name),
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
        let mut blocks = bb::BasicBlocks::from_blocks(blocks);
        add_undefined_vars(&mut blocks, &types);
        let mut function = function.clone();
        function.instrs = blocks.to_instrs();

        SSA { function }
    }

    pub fn from_ssa(self) -> bril::Function {
        let mut blocks = bb::BasicBlocks::from(&self.function.instrs);

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
            let old_label = blocks.blocks[block_idx].label.clone();
            let new_label = blocks.create_label();
            // Add a block labelled new_label that defines all the variables
            // needed along the edge (pred_idx, block_idx)
            {
                let mut defined = HashSet::new(); // Variables defined in this block
                let mut block = bb::BasicBlock::from(new_label.clone());
                // Define each variable
                for (typ, dest, arg) in vars {
                    block
                        .instrs
                        .push(bril::Instruction::id(typ, dest.clone(), arg));
                    defined.insert(dest);
                }
                // Jump to the target block
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
