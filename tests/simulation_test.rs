use std::collections::HashMap;
use std::rc::Rc;
use metsi_rust::configuration_utils::{bound_operation, ParameteredOperation, ParameterMap};
use metsi_rust::branching_generators::{generator_map, GeneratorFn};
use metsi_rust::event_graph::{BoxedOperation, EventDAG, EventNode, EventNodes, OperationChain, UnboundOperation};

fn increment(val: i32, params: ParameterMap) -> i32 {
    let addition = params.get("increase").unwrap().parse::<i32>().unwrap();
    val + addition
}

fn decrement(val: i32, params: ParameterMap) -> i32 {
    let removal = params.get("decrease").unwrap().parse::<i32>().unwrap();
    val - removal
}
fn do_nothing(val: i32) -> i32 {
    val
}



#[test]
fn test_simple_run() {
    let configuration = HashMap::from(
        [
            ("increment", ParameterMap::from([("increase", "2")])),
            ("decrement", ParameterMap::from([("decrease", "1")]))
        ]
    );

    let operation_map = HashMap::from([
        ("increment", increment as ParameteredOperation<i32>),
        ("decrement", decrement as ParameteredOperation<i32>)
    ]);

    let generator_map = generator_map::<i32>();

    let simconfig = Vec::from([
            ("sequence", Vec::from(["increment", "increment"])),
            ("alternatives", Vec::from(["increment", "decrement"])),
            ("sequence", Vec::from(["increment", "increment"]))
    ]);


    let root: EventNode<i32> = EventDAG::new_node(Box::new(do_nothing));

    let mut nodes: EventNodes<i32> = vec![Rc::clone(&root)];

    let sim: Vec<(GeneratorFn<i32>, OperationChain<i32>)> = simconfig.iter().map(|generator_declaration| {
        let generator_fn = *generator_map.get(generator_declaration.0).unwrap();
        let operations: OperationChain<i32> = generator_declaration.1.iter().map(|opname| {
            let op: ParameteredOperation<i32> = *operation_map.get(opname).unwrap();
            let params = configuration.get(opname).unwrap();
            bound_operation(op, params.clone())
        }).collect::<OperationChain<i32>>();
        (generator_fn, operations)
    }).collect();
    for generable in sim {
        nodes = generable.0(nodes, generable.1);
    }

    let result = root.borrow().evaluate_depth(10);
    assert_eq!(vec![20, 17], result);
}