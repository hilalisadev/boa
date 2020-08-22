use super::{Context, Executable};
use crate::{builtins::value::Value, syntax::ast::node::Throw, Result};

impl Executable for Throw {
    #[inline]
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        Err(self.expr().run(interpreter)?)
    }
}
