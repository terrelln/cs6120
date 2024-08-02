use super::graph::Graph;
use std::{
    collections::{HashMap, HashSet},
    vec,
};

pub struct DominanceTree<'a, G: Graph> {
    reachable_nodes: HashSet<G::Node>,
    immediately_dominates: HashMap<G::Node, Vec<G::Node>>,
    immediate_dominator: HashMap<G::Node, G::Node>,
    graph: &'a G,
}

impl<'a, G: Graph> DominanceTree<'a, G> {
    pub fn new(graph: &'a G) -> Self {
        let reachable_nodes = graph.pre_order_iter().collect();
        let immediately_dominates = Self::build_immediately_dominates(graph);
        let immediate_dominator = Self::build_immediate_dominator(&immediately_dominates);
        let immediately_dominates = immediately_dominates
            .into_iter()
            .map(|(node, dominated)| (node, dominated.into_iter().collect()))
            .collect();
        DominanceTree {
            reachable_nodes,
            immediately_dominates,
            immediate_dominator,
            graph,
        }
    }

    pub fn immediate_dominator(&self, node: G::Node) -> Option<G::Node> {
        let dominator = self.immediate_dominator.get(&node).map(|x| *x);
        assert!(
            dominator.is_some()
                || !self.reachable_nodes.contains(&node)
                || Some(node) == self.graph.entry_node()
        );
        dominator
    }

    pub fn immediately_dominated_nodes(&self, node: G::Node) -> impl Iterator<Item = G::Node> + '_ {
        let imm_dom = self.immediately_dominates.get(&node);
        match imm_dom {
            Some(imm_dom) => imm_dom.iter().cloned(),
            None => [].iter().cloned(),
        }
    }

    pub fn dominated_nodes(&self, node: G::Node) -> impl Iterator<Item = G::Node> + '_ {
        let stack = if self.reachable_nodes.contains(&node) {
            vec![node]
        } else {
            vec![]
        };
        DominatedIterator {
            tree: self,
            stack: stack,
        }
    }

    pub fn strictly_dominated_nodes(&self, node: G::Node) -> impl Iterator<Item = G::Node> + '_ {
        DominatedIterator {
            tree: self,
            stack: self.immediately_dominated_nodes(node).collect(),
        }
    }

    pub fn dominance_frontier(&self, node: G::Node) -> impl Iterator<Item = G::Node> + '_ {
        let strictly_dominated = self.strictly_dominated_nodes(node).collect::<HashSet<_>>();
        let dominance_frontier = self
            .dominated_nodes(node)
            .flat_map(|dom| self.graph.successors(dom))
            .filter(|succ| !strictly_dominated.contains(succ))
            .collect::<HashSet<_>>();
        dominance_frontier.into_iter()
    }
}

struct DominatedIterator<'a, 'b, G: Graph> {
    tree: &'a DominanceTree<'b, G>,
    stack: Vec<G::Node>,
}

impl<'a, 'b, G: Graph> Iterator for DominatedIterator<'a, 'b, G> {
    type Item = G::Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            self.stack
                .extend(self.tree.immediately_dominated_nodes(node));
            node
        })
    }
}

impl<'a, G: Graph> DominanceTree<'a, G> {
    fn build_immediately_dominates(graph: &'a G) -> HashMap<G::Node, HashSet<G::Node>> {
        let dominators = Self::build_dominators(graph);
        let strictly_dominates = Self::build_strictly_dominates(&dominators);
        let mut imm_dom = HashMap::new();

        for (node, dominated) in &strictly_dominates {
            for dom in dominated {
                assert!(dominators[dom].contains(node));
                let intersection = dominators[dom].intersection(dominated).count();
                if intersection == 1 {
                    imm_dom
                        .entry(*node)
                        .or_insert_with(HashSet::new)
                        .insert(*dom);
                }
            }
        }

        imm_dom
    }

    fn build_immediate_dominator(
        immediately_dominates: &HashMap<G::Node, HashSet<G::Node>>,
    ) -> HashMap<G::Node, G::Node> {
        let mut immediate_dominator = HashMap::new();

        for (dominator, dominated) in immediately_dominates {
            for node in dominated {
                assert!(!immediate_dominator.contains_key(node));
                immediate_dominator.insert(*node, *dominator);
            }
        }

        immediate_dominator
    }

