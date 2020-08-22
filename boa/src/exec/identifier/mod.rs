use super::{Context, Executable};
use crate::{builtins::value::Value, syntax::ast::node::identifier::Identifier, Result};

impl Executable for Identifier {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        interpreter
            .realm()
            .environment
            .get_binding_value(self.as_ref())
            .ok_or_else(|| interpreter.construct_reference_error(self.as_ref()))
    }
}
