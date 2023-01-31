use std::collections::HashMap;
pub type ParameterMap = HashMap<&'static str, &'static str>;
type ParameteredOperation<'a, T> = fn(T, &'a ParameterMap) -> T;


pub fn bound_operation<'a, T: 'a>(op: ParameteredOperation<'a, T>, params: &'a ParameterMap) -> Box<dyn Fn(T) -> T + 'a> {
    Box::new(move |payload| op(payload, params))
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::branching_generators::{generator_map, sequence};
    use crate::configuration_utils::*;
    use crate::event_graph::EventDAG;

    fn parametered_increment(val: i32, params: &ParameterMap) -> i32 {
        let addition = params.get("increase").unwrap().parse::<i32>().unwrap();
        val + addition
    }

    #[test]
    fn operation_binding_works() {
        let mut params = ParameterMap::new();
        params.insert("increase", "2");
        let operation = bound_operation(parametered_increment, &params);
        let mut val = 0;
        val = operation(val);
        val = operation(val);
        assert_eq!(4, val);
    }
}