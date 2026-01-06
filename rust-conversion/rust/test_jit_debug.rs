// Test JIT execution with debugging
fn main() {
    println!("=== TESTING JIT EXECUTION WITH DEBUGGING ===");
    
    // This should trigger the JIT compilation and execution path
    use frameworks_emulator_init::main_init::evaluate_erlang_expression_with_bindings;
    use infrastructure_utilities::erl_eval::Bindings;
    
    let mut bindings = Bindings::new();
    println!("Evaluating 2+2...");
    
    match evaluate_erlang_expression_with_bindings("2+2.", &mut bindings) {
        Ok(term) => {
            println!("SUCCESS: Result = {:?}", term);
        }
        Err(e) => {
            println!("Evaluation failed: {}", e);
        }
    }
    
    println!("=== TEST COMPLETE ===");
}
