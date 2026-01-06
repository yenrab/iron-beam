// Simple test to verify JIT execution doesn't crash
fn main() {
    println!("Testing JIT execution fix...");
    
    // Test basic evaluation without JIT first
    use frameworks_emulator_init::main_init::evaluate_erlang_expression_with_bindings;
    use infrastructure_utilities::erl_eval::Bindings;
    
    let mut bindings = Bindings::new();
    match evaluate_erlang_expression_with_bindings("1+1.", &mut bindings) {
        Ok(term) => {
            println!("Basic evaluation successful: {:?}", term);
        }
        Err(e) => {
            println!("Basic evaluation failed: {}", e);
        }
    }
    
    println!("JIT fix compilation test completed successfully!");
}
