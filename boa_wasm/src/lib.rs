use boa::{exec::Executable, syntax::Parser, Context};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    let expr = Parser::new(src.as_bytes())
        .parse_all()
        .map_err(|e| JsValue::from(format!("Parsing Error: {}", e)))?;

    // Setup executor
    let mut engine = Context::new();

    // Setup executor
    expr.run(&mut engine)
        .map_err(|e| JsValue::from(format!("Error: {}", e.display())))
        .map(|v| v.display().to_string())
}