    fn build_strictly_dominates(
        dominators: &HashMap<G::Node, HashSet<G::Node>>,
    ) -> HashMap<G::Node, HashSet<G::Node>> {
        let mut dominates: HashMap<_, _> = dominators
            .keys()
            .cloned()
            .map(|node| (node, HashSet::new()))
            .collect();

        for (node, dominators) in dominators {
            for dominator in dominators {
                dominates.get_mut(dominator).unwrap().insert(*node);
            }
        }

        for (node, dominated) in &mut dominates {
            dominated.remove(node);
        }

        dominates
    }

    fn build_dominators(graph: &'a G) -> HashMap<G::Node, HashSet<G::Node>> {
        let mut dom = HashMap::new();
        if graph.entry_node().is_none() {
            // No entry node => no node dominates any other
            return dom;
        }

        // Initialize the entry node to be dominated by only itself
        let entry_node = graph.entry_node().unwrap();
        dom.insert(entry_node, HashSet::from([entry_node]));

        let iter = graph.post_order_iter().rev();
        assert!(iter.clone().next() == graph.entry_node());

        // Initialize every other node reachable from the entry node to be
        // dominated by every node reachable from the entry node.
        for node in iter.clone().skip(1) {
            dom.insert(node, iter.clone().collect());
        }
        let reachable_nodes = iter.clone().collect::<HashSet<_>>();

        loop {
            let mut changed = false;
            for node in iter.clone().skip(1) {
                assert!(Some(node) != graph.entry_node());
                let pred_dom = graph
                    .predecessors(node)
                    .filter(|node| reachable_nodes.contains(node))
                    .map(|pred| dom.get(&pred).unwrap())
                    .fold(None, |acc, x| match acc {
                        Some(acc) => Some(&acc & x),
                        None => Some(x.clone()),
                    });
                let mut pred_dom = pred_dom.unwrap_or_default();
                pred_dom.insert(node);
                dom.entry(node).and_modify(|x| {
                    changed |= x != &pred_dom;
                    *x = pred_dom;
                });
            }
            if !changed {
                break;
            }
        }
        dom
    }
}

impl<'a, G:Graph> Graph for DominanceTree<'a, G> {
    type Node = G::Node;

    fn entry_node(&self) -> Option<Self::Node> {
        self.graph.entry_node()
    }

    fn exit_node(&self) -> Option<Self::Node> {
        // Inverting the tree doesn't make sense
        None
    }

