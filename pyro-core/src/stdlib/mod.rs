pub mod math;
pub mod io;
pub mod time;

use crate::interpreter::Interpreter;

pub fn register_std_libs(interpreter: &mut Interpreter) {
    interpreter.register_native_module("std.math", math::module());
    interpreter.register_native_module("std.io", io::module());
    interpreter.register_native_module("std.time", time::module());
}
