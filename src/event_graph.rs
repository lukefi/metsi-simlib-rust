use std::cell::RefCell;
use std::rc::Rc;

pub type UnboundOperation<T> = dyn Fn(T) -> T;
pub type BoxedOperation<T> = Box<UnboundOperation<T>>;
pub type OperationChain<T> = Vec<UnboundOperation<T>>;
type OperationResults<T> = Vec<T>;
type UniqueChains<T> = Vec<EventNodes<T>>;
pub type EventNode<T> = Rc<RefCell<EventDAG<T>>>;
pub type EventNodes<T> = Vec<EventNode<T>>;

pub struct EventDAG<T> {
    operation: BoxedOperation<T>,
    followers: EventNodes<T>
}

/// EventDAG describes a simulation, optionally branching into alternative events. All nodes
/// of the graph hold a function with signature T -> T, where T represents the simulated state.
/// The functions are thus simulation events. T must implement Copy (or be implicitly copyable)
/// for moving ownership into alternative event branches.
impl<T: Copy> EventDAG<T> {
    /// Construct a new EventDAG<T> node with given Operation<T> function reference
    fn new(operation: &'static UnboundOperation<T>) -> EventDAG<T> {
        EventDAG { operation: Box::new(operation), followers: Vec::new()}
    }

    pub fn new_node(operation: &'static UnboundOperation<T>) -> EventNode<T> {
        Rc::new(RefCell::new(EventDAG { operation: Box::new(operation), followers: Vec::new()}))
    }

    pub fn wrap(self) -> EventNode<T> {
        Rc::new(RefCell::new(self))
    }

    /// Attach another EventDAG<T> into self.
    fn add_branch(&mut self, branch: EventDAG<T>) {
        self.followers.push(Rc::new(RefCell::new(branch)))
    }

    pub fn add_follower_node(&mut self, node: &EventNode<T>) {
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

    /// Generate vectors of EventNode<T> representing unique chains through this
    /// EventDAG<T>. Recursive post-order walkthrough of the graph is performed.
    fn node_chains(wrapped_self: EventNode<T>) -> UniqueChains<T> {
        let mut result = UniqueChains::new();
        if wrapped_self.borrow().followers.len() == 0 {
            let mut current = EventNodes::new();
            current.push(Rc::clone(&wrapped_self));
            result.push(current);
        }
        else {
            for branch in &wrapped_self.borrow().followers {
                let from_branch = EventDAG::node_chains(Rc::clone(branch));
                for chain in from_branch {
                    let mut current = EventNodes::new();
                    current.push(Rc::clone(&wrapped_self));
                    current.extend(chain);
                    result.push(current);
                }
            }
        }
        result
    }

    /// Evaluate unique function chains represented by this EventDAG<T>, producing their
    /// results as a Vec<T>.
    pub fn evaluate_chains(self, payload: T) -> OperationResults<T> {
        let chains = EventDAG::node_chains(Rc::new(RefCell::new(self)));
        let mut results = OperationResults::new();
        for chain in chains {
            let mut current: T = payload;
            for node in chain {
                current = (node.borrow().operation)(current)
            }
            results.push(current)
        }
        results
    }

    /// Evaluate the total computation represented by this EventDAG<T>, producing its results
    /// as a Vec<T>. Recursive pre-order walkthrough is performed.
    pub fn evaluate_depth(&self, payload: T) -> OperationResults<T> {
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
        let mut root = EventDAG::new(&increment);
        let mut s1 = EventDAG::new(&increment);
        let b1 = EventDAG::new(&increment);
        let b2 = EventDAG::new(&increment);
        s1.add_branch(b1);
        s1.add_branch(b2);
        root.add_branch(s1);
        root
    }

    #[test]
    fn chains_are_produced() {
        let root = create_fixture();
        let chains = EventDAG::node_chains(root.wrap());
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
        let root = create_fixture().wrap();
        let mut chains = EventDAG::node_chains(Rc::clone(&root));
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 3);
        assert_eq!(chains[1].len(), 3);

        let leafs = root.borrow().collect_leaf_nodes();
        for leaf in leafs {
            let extension = EventDAG::new(&increment);
            leaf.borrow_mut().add_branch(extension);
        }

        chains = EventDAG::node_chains(Rc::clone(&root));
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 4);
        assert_eq!(chains[1].len(), 4);
    }

    #[test]
    fn nodes_are_shareable() {
        let root = create_fixture();
        let leafs = root.collect_leaf_nodes();
        let extension = Rc::new(RefCell::new(EventDAG::new(&increment)));
        for leaf in leafs {
            leaf.borrow_mut().add_follower_node(&extension);
        }
        let chains = EventDAG::node_chains(root.wrap());
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].len(), 4);
        assert_eq!(chains[1].len(), 4);
    }
}
