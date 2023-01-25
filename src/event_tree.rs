use std::cell::RefCell;
use std::rc::Rc;

type Operation<T> = fn(T) -> T;
type OperationChain<T> = Vec<Operation<T>>;
type OperationResults<T> = Vec<T>;
type UniqueChains<T> = Vec<OperationChain<T>>;
type EventNode<T> = Rc<RefCell<EventDAG<T>>>;
type EventNodes<T> = Vec<EventNode<T>>;

pub struct EventDAG<T> {
    operation: Operation<T>,
    followers: EventNodes<T>
}

/// EventDAG describes a simulation, optionally branching into alternative events. All nodes
/// of the graph hold a function with signature T -> T, where T represents the simulated state.
/// The functions are thus simulation events. T must implement Copy (or be implicitly copyable)
/// for moving ownership into alternative event branches.
impl<T: Copy> EventDAG<T> {
    /// Construct a new EventDAG<T> node with given Operation<T> function reference
    pub fn new(operation: Operation<T>) -> EventDAG<T> {
        EventDAG { operation, followers: Vec::new()}
    }

    /// Attach another EventDAG<T> into self.
    fn add_branch(&mut self, branch: EventDAG<T>) {
        self.followers.push(Rc::new(RefCell::new(branch)))
    }

    fn add_follower_node(&mut self, node: &EventNode<T>) {
        self.followers.push(Rc::clone(node))
    }

    fn is_leaf(&self) -> bool {
        return self.followers.len() == 0
    }

    /// Obtain mutable borrows for leaf nodes of this EventDAG<T>
    fn collect_leaf_nodes(&self) -> EventNodes<T> {
        let mut result = Vec::new();

        for branch in &self.followers {
            if !branch.borrow().is_leaf() {
                result.extend(branch.borrow().collect_leaf_nodes())
            }
            else {
                result.push(Rc::clone(branch))
            }
        }
        return result;
    }

    /// Generate vectors of T => T functions representing unique call chains through this
    /// EventDAG<T>. Recursive post-order walkthrough of the graph is performed.
    fn operation_chains(&self) -> UniqueChains<T> {
        let mut result = UniqueChains::new();
        if self.followers.len() == 0 {
            let mut current = OperationChain::new();
            current.push(self.operation);
            result.push(current);
        }
        else {
            for branch in &self.followers {
                let from_branch = branch.borrow().operation_chains();
                for chain in from_branch {
                    let mut current = OperationChain::new();
                    current.push(self.operation);
                    current.extend(chain);
                    result.push(current);
                }
            }
        }
        result
    }

    /// Evaluate unique function chains represented by this EventDAG<T>, producing their
    /// results as a Vec<T>.
    fn evaluate_chains(self, payload: T) -> OperationResults<T> {
        let chains = self.operation_chains();
        let mut results = OperationResults::new();
        for chain in chains {
            let mut current: T = payload;
            for operation in chain {
                current = operation(current)
            }
            results.push(current)
        }
        results
    }

    /// Evaluate the total computation represented by this EventDAG<T>, producing its results
    /// as a Vec<T>. Recursive pre-order walkthrough is performed.
    fn evaluate_depth(&self, payload: T) -> OperationResults<T> {
        let mut results = OperationResults::new();
        let current = (self.operation)(payload);
        let extension = match &self.followers {
            branches if branches.len() == 0 => {
                vec![current]
            }
            branches => {
                branches
                    .iter()
                    .map(|branch| branch.borrow().evaluate_depth(current))
                    .flatten()
                    .collect()
            }
        };
        results.extend(extension);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn increment(x: i32) -> i32 { x+1 }
    fn create_fixture() -> EventDAG<i32> {
        let mut root = EventDAG::new(increment);
        let mut s1 = EventDAG::new(increment);
        let b1 = EventDAG::new(increment);
        let b2 = EventDAG::new(increment);
        s1.add_branch(b1);
        s1.add_branch(b2);
        root.add_branch(s1);
        root
    }

    #[test]
    fn chains_are_produced() {
        let root = create_fixture();
        let chains = root.operation_chains();
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 3);
        assert_eq!(chains[1].len(), 3);
    }

    #[test]
    fn chains_are_evaluable() {
        let root = create_fixture();
        let results = root.evaluate_chains( 0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], 3);
        assert_eq!(results[1], 3);
    }

    #[test]
    fn graph_is_evaluable() {
        let root = create_fixture();
        let results = root.evaluate_depth(0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], 3);
        assert_eq!(results[1], 3);
    }

    #[test]
    fn graph_is_extensible() {
        let root = create_fixture();
        let mut chains = root.operation_chains();
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 3);
        assert_eq!(chains[1].len(), 3);

        let leafs = root.collect_leaf_nodes();
        for leaf in leafs {
            let extension = EventDAG::new(increment);
            leaf.borrow_mut().add_branch(extension);
        }

        chains = root.operation_chains();
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 4);
        assert_eq!(chains[1].len(), 4);
    }

    #[test]
    fn nodes_are_shareable() {
        let root = create_fixture();
        let leafs = root.collect_leaf_nodes();
        let extension = Rc::new(RefCell::new(EventDAG::new(increment)));
        for leaf in leafs {
            leaf.borrow_mut().add_follower_node(&extension);
        }
        let chains = root.operation_chains();
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 4);
        assert_eq!(chains[1].len(), 4);
    }
}
