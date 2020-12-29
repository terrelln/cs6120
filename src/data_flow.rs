use super::bb;
use std::collections::HashSet;
use std::fmt::Debug;

pub enum DataFlowDirection {
    Forward,
    Backward,
}

pub trait DataFlowAlgorithm {
    type Result: PartialEq + Debug + 'static;

    fn direction(&self) -> DataFlowDirection;

    fn init(&self) -> Self::Result;

    fn transfer(&self, block_id: usize, block: &bb::BasicBlock, input: &Self::Result) -> Self::Result;

    fn merge<'a>(&self, input: impl Iterator<Item = &'a Self::Result>) -> Self::Result;
}

fn predecessors<'a>(
    blocks: &'a bb::BasicBlocks,
    block: usize,
    direction: &DataFlowDirection,
) -> std::slice::Iter<'a, usize> {
    match direction {
        DataFlowDirection::Forward => blocks.pred[block].iter(),
        DataFlowDirection::Backward => blocks.succ[block].iter(),
    }
}

fn successors<'a>(
    blocks: &'a bb::BasicBlocks,
    block: usize,
    direction: &DataFlowDirection,
) -> std::slice::Iter<'a, usize> {
    match direction {
        DataFlowDirection::Forward => blocks.succ[block].iter(),
        DataFlowDirection::Backward => blocks.pred[block].iter(),
    }
}

pub fn data_flow<Algo: DataFlowAlgorithm>(
    algo: Algo,
    blocks: &bb::BasicBlocks,
) -> (Vec<Algo::Result>, Vec<Algo::Result>) {
    let direction = algo.direction();
    let mut input = Vec::new();
    let mut output = Vec::new();

    if blocks.blocks.len() == 0 {
        return (Vec::new(), output);
    }

    input.push(Some(algo.init()));
    for _ in 1usize..blocks.blocks.len() {
        input.push(None);
    }
    for _ in 0usize..blocks.blocks.len() {
        output.push(algo.init());
    }

    let mut worklist: HashSet<usize> = (0usize..blocks.blocks.len()).collect();

    while worklist.len() > 0 {
        let idx = *worklist.iter().next().unwrap();

        let block_input =
            algo.merge(predecessors(&blocks, idx, &direction).map(|idx| &output[*idx]));
        let block_output = algo.transfer(idx, &blocks.blocks[idx], &block_input);
        let changed = output[idx] != block_output;
        eprintln!("{}: {:?} -> {:?}", blocks.blocks[idx].label, block_input, block_output);

        input[idx] = Some(block_input);
        output[idx] = block_output;
        eprintln!("\t{:?}", worklist);
        worklist.remove(&idx);
        if changed {
            worklist.extend(successors(&blocks, idx, &direction));
        }
        eprintln!("\t{:?}", worklist);
    }

    // for idx in 0usize..blocks.blocks.len() {
    //     input[idx] = algo.merge(predecessors(&blocks, idx, &direction).map(|idx| &output[*idx]));
    // }
    let input = input.into_iter().map(|x| x.unwrap()).collect();

    return (input, output);
}
