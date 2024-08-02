use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

enum Color {
    //     White,
    Grey,
    Black,
}

enum Traversal {
    PreOrder,
    PostOrder,
}

pub struct GraphIter<G: Graph> {
    nodes: Vec<G::Node>,
    iter: std::ops::Range<usize>,
}

impl<G: Graph> Clone for GraphIter<G> {
    fn clone(&self) -> Self {
        GraphIter {
            nodes: self.nodes.clone(),
            iter: self.iter.clone(),
        }
    }
}

fn traverse<G: Graph>(
    graph: &G,
    nodes: &mut Vec<G::Node>,
    colors: &mut HashMap<G::Node, Color>,
    traversal: &Traversal,
    node: G::Node,
) {
    if colors.contains_key(&node) {
        return;
    }
    colors.insert(node, Color::Grey);
    match traversal {
        Traversal::PreOrder => nodes.push(node),
        Traversal::PostOrder => {}
    }
    for child in graph.successors(node) {
        traverse(graph, nodes, colors, traversal, child);
    }
    *colors.get_mut(&node).unwrap() = Color::Black;
    match traversal {
        Traversal::PreOrder => {}
        Traversal::PostOrder => nodes.push(node),
    }
}

impl<G: Graph> GraphIter<G> {
    fn new(graph: &G, root: Option<G::Node>, traversal: Traversal) -> Self {
        let mut colors = HashMap::new();
        let mut nodes = Vec::new();
        if let Some(root) = root {
            traverse(graph, &mut nodes, &mut colors, &traversal, root);
        }
        let iter = 0..nodes.len();
        GraphIter { nodes, iter }
    }
}

impl<G: Graph> Iterator for GraphIter<G> {
    type Item = G::Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|i| self.nodes[i])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<G: Graph> DoubleEndedIterator for GraphIter<G> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|i| self.nodes[i])
    }
}

pub struct IndirectGraphIter<'a, G: IndirectGraph> {
    graph: &'a G,
    iter: GraphIter<G>,
}

impl<'a, G: IndirectGraph> Iterator for IndirectGraphIter<'a, G> {
    type Item = &'a G::NodeValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|n| self.graph.get(n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, G: IndirectGraph> DoubleEndedIterator for IndirectGraphIter<'a, G> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|n| self.graph.get(n))
    }
}

// pub struct MutableIndirectGraphIter<'a, G: MutableIndirectGraph> {
//     graph: &'a mut G,
//     iter: GraphIter<G>,
// }

// impl<'a, G: MutableIndirectGraph> Iterator for MutableIndirectGraphIter<'a, G> {
//     type Item = &'a mut G::NodeValue;

//     fn next(&mut self) -> Option<&'a mut G::NodeValue> {
//         match self.iter.next().take() {
//             Some(node) => Some(&mut *self.graph.get_mut(node)),
//             None => None,
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl <'a, G: MutableIndirectGraph> DoubleEndedIterator for MutableIndirectGraphIter<'a, G> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|n| self.graph.get_mut(n))
//     }
// }

pub trait Graph {
    type Node: Copy + Eq + Hash + Debug;

    fn display(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    where
        Self::Node: Display,
    {
        for node in self.nodes() {
            write!(f, "{}: [", node)?;
            for (i, succ) in self.successors(node).enumerate() {
                write!(f, " {}", succ)?;
                if i + 1 != self.successors(node).count() {
                    write!(f, ",")?;
                }
            }
            writeln!(f, " ]")?;
        }

        Ok(())
    }

    fn debug(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    where
        Self::Node: Debug,
    {
        for node in self.nodes() {
            write!(f, "{:?}: [", node)?;
            for (i, succ) in self.successors(node).enumerate() {
                write!(f, " {:?}", succ)?;
                if i + 1 != self.successors(node).count() {
                    write!(f, ",")?;
                }
            }
            writeln!(f, " ]")?;
        }

        Ok(())
    }

    fn entry_node(&self) -> Option<Self::Node>;

    fn exit_node(&self) -> Option<Self::Node>;

    fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_;

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_;

    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_;

    fn post_order_iter(&self) -> GraphIter<Self>
    where
        Self: Sized,
    {
        GraphIter::new(self, self.entry_node(), Traversal::PostOrder)
    }

    fn post_order_iter_at(&self, node: Self::Node) -> GraphIter<Self>
    where
        Self: Sized,
    {
        GraphIter::new(self, Some(node), Traversal::PostOrder)
    }

    fn pre_order_iter(&self) -> GraphIter<Self>
    where
        Self: Sized,
    {
        GraphIter::new(self, self.entry_node(), Traversal::PreOrder)
    }

    fn pre_order_iter_at(&self, node: Self::Node) -> GraphIter<Self>
    where
        Self: Sized,
    {
        GraphIter::new(self, Some(node), Traversal::PreOrder)
    }
}

pub struct DisplayGraph<G: Graph>(pub G);

impl<G: Graph> Display for DisplayGraph<G>
where
    G::Node: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.display(f)
    }
}

