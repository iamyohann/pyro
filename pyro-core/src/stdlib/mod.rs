pub mod math;
pub mod fs;
pub mod time;
pub mod env;
pub mod path;
pub mod process;
pub mod json;
pub mod random;

use crate::interpreter::Interpreter;

pub fn register_std_libs(interpreter: &mut Interpreter) {
    interpreter.register_native_module("std.math", math::module());
    interpreter.register_native_module("std.fs", fs::module());
    interpreter.register_native_module("std.time", time::module());
    interpreter.register_native_module("std.env", env::module());
    interpreter.register_native_module("std.path", path::module());
    interpreter.register_native_module("std.process", process::module());
    interpreter.register_native_module("std.json", json::module());
    interpreter.register_native_module("std.random", random::module());
}
