type Operation<T> = fn(T) -> T;
type OperationChain<T> = Vec<Operation<T>>;
type OperationResults<T> = Vec<T>;
type UniqueChains<T> = Vec<OperationChain<T>>;

pub struct EventTree<T> {
    operation: Operation<T>,
    branches: Vec<EventTree<T>>
}

/// EventTree describes a simulation, optionally branching into alternative events. All nodes
/// of the tree hold a function with signature T -> T, where T represents the simulated state.
/// The functions are thus simulation events. T must implement Copy (or be implicitly copyable)
/// for moving ownership into alternative event branches.
impl<T: Copy> EventTree<T> {
    /// Construct a new EventTree<T> node with given Operation<T> function reference
    pub fn new(operation: Operation<T>) -> EventTree<T> {
        EventTree { operation, branches: Vec::new()}
    }

    /// Attach another EventTree<T> into self.
    fn add_branch(&mut self, branch: EventTree<T>) {
        self.branches.push(branch)
    }

    /// Generate vectors of T => T functions representing unique call chains through this
    /// EventTree<T>. Recursive post-order walkthrough of the tree is performed.
    fn operation_chains(&self) -> UniqueChains<T> {
        let mut result = UniqueChains::new();
        if self.branches.len() == 0 {
            let mut current = OperationChain::new();
            current.push(self.operation);
            result.push(current);
        }
        else {
            for branch in &self.branches {
                let from_branch = branch.operation_chains();
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

    /// Evaluate unique function chains represented by this EventTree<T>, producing their
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

    /// Evaluate the total computation represented by this EventTree<T>, producing its results
    /// as a Vec<T>. Recursive pre-order walkthrough is performed.
    fn evaluate_depth(&self, payload: T) -> OperationResults<T> {
        let mut results = OperationResults::new();
        let current = (self.operation)(payload);
        if self.branches.len() == 0 {
            results.push(current)
        }
        else if self.branches.len() == 1 {
            results.extend(self.branches.first().unwrap().evaluate_depth(current))
        }
        else if self.branches.len() > 1 {
            for branch in &self.branches {
                results.extend(branch.evaluate_depth(current))
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn increment(x: i32) -> i32 { x+1 }
    fn create_fixture() -> EventTree<i32> {
        let mut root = EventTree::new(increment);
        let mut s1 = EventTree::new(increment);
        let b1 = EventTree::new(increment);
        let b2 = EventTree::new(increment);
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
    fn tree_is_evaluable() {
        let root = create_fixture();
        let results = root.evaluate_depth(0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], 3);
        assert_eq!(results[1], 3);
    }
}