impl<G: Graph> Debug for DisplayGraph<G>
where
    G::Node: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.debug(f)
    }
}

pub trait IndirectGraph: Graph {
    type NodeValue;

    fn get(&self, node: Self::Node) -> &Self::NodeValue;
}

pub trait MutableIndirectGraph: IndirectGraph {
    fn get_mut(&mut self, node: Self::Node) -> &mut Self::NodeValue;
}

impl<'a, G: Graph> Graph for &'a G {
    type Node = G::Node;

    fn entry_node(&self) -> Option<Self::Node> {
        (*self).entry_node()
    }

    fn exit_node(&self) -> Option<Self::Node> {
        (*self).exit_node()
    }

    fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_ {
        (*self).nodes()
    }

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        (*self).predecessors(node)
    }

    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        (*self).successors(node)
    }
}

impl<'a, G: IndirectGraph> IndirectGraph for &'a G {
    type NodeValue = G::NodeValue;

    fn get(&self, node: Self::Node) -> &Self::NodeValue {
        (*self).get(node)
    }
}

impl<'a, G: Graph> Graph for &'a mut G {
    type Node = G::Node;

    fn entry_node(&self) -> Option<Self::Node> {
        (**self).entry_node()
    }

    fn exit_node(&self) -> Option<Self::Node> {
        (**self).exit_node()
    }

    fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_ {
        (**self).nodes()
    }

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        (**self).predecessors(node)
    }

    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        (**self).successors(node)
    }
}

impl<'a, G: IndirectGraph> IndirectGraph for &'a mut G {
    type NodeValue = G::NodeValue;

    fn get(&self, node: Self::Node) -> &Self::NodeValue {
        (**self).get(node)
    }
}

impl<'a, G: MutableIndirectGraph> MutableIndirectGraph for &'a mut G {
    fn get_mut(&mut self, node: Self::Node) -> &mut Self::NodeValue {
        (**self).get_mut(node)
    }
}

pub struct Inverse<G: Graph>(pub G);

impl<G: Graph> Graph for Inverse<G> {
    type Node = G::Node;

    fn entry_node(&self) -> Option<Self::Node> {
        self.0.exit_node()
    }

    fn exit_node(&self) -> Option<Self::Node> {
        self.0.entry_node()
    }

    fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_ {
        self.0.nodes()
    }

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.0.successors(node)
    }

    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.0.predecessors(node)
    }
}

impl<G: IndirectGraph> IndirectGraph for Inverse<G> {
    type NodeValue = G::NodeValue;

    fn get(&self, node: Self::Node) -> &Self::NodeValue {
        self.0.get(node)
    }
}

