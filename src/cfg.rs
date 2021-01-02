use super::bb;
use super::bril;
use std::collections::{HashMap, HashSet, BTreeSet};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct CFG<'a> {
    blocks: &'a bb::BasicBlocks,
}

impl Debug for CFG<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CFG")
            .field("succ", &self.blocks.succ)
            .field("pred", &self.blocks.pred)
            .finish()
    }
}

impl PartialEq for CFG<'_> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for CFG<'_> {}

impl std::hash::Hash for CFG<'_> {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

#[derive(Copy, Clone, Debug)]
enum IterOrder {
    InOrder,
    PostOrder,
}

pub struct CFGIter<'a> {
    cfg: CFG<'a>,
    blocks: Vec<usize>,
    begin: usize,
    end: usize,
}

fn iterate(
    output: &mut Vec<usize>,
    visited: &mut Vec<bool>,
    blocks: &bb::BasicBlocks,
    order: IterOrder,
    idx: usize,
) {
    if visited[idx] {
        return;
    }
    visited[idx] = true;
    match order {
        IterOrder::InOrder => output.push(idx),
        IterOrder::PostOrder => {}
    }
    for child in &blocks.succ[idx] {
        iterate(output, visited, blocks, order, *child);
    }
    match order {
        IterOrder::InOrder => {}
        IterOrder::PostOrder => output.push(idx),
    }
}

impl<'a> CFGIter<'a> {
    fn new(cfg: CFG<'a>, order: IterOrder) -> CFGIter<'a> {
        let len = cfg.blocks.blocks.len();
        let mut visited: Vec<bool> = std::iter::repeat(false).take(len).collect();
        let mut blocks = Vec::with_capacity(len);
        iterate(&mut blocks, &mut visited, &cfg.blocks, order, 0usize);
        assert_eq!(len, blocks.len());
        CFGIter {
            cfg,
            blocks,
            begin: 0,
            end: len,
        }
    }
}

impl<'a> Iterator for CFGIter<'a> {
    type Item = Block<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            None
        } else {
            let idx = self.begin;
            self.begin += 1;
            Some(Block {
                cfg: self.cfg,
                idx: self.blocks[idx],
            })
        }
    }
}

impl<'a> DoubleEndedIterator for CFGIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            None
        } else {
            self.end -= 1;
            let idx = self.end;
            Some(Block {
                cfg: self.cfg,
                idx: self.blocks[idx],
            })
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Block<'a> {
    cfg: CFG<'a>,
    idx: usize,
}

impl Debug for Block<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block").field("idx", &self.idx).finish()
    }
}

impl<'a> Block<'a> {
    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn label(&self) -> &String {
        &self.basic_block().label
    }

    pub fn basic_block(&self) -> &bb::BasicBlock {
        &self.cfg.blocks.blocks[self.idx]
    }

    pub fn instrs(&self) -> &Vec<bril::Instruction> {
        &self.basic_block().instrs
    }

    pub fn pred(&self) -> &'a [usize] {
        &self.cfg.blocks.pred[self.idx()]
    }
    pub fn succ(&self) -> &'a [usize] {
        &self.cfg.blocks.succ[self.idx()]
    }

    pub fn predecessors(&self) -> impl Iterator<Item = Block<'_>> {
        let cfg = self.cfg;
        self.pred().iter().map(move |&idx| Block { cfg, idx })
    }

    pub fn successors(&self) -> impl Iterator<Item = Block<'_>> {
        let cfg = self.cfg;
        self.succ().iter().map(move |&idx| Block { cfg: cfg, idx })
    }
}

impl<'a> CFG<'a> {
    pub fn new(blocks: &'a bb::BasicBlocks) -> CFG<'a> {
        CFG { blocks }
    }

    pub fn post_order_iter(&self) -> CFGIter<'a> {
        CFGIter::new(*self, IterOrder::PostOrder)
    }

    pub fn in_order_iter(&self) -> CFGIter<'a> {
        CFGIter::new(*self, IterOrder::InOrder)
    }

    pub fn len(&self) -> usize {
        self.blocks.blocks.len()
    }

    pub fn get_block(&self, idx: usize) -> Block<'a> {
        Block { cfg: *self, idx }
    }
}

