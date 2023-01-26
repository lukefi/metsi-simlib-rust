use std::collections::HashMap;
pub type ParameterMap = HashMap<&'static str, &'static str>;
type ParameteredOperation<'a, T> = fn(T, &'a ParameterMap) -> T;


pub fn bound_operation<'a, T: 'a>(op: ParameteredOperation<'a, T>, params: &'a ParameterMap) -> impl Fn(T) -> T + 'a {
    move |payload| op(payload, params)
}

#[cfg(test)]
mod tests {
    use crate::operation_utils::*;

    fn parametered_increment(val: i32, params: &ParameterMap) -> i32 {
        let addition = params.get("increase").unwrap().parse::<i32>().unwrap();
        val + addition
    }

    #[test]
    fn partial_application_works() {
        let mut params = ParameterMap::new();
        params.insert("increase", "2");
        let operation = bound_operation(parametered_increment, &params);
        let mut val = 0;
        val = operation(val);
        val = operation(val);
        assert_eq!(4, val);
    }
}