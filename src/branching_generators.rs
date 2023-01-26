use std::collections::HashMap;
use std::rc::Rc;
use super::event_graph::*;
pub type GeneratorFn<T> = fn(EventNodes<T>, OperationChain<T>) -> EventNodes<T>;

/// Generate a linear sequence of EventNodes from an OperationChain. Attach it as a follower
/// into each of the given EventNodes.
fn sequence<T: Copy>(previous: EventNodes<T>, operations: OperationChain<T>) -> EventNodes<T> {

    if operations.len() == 0 {
        previous
    }
    else {
        let nodes: EventNodes<T> = operations
            .iter()
            .map(|op| { return EventDAG::new_node(*op) })
            .collect();
        let leaf = nodes.iter().reduce(|acc, cur| {
                acc.borrow_mut().add_follower_node(&cur);
                cur
            }).unwrap();
        let new_root = &nodes[0];
        for prev in previous {
            prev.borrow_mut().add_follower_node(new_root);
        }
        vec![Rc::clone(leaf)]
    }
}

/// Generate a collection of individual EventNodes from an OperationChain. Attach each of them as a
/// follower into each of the given EventNodes.
fn alternatives<T: Copy>(previous: EventNodes<T>, operations: OperationChain<T>) -> EventNodes<T> {

    if operations.len() == 0 {
        previous
    } else {
        let nodes: EventNodes<T> = operations
            .iter()
            .map(|op| { return EventDAG::new_node(*op) })
            .collect();
        for prev in previous {
            for node in nodes.iter() {
                prev.borrow_mut().add_follower_node(&node)
            }
        }
        nodes
    }
}


/// Get a map of generator functions resolvable from strings.
pub fn generator_map<T: Copy>() -> HashMap<&'static str, GeneratorFn<T>> {
    HashMap::from([
        ("sequence", sequence as GeneratorFn<T>),
        ("alternatives", alternatives as GeneratorFn<T>)
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn increment(x: i32) -> i32 { x + 1 }
    fn do_nothing(x: i32) -> i32 { x }

    fn create_ops(operation: fn(i32) -> i32, times: i32) -> OperationChain<i32> {
        let mut ops = Vec::new();
        for _i in 0..times {
            ops.push(operation)
        }
        ops
    }

    #[test]
    fn test_generator_mapping() {
        let map = generator_map();
        let gen_fn = map.get("sequence").unwrap();
        let generator_root = EventDAG::new_node(do_nothing);
        let graph = gen_fn(vec![Rc::clone(&generator_root)], create_ops(increment, 2));
        let result = generator_root.borrow().evaluate_depth(0);
        assert_eq!(1, graph.len());
        assert_eq!(2, result[0])
    }

    #[test]
    fn test_graph_extending() {
        let generator_root = EventDAG::new_node(do_nothing);

        let level_1 = sequence(vec![generator_root.clone()], create_ops(increment, 2));
        let level_2 = alternatives(level_1, create_ops(increment, 2));
        let level_3 = alternatives(level_2, create_ops(increment, 2));

        let chains_results = generator_root.borrow().evaluate_chains(0);
        let depth_results = generator_root.borrow().evaluate_depth(0);

        assert_eq!(vec![4, 4, 4, 4], chains_results);
        assert_eq!(vec![4, 4, 4, 4], depth_results);
        assert_eq!(level_3.len(), 2);
    }
}