// Debug JIT execution directly
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn main() {
    println!("=== DEBUG JIT EXECUTION ===");
    
    // Try to trigger JIT compilation and execution
    use frameworks_emulator_init::main_init::evaluate_erlang_expression_with_bindings;
    use infrastructure_utilities::erl_eval::Bindings;
    
    let mut bindings = Bindings::new();
    println!("Evaluating 2+2...");
    
    match evaluate_erlang_expression_with_bindings("2+2.", &mut bindings) {
        Ok(term) => {
            println!("SUCCESS: Result = {:?}", term);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    
    println!("=== DEBUG COMPLETE ===");
}
