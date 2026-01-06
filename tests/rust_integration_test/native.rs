use pyro_core::interpreter::Value;

pub fn rand_float(_args: Vec<Value>) -> Result<Value, Value> {
    let x: f64 = rand::random();
    Ok(Value::Float(x))
}