fn dominators(cfg: CFG) -> HashMap<usize, HashSet<usize>> {
    let all: HashSet<_> = (0usize..cfg.len()).collect();
    let mut dom: HashMap<_, _> = (0usize..cfg.len()).map(|b| (b, all.clone())).collect();

    loop {
        let mut changed = false;
        for block in cfg.post_order_iter().rev() {
            let self_dom: HashSet<usize> = vec![block.idx()].into_iter().collect();
            let mut pred_iter = block.pred().iter();
            let block_dom = pred_iter.next();
            let block_dom = match block_dom {
                None => HashSet::new(),
                Some(block_dom) => pred_iter.fold(dom[block_dom].clone(), |block_dom, pred| {
                    &block_dom & &dom[pred]
                }),
            };
            let block_dom = &block_dom | &self_dom;
            changed |= block_dom != dom[&block.idx()];
            dom.insert(block.idx(), block_dom);
        }
        if !changed {
            break;
        }
    }

    dom
}

fn strictly_dominates(cfg: CFG) -> HashMap<usize, HashSet<usize>> {
    let dominators = dominators(cfg);
    let mut dominates: HashMap<_, _> = (0usize..cfg.len()).map(|b| (b, HashSet::new())).collect();

    for (vertex, dominators) in &dominators {
        for dominator in dominators {
            dominates.get_mut(dominator).unwrap().insert(*vertex);
        }
    }
    for (vertex, dominates) in &mut dominates {
        dominates.remove(vertex);
    }
    // eprintln!("dominators = {:?}", dominators);
    // eprintln!("dominates = {:?}", dominates);

    dominates
}

#[derive(Debug)]
pub struct DominanceTree<'a> {
    cfg: CFG<'a>,
    tree: Vec<BTreeSet<usize>>,
}

pub struct DominatedIterator<'c, 'd> {
    tree: &'d DominanceTree<'c>,
    stack: Vec<usize>,
}

impl<'c, 'd> Iterator for DominatedIterator<'c, 'd> {
    type Item = Block<'c>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|idx| {
            let block = self.tree.cfg.get_block(idx);
            self.stack.extend(self.tree.tree[block.idx].iter());
            block
        })
    }
}

impl<'c> DominanceTree<'c> {
    pub fn new(cfg: CFG) -> DominanceTree {
        let dom = strictly_dominates(cfg);
        let mut tree: Vec<_> = std::iter::repeat(BTreeSet::new()).take(cfg.len()).collect();
        for (idx, set) in tree.iter_mut().enumerate() {
            let mut imm_dom: HashSet<_> = dom[&idx].clone();
            for dominated in &dom[&idx] {
                let mut dom: HashSet<_> = dom[&dominated].clone();
                dom.remove(&dominated);
                imm_dom = &imm_dom - &dom;
            }
            set.extend(imm_dom.iter());
        }
        DominanceTree { cfg, tree }
    }

    pub fn dominance_frontier(
        &'c self,
        idx: usize,
    ) -> impl Iterator<Item = (Block<'c>, Block<'c>)> {
        // For now just use an inefficient solution.
        // Can be replaced with a more efficient impl without changing callers.
        // TODO: Replace me
        let strictly_dominated: HashSet<_> = self.strictly_dominated(idx).collect();
        let cfg = self.cfg;
        self.dominated(idx)
            .flat_map(|b| b.succ().iter().map(move |s| (b, s)))
            .map(move |(b, &s)| (b, cfg.get_block(s)))
            .filter(move |(_, s)| !strictly_dominated.contains(s))
            .collect::<Vec<_>>()
            .into_iter()
        // self.tree[idx]
        //     .iter()
        //     .flat_map(|&dom| self.tree[dom].iter())
        //     .filter(|&sub_dom| !self.tree[idx].contains(sub_dom))
        //     .map(|&i| self.cfg.get_block(i))
        //     .collect::<Vec<_>>()
        //     .into_iter()
    }

    pub fn immediately_dominated(&self, idx: usize) -> impl Iterator<Item = Block<'c>> {
        self.tree[idx]
            .iter()
            .map(|&i| self.cfg.get_block(i))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn dominated(&self, idx: usize) -> DominatedIterator<'c, '_> {
        DominatedIterator {
            tree: self,
            stack: vec![idx],
        }
    }

    pub fn strictly_dominated(&self, idx: usize) -> DominatedIterator<'c, '_> {
        DominatedIterator {
            tree: self,
            stack: self.tree[idx].iter().copied().collect(),
        }
    }
}
