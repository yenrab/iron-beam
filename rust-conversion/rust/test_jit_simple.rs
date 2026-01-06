use frameworks_emulator_init::main_init::evaluate_erlang_expression_with_bindings;
use infrastructure_utilities::erl_eval::Bindings;

fn main() {
    println!("Testing JIT execution of 2+2...");
    
    let mut bindings = Bindings::new();
    match evaluate_erlang_expression_with_bindings("2+2.", &mut bindings) {
        Ok(term) => {
            println!("Result: {:?}", term);
            match term {
                entities_data_handling::term_hashing::Term::Small(4) => {
                    println!("SUCCESS: 2+2 = 4");
                }
                _ => {
                    println!("Got different result: {:?}", term);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
