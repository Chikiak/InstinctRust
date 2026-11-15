use instinct_core::value::Value;

pub fn int(n: i64) -> Value {
    Value::Int(n)
}

pub fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}