impl<G: MutableIndirectGraph> MutableIndirectGraph for Inverse<G> {
    fn get_mut(&mut self, node: Self::Node) -> &mut Self::NodeValue {
        self.0.get_mut(node)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[derive(Clone, Debug)]
    pub struct SimpleGraph {
        nodes: Vec<i32>,
        pred: HashMap<i32, BTreeSet<i32>>,
        succ: HashMap<i32, BTreeSet<i32>>,
        entry: Option<i32>,
        exit: Option<i32>,
    }

    impl SimpleGraph {
        pub fn new() -> Self {
            SimpleGraph {
                nodes: Vec::new(),
                pred: HashMap::new(),
                succ: HashMap::new(),
                entry: None,
                exit: None,
            }
        }

        pub fn set_entry(&mut self, entry: i32) {
            self.entry = Some(entry);
        }

        pub fn set_exit(&mut self, exit: i32) {
            self.exit = Some(exit);
        }

        pub fn add_node(&mut self, node: i32) {
            self.nodes.push(node);
        }

        pub fn extend_nodes(&mut self, nodes: impl Iterator<Item = i32>) {
            self.nodes.extend(nodes);
        }

        pub fn add_edge(&mut self, from: i32, to: i32) {
            self.succ.entry(from).or_insert_with(BTreeSet::new).insert(to);
            self.pred.entry(to).or_insert_with(BTreeSet::new).insert(from);
        }

        pub fn extend_edges(&mut self, edges: impl Iterator<Item = (i32, i32)>) {
            for (from, to) in edges {
                self.add_edge(from, to);
            }
        }
    }

    impl Graph for SimpleGraph {
        type Node = i32;

        fn entry_node(&self) -> Option<Self::Node> {
            self.entry
        }

        fn exit_node(&self) -> Option<Self::Node> {
            self.exit
        }

        fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_ {
            self.nodes.iter().copied()
        }

        fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
            match self.pred.get(&node) {
                Some(pred) => pred.clone().into_iter(),
                None => BTreeSet::new().into_iter(),
            }
        }

        fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
            match self.succ.get(&node) {
                Some(succ) => succ.clone().into_iter(),
                None => BTreeSet::new().into_iter(),
            }
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    struct TestNode(usize);

    struct TestGraph {
        nodes: Vec<TestNode>,
        pred: Vec<Vec<usize>>,
        succ: Vec<Vec<usize>>,
    }

    impl Graph for TestGraph {
        type Node = usize;

        fn entry_node(&self) -> Option<Self::Node> {
            if self.nodes.len() == 0 {
                None
            } else {
                Some(0)
            }
        }

        fn exit_node(&self) -> Option<Self::Node> {
            if self.nodes.len() == 0 {
                None
            } else {
                Some(self.nodes.len() - 1)
            }
        }

        fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_ {
            (0..self.nodes.len()).into_iter()
        }

        fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
            self.pred[node].iter().copied()
        }

        fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
            self.succ[node].iter().copied()
        }
    }

    impl IndirectGraph for TestGraph {
        type NodeValue = TestNode;

        fn get(&self, node: Self::Node) -> &Self::NodeValue {
            &self.nodes[node]
        }
    }

    impl MutableIndirectGraph for TestGraph {
        fn get_mut(&mut self, node: Self::Node) -> &mut Self::NodeValue {
            &mut self.nodes[node]
        }
    }

    #[test]
    fn test_basic() {
        let nodes: Vec<_> = vec![101, 102, 103, 104]
            .into_iter()
            .map(|v| TestNode(v))
            .collect();
        let pred = vec![vec![], vec![0, 2], vec![0], vec![2]];
        let succ = vec![vec![1, 2], vec![2], vec![1, 3], vec![]];
        let mut graph = TestGraph { nodes, pred, succ };
        assert_eq!(graph.entry_node(), Some(0));
        assert_eq!(graph.successors(0).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(graph.successors(1).collect::<Vec<_>>(), vec![2]);
        assert_eq!(graph.successors(2).collect::<Vec<_>>(), vec![1, 3]);
        assert_eq!(graph.successors(3).collect::<Vec<_>>(), Vec::<usize>::new());
        let inv = Inverse(&graph);
        assert_eq!(inv.entry_node(), Some(3));
        assert_eq!(inv.successors(0).collect::<Vec<_>>(), Vec::<usize>::new());
        assert_eq!(inv.successors(1).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(inv.successors(2).collect::<Vec<_>>(), vec![0]);
        assert_eq!(inv.successors(3).collect::<Vec<_>>(), vec![2]);
        assert_eq!(inv.get(0), &TestNode(101));
        let mut inv = Inverse(&mut graph);
        assert_eq!(inv.get_mut(1), &mut TestNode(102));
    }
}