    fn nodes(&self) -> impl Iterator<Item = Self::Node> + '_ {
        self.reachable_nodes.iter().cloned()
    }

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.immediate_dominator(node).into_iter()
    }
    
    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.immediately_dominated_nodes(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::graph::tests::SimpleGraph;
    use rand::distributions::Uniform;
    use rand::prelude::*;

    fn is_reachable_from<G: Graph>(graph: &G, src: G::Node, dst: G::Node) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![dst];

        while let Some(node) = stack.pop() {
            if node == src {
                return true;
            }
            visited.insert(node);
            stack.extend(graph.predecessors(node).filter(|x| !visited.contains(x)));
        }

        return false;
    }

    fn dominates<G: Graph>(graph: &G, dominator: G::Node, dominated: G::Node) -> bool {
        if graph.entry_node().is_none() {
            return false;
        }
        let entry_node = graph.entry_node().unwrap();
        if !is_reachable_from(graph, entry_node, dominator) {
            return false;
        }
        let mut predecessors = HashSet::new();
        let mut visited = HashSet::new();
        predecessors.insert(dominated);

        while !predecessors.is_empty() {
            // Don't traverse through the dominator node, just mark it as visited.
            if predecessors.contains(&dominator) {
                visited.insert(dominator);
                predecessors.remove(&dominator);
                continue;
            }
            // If we've reached the entry node without traversing through dom,
            // then dom doesn't dominate node.
            if predecessors.contains(&entry_node) {
                assert_ne!(dominator, entry_node);
                return false;
            }

            // Pop a node from the predecessors list, mark as visited, and add all of its unvisited predecessors
            let node = predecessors.iter().next().unwrap().clone();
            visited.insert(node);
            predecessors.remove(&node);
            predecessors.extend(graph.predecessors(node).filter(|x| !visited.contains(x)));
        }

        // dominator dominates dominated iff we've reached the dominator node
        visited.contains(&dominator)
    }

    fn test_dominance_tree<G: Graph>(graph: &G, dom: &DominanceTree<G>) {
        let all_nodes = graph.nodes().collect::<HashSet<_>>();
        for dominator in graph.nodes() {
            // Test that dominated nodes is correct by re-computing the dominates relationship naively
            let dominated_nodes = dom.dominated_nodes(dominator).collect::<HashSet<_>>();
            for dominated in &dominated_nodes {
                assert!(dominates(&graph, dominator, *dominated));
            }
            for not_dominated in all_nodes.difference(&dominated_nodes) {
                assert!(!dominates(&graph, dominator, *not_dominated));
            }

            // Test that strictly dominated nodes is equal to dominated - self
            let strictly_dominated_nodes = dom
                .strictly_dominated_nodes(dominator)
                .collect::<HashSet<_>>();
            assert_eq!(
                strictly_dominated_nodes,
                &dominated_nodes - &HashSet::from([dominator])
            );

            // Test that immediately dominated nodes is correct
            let immediately_dominated_nodes = dom
                .immediately_dominated_nodes(dominator)
                .collect::<HashSet<_>>();
            // Every immediately dominated node is strictly dominated
            assert_eq!(
                immediately_dominated_nodes,
                &immediately_dominated_nodes & &strictly_dominated_nodes,
            );
            // No strictly dominated node's strict dominators contains any node that is immediately dominated
            for node in &strictly_dominated_nodes {
                let dominated = dom.strictly_dominated_nodes(*node).collect::<HashSet<_>>();
                assert!((&immediately_dominated_nodes & &dominated).is_empty());
            }

            // Test that immediate_dominator is the inverse relation of immediately dominated
            for dominated in &immediately_dominated_nodes {
                assert_eq!(dom.immediate_dominator(*dominated), Some(dominator));
            }

            // Test that the dominance frontier is correct
            let dominance_frontier = dom.dominance_frontier(dominator).collect::<HashSet<_>>();
            // No node in the dominance frontier is strictly dominated
            assert!((&strictly_dominated_nodes & &dominance_frontier).is_empty());
            // Every node in the dominance frontier has a predecessor that is dominated
            for node in dominance_frontier {
                let predecessors = graph.predecessors(node).collect::<HashSet<_>>();
                assert!(!(&dominated_nodes & &predecessors).is_empty());
            }
        }
    }

    #[test]
    fn test_basic() {
        let mut graph = SimpleGraph::new();
        graph.set_entry(1);
        graph.extend_nodes(1..6);
        graph.extend_edges(
            vec![
                (1, 2),
                (2, 3),
                (2, 4),
                (2, 6),
                (3, 5),
                (4, 5),
                (5, 2),
                (2, 6),
            ]
            .into_iter(),
        );
        let dom = DominanceTree::new(&graph);

        assert_eq!(dom.immediate_dominator(1), None);
        assert_eq!(dom.immediate_dominator(2), Some(1));
        assert_eq!(dom.immediate_dominator(3), Some(2));
        assert_eq!(dom.immediate_dominator(4), Some(2));
        assert_eq!(dom.immediate_dominator(5), Some(2));
        assert_eq!(dom.immediate_dominator(6), Some(2));

        test_dominance_tree(&graph, &dom);
    }

    #[test]
    fn test_random_graphs() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..100 {
            let mut graph = SimpleGraph::new();
            graph.set_entry(0);
            graph.extend_nodes(0..10);

            let num_edges = Uniform::new(0, 100).sample(&mut rng);
            for _ in 0..num_edges {
                let src = Uniform::new(0, 10).sample(&mut rng);
                let dst = Uniform::new(1, 10).sample(&mut rng);
                graph.add_edge(src, dst);
            }

            let dom = DominanceTree::new(&graph);
            test_dominance_tree(&graph, &dom);
        }
    }
}
